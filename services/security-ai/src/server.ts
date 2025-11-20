/**
 * Security AI Service HTTP Server
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import { SecurityAIEngine } from "./security_engine.js";
import type {
	ThreatDetectionRequest,
	ThreatDetectionResponse,
	VulnerabilityScanRequest,
	VulnerabilityScanResponse,
	ThreatIntelligenceRequest,
	ThreatIntelligenceResponse,
} from "./types.js";

const PORT = 9009;

export class SecurityAIServer {
	private app: express.Application;
	private securityEngine: SecurityAIEngine;

	constructor() {
		this.app = express();
		this.securityEngine = new SecurityAIEngine();
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
			res.json({ status: "ok", service: "security-ai" });
		});

		// Detect threats
		this.app.post("/api/security/detect", async (req: Request, res: Response) => {
			try {
				const request: ThreatDetectionRequest = req.body;
				const response: ThreatDetectionResponse =
					await this.securityEngine.detectThreats(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Scan vulnerabilities
		this.app.post("/api/security/scan", async (req: Request, res: Response) => {
			try {
				const request: VulnerabilityScanRequest = req.body;
				const response: VulnerabilityScanResponse =
					await this.securityEngine.scanVulnerabilities(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Get threat intelligence
		this.app.post("/api/security/intelligence", async (req: Request, res: Response) => {
			try {
				const request: ThreatIntelligenceRequest = req.body;
				const response: ThreatIntelligenceResponse =
					await this.securityEngine.getThreatIntelligence(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`Security AI Service (security-ai) listening on port ${PORT}`);
		});
	}
}

