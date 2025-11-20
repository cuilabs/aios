/**
 * Adaptive Learning Service HTTP Server
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import { LearningEngine } from "./learning_engine.js";
import type {
	LearningRequest,
	LearningResponse,
	PredictionRequest,
	PredictionResponse,
} from "./types.js";

const PORT = 9008;

export class LearningServer {
	private app: express.Application;
	private learningEngine: LearningEngine;

	constructor() {
		this.app = express();
		this.learningEngine = new LearningEngine();
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
			res.json({ status: "ok", service: "learningd" });
		});

		// Learn from behavior
		this.app.post("/api/learning/learn", async (req: Request, res: Response) => {
			try {
				const request: LearningRequest = req.body;
				const response: LearningResponse = await this.learningEngine.learn(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Predict user actions
		this.app.post("/api/learning/predict", async (req: Request, res: Response) => {
			try {
				const request: PredictionRequest = req.body;
				const response: PredictionResponse = await this.learningEngine.predict(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Get user profile
		this.app.get("/api/learning/profile/:userId", (req: Request, res: Response) => {
			try {
				const { userId } = req.params;
				const profile = this.learningEngine.getProfile(userId);
				if (profile) {
					res.json(profile);
				} else {
					res.status(404).json({ error: "Profile not found" });
				}
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`Adaptive Learning Service (learningd) listening on port ${PORT}`);
		});
	}
}

