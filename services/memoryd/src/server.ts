/**
 * Memory Fabric HTTP Server
 *
 * Production-grade HTTP REST API server for memory fabric service.
 * Exposes memory write, read, versioning, and snapshot endpoints.
 */

import cors from "cors";
import express, { type Request, type Response, type NextFunction } from "express";
import type { MemoryFabricService } from "./index.js";

const PORT = 9002;

interface MemoryEntry {
	data: Uint8Array;
	versionId: number;
	leaseId?: string;
	leaseExpiresAt?: number;
}

export class MemoryFabricServer {
	private readonly app: express.Application;
	private readonly service: MemoryFabricService;
	private readonly memoryStore: Map<string, MemoryEntry[]>;
	private server: ReturnType<express.Application["listen"]> | null = null;
	private versionCounter = 0;

	constructor(service: MemoryFabricService) {
		this.service = service;
		this.memoryStore = new Map();
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
		this.startLeaseCleanup();
	}

	private setupMiddleware(): void {
		// CORS for localhost development
		this.app.use(
			cors({
				origin: ["http://localhost:9002", "http://127.0.0.1:9002"],
				credentials: true,
			})
		);

		// JSON body parser
		this.app.use(express.json({ limit: "10mb" }));

		// Request logging
		this.app.use((req: Request, _res: Response, next: NextFunction) => {
			console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
			next();
		});

		// Error handler
		this.app.use((err: Error, _req: Request, res: Response, _next: NextFunction) => {
			console.error("Error:", err);
			res.status(500).json({
				success: false,
				error: err.message || "Internal server error",
			});
		});
	}

	private setupRoutes(): void {
		// Memory Operations
		this.app.post("/api/memory/write", this.handleWrite.bind(this));
		this.app.get("/api/memory/read/:key", this.handleRead.bind(this));
		this.app.get("/api/memory/read/:key/:version", this.handleReadVersion.bind(this));
		this.app.get("/api/memory/snapshot", this.handleSnapshot.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "memoryd" });
		});
	}

	// Memory Operation Handlers

	private async handleWrite(req: Request, res: Response): Promise<void> {
		try {
			const body = req.body as {
				key?: string;
				data?: string; // base64 encoded
				lease_duration_ms?: number;
			};

			const key = body.key;
			const data = body.data;
			const lease_duration_ms = body.lease_duration_ms;

			if (!key || !data) {
				res.status(400).json({
					success: false,
					error: "Missing required fields: key and data",
				});
				return;
			}

			// Decode base64 data
			let dataBytes: Uint8Array;
			try {
				const buffer = Buffer.from(data, "base64");
				dataBytes = new Uint8Array(buffer);
			} catch (error) {
				res.status(400).json({
					success: false,
					error: "Invalid base64 data",
				});
				return;
			}

			// Store in memory fabric service
			await this.service.store("system", dataBytes, {
				key,
				lease_duration_ms,
			});

			// Also store in local map for retrieval
			const versionId = ++this.versionCounter;
			const entry: MemoryEntry = {
				data: dataBytes,
				versionId,
				leaseId: lease_duration_ms ? `lease_${key}_${Date.now()}` : undefined,
				leaseExpiresAt: lease_duration_ms ? Date.now() + lease_duration_ms : undefined,
			};

			if (!this.memoryStore.has(key)) {
				this.memoryStore.set(key, []);
			}

			const entries = this.memoryStore.get(key);
			if (entries) {
				entries.push(entry);
			}

			res.json({
				success: true,
				version_id: versionId,
				lease_id: entry.leaseId,
			});
		} catch (error) {
			const err = _error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to write to memory fabric",
			});
		}
	}

	private async handleRead(req: Request, res: Response): Promise<void> {
		try {
			const key = req.params["key"];

			if (!key) {
				res.status(400).json({
					success: false,
					error: "Missing key parameter",
				});
				return;
			}

			const entries = this.memoryStore.get(key);
			if (!entries || entries.length === 0) {
				res.status(404).json({
					success: false,
					error: "Memory entry not found",
				});
				return;
			}

			// Get latest entry (not expired)
			const now = Date.now();
			const validEntries = entries.filter(
				(entry) => !entry.leaseExpiresAt || entry.leaseExpiresAt > now
			);

			if (validEntries.length === 0) {
				res.status(404).json({
					success: false,
					error: "Memory entry expired or not found",
				});
				return;
			}

			const latestEntry = validEntries[validEntries.length - 1];
			if (!latestEntry) {
				res.status(404).json({
					success: false,
					error: "Memory entry not found",
				});
				return;
			}

			const dataBase64 = Buffer.from(latestEntry.data).toString("base64");

			res.json({
				data: dataBase64,
				version_id: latestEntry.versionId,
			});
		} catch (error) {
			const err = _error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to read from memory fabric",
			});
		}
	}

	private async handleReadVersion(req: Request, res: Response): Promise<void> {
		try {
			const key = req.params["key"];
			const versionStr = req.params["version"];

			if (!key || !versionStr) {
				res.status(400).json({
					success: false,
					error: "Missing key or version parameter",
				});
				return;
			}

			const version = Number.parseInt(versionStr, 10);

			if (Number.isNaN(version)) {
				res.status(400).json({
					success: false,
					error: "Invalid version number",
				});
				return;
			}

			const entries = this.memoryStore.get(key);
			if (!entries || entries.length === 0) {
				res.status(404).json({
					success: false,
					error: "Memory entry not found",
				});
				return;
			}

			const entry = entries.find((e) => e.versionId === version);
			if (!entry) {
				res.status(404).json({
					success: false,
					error: "Memory version not found",
				});
				return;
			}

			// Check if expired
			if (entry.leaseExpiresAt && entry.leaseExpiresAt <= Date.now()) {
				res.status(404).json({
					success: false,
					error: "Memory entry expired",
				});
				return;
			}

			const dataBase64 = Buffer.from(entry.data).toString("base64");

			res.json({
				data: dataBase64,
				version_id: entry.versionId,
			});
		} catch (error) {
			const err = _error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to read memory version",
			});
		}
	}

	private async handleSnapshot(_req: Request, res: Response): Promise<void> {
		try {
			// Create snapshot of memory fabric state
			const snapshot: Record<string, unknown> = {};

			for (const [key, entries] of this.memoryStore.entries()) {
				const latestEntry = entries.length > 0 ? entries[entries.length - 1] : undefined;
				snapshot[key] = {
					versions: entries.length,
					latest_version: latestEntry ? latestEntry.versionId : null,
				};
			}

			res.json({
				snapshot,
			});
		} catch (error) {
			const err = _error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to create snapshot",
			});
		}
	}

	private startLeaseCleanup(): void {
		setInterval(() => {
			const now = Date.now();
			for (const [key, entries] of this.memoryStore.entries()) {
				const validEntries = entries.filter(
					(entry) => !entry.leaseExpiresAt || entry.leaseExpiresAt > now
				);
				if (validEntries.length !== entries.length) {
					this.memoryStore.set(key, validEntries);
				}
			}
		}, 60000); // Every minute
	}

	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				const server = this.app.listen(PORT, () => {
					console.log(`Memory Fabric Service listening on port ${PORT}`);
					resolve();
				});

				server.on("error", (err: Error) => {
					console.error("Server error:", err);
					reject(err);
				});

				this.server = server;
			} catch (error) {
				reject(_error);
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
						console.log("Memory Fabric Service stopped");
						resolve();
					}
				});
			} else {
				resolve();
			}
		});
	}
}
