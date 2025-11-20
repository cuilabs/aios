/**
 * ML Bridge Service
 * 
 * IPC bridge between kernel and ML daemon (mld).
 * Receives IPC messages from kernel, translates to HTTP requests to ML daemon,
 * and returns responses via IPC.
 * 
 * This service runs in userland and provides the interface for kernel AI subsystems
 * to access ML predictions via the ML daemon HTTP API.
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import axios from "axios";

const PORT = 9006;
const MLD_URL = "http://127.0.0.1:9005";

/**
 * ML Bridge Server
 */
export class MLBridgeServer {
	private readonly app: express.Application;

	constructor() {
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		this.app.use(cors());
		this.app.use(express.json({ limit: "1mb" }));
	}

	private setupRoutes(): void {
		// Workload prediction
		this.app.post("/api/ml/predict/workload", this.handlePredictWorkload.bind(this));

		// Threat detection
		this.app.post("/api/ml/predict/threat", this.handlePredictThreat.bind(this));

		// Failure prediction
		this.app.post("/api/ml/predict/failure", this.handlePredictFailure.bind(this));

		// Memory prediction
		this.app.post("/api/ml/predict/memory", this.handlePredictMemory.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "ml-bridge" });
		});
	}

	/**
	 * Handle workload prediction
	 */
	private async handlePredictWorkload(req: Request, res: Response): Promise<void> {
		try {
			const kernelRequest = req.body;

			// Translate kernel request to ML daemon format
			const mldRequest = {
				agentId: `agent-${kernelRequest.agent_id}`,
				historicalCpu: kernelRequest.historical_cpu || [],
				historicalMemory: kernelRequest.historical_memory || [],
				historicalGpu: kernelRequest.historical_gpu || [],
				timeOfDay: kernelRequest.time_of_day || 12,
				dayOfWeek: kernelRequest.day_of_week || 1,
				currentCpu: kernelRequest.current_cpu || 0.5,
				currentMemory: kernelRequest.current_memory || 0,
				currentGpu: kernelRequest.current_gpu,
			};

			// Call ML daemon
			const response = await axios.post(`${MLD_URL}/api/ml/predict/workload`, mldRequest, {
				timeout: 5000,
			});

			// Translate ML daemon response to kernel format
			const kernelResponse = {
				success: response.data.success,
				prediction: {
					predicted_cpu: response.data.prediction?.predictedCpu || 0.5,
					predicted_memory: response.data.prediction?.predictedMemory || 0,
					predicted_gpu: response.data.prediction?.predictedGpu,
					confidence: response.data.prediction?.confidence || 0.5,
				},
			};

			res.json(kernelResponse);
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict workload",
			});
		}
	}

	/**
	 * Handle threat detection
	 */
	private async handlePredictThreat(req: Request, res: Response): Promise<void> {
		try {
			const kernelRequest = req.body;

			// Translate kernel request to ML daemon format
			const mldRequest = {
				agentId: `agent-${kernelRequest.agent_id}`,
				metrics: {
					operationCount: kernelRequest.metrics?.operation_count || 0,
					syscallCount: kernelRequest.metrics?.syscall_count || 0,
					memoryUsage: kernelRequest.metrics?.memory_usage || 0,
					networkActivity: kernelRequest.metrics?.network_activity || 0,
				},
				anomalies: (kernelRequest.anomalies || []).map((a: any) => ({
					anomalyType: a.anomaly_type || 0,
					severity: a.severity || 0.5,
					timestamp: a.timestamp || Date.now(),
				})),
				historicalThreats: kernelRequest.historical_threats || [],
				timeSinceLastThreat: kernelRequest.time_since_last_threat || 0,
			};

			// Call ML daemon
			const response = await axios.post(`${MLD_URL}/api/ml/predict/threat`, mldRequest, {
				timeout: 5000,
			});

			// Translate ML daemon response to kernel format
			const kernelResponse = {
				success: response.data.success,
				prediction: {
					threat_score: response.data.prediction?.threatScore || 0.0,
					threat_type: response.data.prediction?.threatType || 0,
					confidence: response.data.prediction?.confidence || 0.5,
					recommended_action: response.data.prediction?.recommendedAction || 0,
				},
			};

			res.json(kernelResponse);
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to detect threat",
			});
		}
	}

	/**
	 * Handle failure prediction
	 */
	private async handlePredictFailure(req: Request, res: Response): Promise<void> {
		try {
			const kernelRequest = req.body;

			// Translate kernel request to ML daemon format
			const mldRequest = {
				component: kernelRequest.component || "unknown",
				healthScore: kernelRequest.health_score || 0.5,
				currentValue: kernelRequest.current_value || 0.0,
				baseline: kernelRequest.baseline || 0.0,
				trend: kernelRequest.trend || 0,
				historicalHealth: kernelRequest.historical_health || [],
				failureHistory: kernelRequest.failure_history || [],
				timeSinceLastFailure: kernelRequest.time_since_last_failure || 0,
			};

			// Call ML daemon
			const response = await axios.post(`${MLD_URL}/api/ml/predict/failure`, mldRequest, {
				timeout: 5000,
			});

			// Translate ML daemon response to kernel format
			const kernelResponse = {
				success: response.data.success,
				prediction: {
					failure_probability: response.data.prediction?.failureProbability || 0.0,
					predicted_time: response.data.prediction?.predictedTime
						? response.data.prediction.predictedTime * 1_000_000 // Convert ms to ns
						: null,
					confidence: response.data.prediction?.confidence || 0.5,
					failure_type: response.data.prediction?.failureType || 0,
				},
			};

			res.json(kernelResponse);
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict failure",
			});
		}
	}

	/**
	 * Handle memory prediction
	 */
	private async handlePredictMemory(req: Request, res: Response): Promise<void> {
		try {
			const kernelRequest = req.body;

			// Translate kernel request to ML daemon format
			const mldRequest = {
				agentId: `agent-${kernelRequest.agent_id}`,
				accessHistory: kernelRequest.access_history || [],
				accessTypes: kernelRequest.access_types || [],
				accessTimestamps: kernelRequest.access_timestamps || [],
				currentAddress: kernelRequest.current_address || 0.0,
				localityScore: kernelRequest.locality_score || 0.5,
			};

			// Call ML daemon
			const response = await axios.post(`${MLD_URL}/api/ml/predict/memory`, mldRequest, {
				timeout: 5000,
			});

			// Translate ML daemon response to kernel format
			const kernelResponse = {
				success: response.data.success,
				prediction: {
					next_address: response.data.prediction?.nextAddress || 0.0,
					access_probability: response.data.prediction?.accessProbability || 0.0,
					access_type: response.data.prediction?.accessType || 0,
					confidence: response.data.prediction?.confidence || 0.5,
				},
			};

			res.json(kernelResponse);
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to predict memory access",
			});
		}
	}

	/**
	 * Start server
	 */
	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			const server = this.app.listen(PORT, () => {
				console.log(`ML Bridge Service listening on port ${PORT}`);
				resolve();
			});

			server.on("error", (err: Error) => {
				console.error("Failed to start ML Bridge Service:", err);
				reject(err);
			});
		});
	}

	/**
	 * Stop server
	 */
	async stop(): Promise<void> {
		// Implementation for graceful shutdown
	}
}

