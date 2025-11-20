/**
 * AI-Powered UI/UX Service HTTP Server
 */

import cors from "cors";
import express, { type Request, type Response } from "express";
import type {
	GestureRecognitionRequest,
	GestureRecognitionResponse,
	InterfaceAdjustmentRequest,
	InterfaceAdjustmentResponse,
	NotificationFilterRequest,
	NotificationFilterResponse,
} from "./types.js";
import { UIAIEngine } from "./ui_engine.js";

const PORT = 9011;

export class UIAIServer {
	private app: express.Application;
	private uiEngine: UIAIEngine;

	constructor() {
		this.app = express();
		this.uiEngine = new UIAIEngine();
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
			res.json({ status: "ok", service: "ui-ai" });
		});

		// Recognize gesture
		this.app.post("/api/ui/gesture", async (req: Request, res: Response) => {
			try {
				const request: GestureRecognitionRequest = req.body;
				const response: GestureRecognitionResponse = await this.uiEngine.recognizeGesture(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Adjust interface
		this.app.post("/api/ui/adjust", async (req: Request, res: Response) => {
			try {
				const request: InterfaceAdjustmentRequest = req.body;
				const response: InterfaceAdjustmentResponse = await this.uiEngine.adjustInterface(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Filter notifications
		this.app.post("/api/ui/notifications/filter", async (req: Request, res: Response) => {
			try {
				const request: NotificationFilterRequest = req.body;
				const response: NotificationFilterResponse =
					await this.uiEngine.filterNotifications(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`AI-Powered UI/UX Service (ui-ai) listening on port ${PORT}`);
		});
	}
}
