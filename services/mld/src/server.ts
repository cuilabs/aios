/**
 * ML Daemon HTTP Server
 *
 * Production-grade HTTP REST API server for ML inference service.
 * Exposes optimized ML prediction endpoints for kernel AI subsystems.
 */

import { getInferenceEngine } from "@aios/ml";
import cors from "cors";
import express, { type Request, type Response } from "express";
import type { FailureFeatures, MemoryFeatures, ThreatFeatures, WorkloadFeatures } from "./types.js";

const PORT = 9005;

export class MLDaemonServer {
	private readonly app: express.Application;
	private readonly inferenceEngine = getInferenceEngine();
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private server: ReturnType<express.Application["listen"]> | null = null;

	constructor() {
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		// CORS for localhost development
		this.app.use(
			cors({
				origin: ["http://localhost:9005", "http://127.0.0.1:9005"],
				credentials: true,
			})
		);

		// JSON body parser (optimized for speed)
		this.app.use(express.json({ limit: "1mb" }));

		// Request logging (minimal for performance)
		this.app.use((req: Request, _res: Response, next) => {
			if (process.env["VERBOSE"] === "1") {
				console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
			}
			next();
		});

		// Error handler
		this.app.use((err: Error, _req: Request, res: Response, _next: () => void) => {
			console.error("ML Daemon Error:", err);
			res.status(500).json({
				success: false,
				error: err.message || "Internal server error",
			});
		});
	}

