/**
 * Autonomous Update Manager HTTP Server
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import { UpdateManagerEngine } from "./update_engine.js";
import type {
	UpdateScheduleRequest,
	UpdateScheduleResponse,
	UpdateImpactRequest,
	UpdateImpactResponse,
	RollbackRequest,
	RollbackResponse,
} from "./types.js";

const PORT = 9010;

export class UpdateManagerServer {
	private app: express.Application;
	private updateEngine: UpdateManagerEngine;

	constructor() {
		this.app = express();
		this.updateEngine = new UpdateManagerEngine();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		this.app.use(cors());
		this.app.use(express.json());
	}

	private setupRoutes(): void {
		// Health check
		this.app.get("/health", (req: Request, res: Response) => {
			res.json({ status: "ok", service: "update-manager" });
		});

		// Schedule updates
		this.app.post("/api/updates/schedule", async (req: Request, res: Response) => {
			try {
				const request: UpdateScheduleRequest = req.body;
				const response: UpdateScheduleResponse =
					await this.updateEngine.scheduleUpdates(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Assess impact
		this.app.post("/api/updates/impact", async (req: Request, res: Response) => {
			try {
				const request: UpdateImpactRequest = req.body;
				const response: UpdateImpactResponse = await this.updateEngine.assessImpact(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Rollback update
		this.app.post("/api/updates/rollback", async (req: Request, res: Response) => {
			try {
				const request: RollbackRequest = req.body;
				const response: RollbackResponse = await this.updateEngine.rollback(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`Autonomous Update Manager (update-manager) listening on port ${PORT}`);
		});
	}
}

