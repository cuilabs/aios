/**
 * Boot Log HTTP Server
 * 
 * Exposes boot log endpoints for boot reproducibility testing
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import { BootLogManager } from "./bootlog.js";

const PORT = 9005;

export class BootLogServer {
	private readonly app: express.Application;
	private readonly bootLogManager: BootLogManager;
	private server: ReturnType<express.Application["listen"]> | null = null;

	constructor() {
		this.bootLogManager = new BootLogManager();
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
		this.bootLogManager.initialize().catch((err) => {
			console.error("Failed to initialize boot log manager:", err);
		});
	}

	private setupMiddleware(): void {
		this.app.use(cors({
			origin: ["http://localhost:9005", "http://127.0.0.1:9005"],
			credentials: true,
		}));
		this.app.use(express.json());
	}

	private setupRoutes(): void {
		this.app.get("/api/boot/log", this.handleGetBootLog.bind(this));
		this.app.post("/api/boot/entry", this.handleWriteBootEntry.bind(this));
	}

	private async handleGetBootLog(req: Request, res: Response): Promise<void> {
		try {
			const bootId = req.query.boot_id as string | undefined;
			const log = await this.bootLogManager.readBootLog(bootId);
			res.json({
				log,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to read boot log",
			});
		}
	}

	private async handleWriteBootEntry(req: Request, res: Response): Promise<void> {
		try {
			const { entry } = req.body as { entry?: string };
			if (!entry) {
				res.status(400).json({
					success: false,
					error: "Missing required field: entry",
				});
				return;
			}
			await this.bootLogManager.writeEntry(entry);
			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to write boot entry",
			});
		}
	}

	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				const server = this.app.listen(PORT, () => {
					console.log(`Boot Log Server listening on port ${PORT}`);
					resolve();
				});
				server.on("error", (err: Error) => {
					console.error("Server error:", err);
					reject(err);
				});
				this.server = server;
			} catch (error) {
				reject(error);
			}
		});
	}

	async stop(): Promise<void> {
		return new Promise((resolve, reject) => {
			if (this.server) {
				this.server.close((err?: Error) => {
					if (err) {
						reject(err);
					} else {
						console.log("Boot Log Server stopped");
						resolve();
					}
				});
			} else {
				resolve();
			}
		});
	}
}