	private setupRoutes(): void {
		// Workload prediction
		this.app.post("/api/ml/predict/workload", this.handlePredictWorkload.bind(this));
		this.app.post("/api/ml/predict/workload/batch", this.handleBatchPredictWorkload.bind(this));

		// Threat detection
		this.app.post("/api/ml/predict/threat", this.handlePredictThreat.bind(this));
		this.app.post("/api/ml/predict/threat/batch", this.handleBatchPredictThreat.bind(this));

		// Failure prediction
		this.app.post("/api/ml/predict/failure", this.handlePredictFailure.bind(this));

		// Memory prediction
		this.app.post("/api/ml/predict/memory", this.handlePredictMemory.bind(this));

		// Metrics
		this.app.get("/api/ml/metrics", this.handleGetMetrics.bind(this));
		this.app.get("/api/ml/metrics/:model", this.handleGetModelMetrics.bind(this));
		this.app.post("/api/ml/metrics/reset", this.handleResetMetrics.bind(this));

		// Cache management
		this.app.get("/api/ml/cache/stats", this.handleGetCacheStats.bind(this));
		this.app.post("/api/ml/cache/clear", this.handleClearCache.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "mld" });
		});
	}

	/**
	 * Handle workload prediction
	 */
	private async handlePredictWorkload(req: Request, res: Response): Promise<void> {
		try {
			const features = req.body as WorkloadFeatures;

			if (!features || !features.agentId) {
				res.status(400).json({
					success: false,
					error: "Invalid workload features: agentId required",
				});
				return;
			}

			const startTime = performance.now();
			const prediction = await this.inferenceEngine.predictWorkload(features);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				prediction,
				latency_ms: latency,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict workload",
			});
		}
	}

	/**
	 * Handle batch workload prediction
	 */
	private async handleBatchPredictWorkload(req: Request, res: Response): Promise<void> {
		try {
			const featuresList = req.body.features as WorkloadFeatures[];

			if (!Array.isArray(featuresList) || featuresList.length === 0) {
				res.status(400).json({
					success: false,
					error: "Invalid features: expected array of workload features",
				});
				return;
			}

			const startTime = performance.now();
			const predictions = await this.inferenceEngine.batchPredictWorkload(featuresList);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				predictions,
				latency_ms: latency,
				count: predictions.length,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to batch predict workload",
			});
		}
	}

	/**
	 * Handle threat detection
	 */
	private async handlePredictThreat(req: Request, res: Response): Promise<void> {
		try {
			const features = req.body as ThreatFeatures;

			if (!features || !features.agentId) {
				res.status(400).json({
					success: false,
					error: "Invalid threat features: agentId required",
				});
				return;
			}

			const startTime = performance.now();
			const prediction = await this.inferenceEngine.predictThreat(features);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				prediction,
				latency_ms: latency,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to detect threat",
			});
		}
	}

	/**
	 * Handle batch threat detection
	 */
	private async handleBatchPredictThreat(req: Request, res: Response): Promise<void> {
		try {
			const featuresList = req.body.features as ThreatFeatures[];

			if (!Array.isArray(featuresList) || featuresList.length === 0) {
				res.status(400).json({
					success: false,
					error: "Invalid features: expected array of threat features",
				});
				return;
			}

			const startTime = performance.now();
			const predictions = await this.inferenceEngine.batchPredictThreat(featuresList);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				predictions,
				latency_ms: latency,
				count: predictions.length,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to batch detect threats",
			});
		}
	}

	/**
	 * Handle failure prediction
	 */
	private async handlePredictFailure(req: Request, res: Response): Promise<void> {
		try {
			const features = req.body as FailureFeatures;

			if (!features || !features.component) {
				res.status(400).json({
					success: false,
					error: "Invalid failure features: component required",
				});
				return;
			}

			const startTime = performance.now();
			const prediction = await this.inferenceEngine.predictFailure(features);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				prediction,
				latency_ms: latency,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict failure",
			});
		}
	}

	/**
	 * Handle memory access prediction
	 */
	private async handlePredictMemory(req: Request, res: Response): Promise<void> {
		try {
			const features = req.body as MemoryFeatures;

			if (!features || !features.agentId) {
				res.status(400).json({
					success: false,
					error: "Invalid memory features: agentId required",
				});
				return;
			}

			const startTime = performance.now();
			const prediction = await this.inferenceEngine.predictMemory(features);
			const latency = performance.now() - startTime;

			res.json({
				success: true,
				prediction,
				latency_ms: latency,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict memory access",
			});
		}
	}

	/**
	 * Get all metrics
	 */
	private handleGetMetrics(_req: Request, res: Response): void {
		try {
			const metrics = this.inferenceEngine.getMetrics();
			const metricsMap = metrics instanceof Map ? Object.fromEntries(metrics) : metrics;

			res.json({
				success: true,
				metrics: metricsMap,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get metrics",
			});
		}
	}

	/**
	 * Get metrics for specific model
	 */
	private handleGetModelMetrics(req: Request, res: Response): void {
		try {
			const modelName = req.params["model"] as string;
			const metrics = this.inferenceEngine.getMetrics(modelName);

			if (!metrics) {
				res.status(404).json({
					success: false,
					error: `Model not found: ${modelName}`,
				});
				return;
			}

			res.json({
				success: true,
				metrics: metrics instanceof Map ? Object.fromEntries(metrics) : metrics,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get model metrics",
			});
		}
	}

	/**
	 * Reset metrics
	 */
	private handleResetMetrics(req: Request, res: Response): void {
		try {
			const body = req.body as { model?: string };
			const modelName = body["model"];

			this.inferenceEngine.resetMetrics(modelName);

			res.json({
				success: true,
				message: modelName ? `Metrics reset for ${modelName}` : "All metrics reset",
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to reset metrics",
			});
		}
	}

	/**
	 * Get cache statistics
	 */
	private handleGetCacheStats(_req: Request, res: Response): void {
		try {
			const stats = this.inferenceEngine.getCacheStats();

			res.json({
				success: true,
				cache: stats,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get cache stats",
			});
		}
	}

	/**
	 * Clear caches
	 */
	private handleClearCache(_req: Request, res: Response): void {
		try {
			this.inferenceEngine.clearCaches();

			res.json({
				success: true,
				message: "All caches cleared",
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to clear caches",
			});
		}
	}

	/**
	 * Start server
	 */
	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			const server = this.app.listen(PORT, () => {
				console.log(`ML Daemon Service listening on port ${PORT}`);
				this.server = server;
				resolve();
			});

			server.on("error", (err: Error) => {
				console.error("Failed to start ML Daemon Service:", err);
				reject(err);
			});
		});
	}

	/**
	 * Stop server
	 */
	async stop(): Promise<void> {
		return new Promise((resolve) => {
			if (this.server) {
				this.server.close(() => {
					console.log("ML Daemon Service stopped");
					this.server = null;
					resolve();
				});
			} else {
				resolve();
			}
		});
	}
}
