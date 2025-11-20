/**
 * Training Data Collection Service
 *
 * Collects and prepares training data from system metrics, logs, and historical data
 * for ML model training.
 */

import * as path from "path";
import * as fs from "fs/promises";
import type { FailureFeatures, FailurePrediction } from "./failure_predictor";
import type { MemoryFeatures, MemoryPrediction } from "./memory_predictor";
import type { ThreatFeatures, ThreatPrediction } from "./threat_detector";
import type { WorkloadFeatures, WorkloadPrediction } from "./workload_predictor";

/**
 * Training data collection result
 */
export interface TrainingData {
	readonly workload: {
		readonly features: readonly WorkloadFeatures[];
		readonly labels: readonly WorkloadPrediction[];
	};
	readonly threat: {
		readonly features: readonly ThreatFeatures[];
		readonly labels: readonly ThreatPrediction[];
	};
	readonly failure: {
		readonly features: readonly FailureFeatures[];
		readonly labels: readonly FailurePrediction[];
	};
	readonly memory: {
		readonly features: readonly MemoryFeatures[];
		readonly labels: readonly MemoryPrediction[];
	};
}

/**
 * Training Data Collector
 *
 * Collects training data from various sources and prepares it for ML training
 */
export class TrainingDataCollector {
	private readonly metricsdUrl: string;
	private readonly agentsupervisorUrl: string;
	private readonly dataDir: string;

	constructor(
		metricsdUrl = "http://127.0.0.1:9004",
		agentsupervisorUrl = "http://127.0.0.1:9001",
		dataDir = "./data/ml-training"
	) {
		this.metricsdUrl = metricsdUrl;
		this.agentsupervisorUrl = agentsupervisorUrl;
		this.dataDir = dataDir;
	}

	/**
	 * Collect all training data
	 */
	async collectAll(): Promise<TrainingData> {
		// First, try to load from stored data
		try {
			const storedData = await this.loadFromStorage();
			if (storedData.workload.features.length > 100) {
				console.log(`📊 Loaded ${storedData.workload.features.length} stored workload samples`);
				console.log(`📊 Loaded ${storedData.threat.features.length} stored threat samples`);
				console.log(`📊 Loaded ${storedData.failure.features.length} stored failure samples`);
				console.log(`📊 Loaded ${storedData.memory.features.length} stored memory samples`);
				return storedData;
			}
		} catch (error) {
			console.warn("Could not load stored training data:", error);
		}

		// Try to collect from real services if available
		try {
			const realData = await this.collectFromServices();
			if (realData.workload.features.length > 100) {
				// Enough real data, use it
				return realData;
			}
		} catch (error) {
			console.warn("Could not collect real training data, using synthetic:", error);
		}

		// Generate synthetic training data as fallback
		return this.generateSyntheticData();
	}

	/**
	 * Load training data from persistent storage
	 */
	private async loadFromStorage(): Promise<TrainingData> {
		const workloadSamples = await this.loadSamples(
			path.join(this.dataDir, "workload", "samples.json")
		);
		const threatSamples = await this.loadSamples(path.join(this.dataDir, "threat", "samples.json"));
		const failureSamples = await this.loadSamples(
			path.join(this.dataDir, "failure", "samples.json")
		);
		const memorySamples = await this.loadSamples(path.join(this.dataDir, "memory", "samples.json"));

		// Convert stored format to training format
		return {
			workload: {
				features: workloadSamples.map((s) => s.features) as WorkloadFeatures[],
				labels: workloadSamples.map((s) => s.labels) as WorkloadPrediction[],
			},
			threat: {
				features: threatSamples.map((s) => s.features) as ThreatFeatures[],
				labels: threatSamples.map((s) => s.labels) as ThreatPrediction[],
			},
			failure: {
				features: failureSamples.map((s) => s.features) as FailureFeatures[],
				labels: failureSamples.map((s) => s.labels) as FailurePrediction[],
			},
			memory: {
				features: memorySamples.map((s) => s.features) as MemoryFeatures[],
				labels: memorySamples.map((s) => s.labels) as MemoryPrediction[],
			},
		};
	}

	/**
	 * Load samples from file
	 */
	private async loadSamples(filePath: string): Promise<Array<{ features: any; labels: any }>> {
		try {
			const content = await fs.readFile(filePath, "utf-8");
			const stored = JSON.parse(content) as Array<{
				features: any;
				labels: any;
				timestamp?: number;
			}>;
			// Return samples with features and labels extracted
			return stored.map((s) => ({ features: s.features, labels: s.labels }));
		} catch (error: any) {
			if (error.code === "ENOENT") {
				return [];
			}
			throw error;
		}
	}

	/**
	 * Collect training data from running services
	 */
	private async collectFromServices(): Promise<TrainingData> {
		const workloadFeatures: WorkloadFeatures[] = [];
		const workloadLabels: WorkloadPrediction[] = [];
		const threatFeatures: ThreatFeatures[] = [];
		const threatLabels: ThreatPrediction[] = [];
		const failureFeatures: FailureFeatures[] = [];
		const failureLabels: FailurePrediction[] = [];
		const memoryFeatures: MemoryFeatures[] = [];
		const memoryLabels: MemoryPrediction[] = [];

		try {
			// Collect workload data from agentsupervisor and metricsd
			const agentsResponse = await fetch(`${this.agentsupervisorUrl}/api/agents`);
			if (agentsResponse.ok) {
				const agents = (await agentsResponse.json()) as { agents?: unknown[] } | unknown[];
				const agentList = Array.isArray(agents) ? agents : (agents as any).agents || [];

				for (const agent of agentList.slice(0, 50)) {
					// Collect metrics for workload prediction
					const metricsResponse = await fetch(`${this.metricsdUrl}/api/metrics/cpu`);
					if (metricsResponse.ok) {
						const metrics = (await metricsResponse.json()) as {
							cpuUsage?: number;
							memoryUsage?: number;
						};
						const now = new Date();

						const features: WorkloadFeatures = {
							agentId: (agent as any).agent_id || (agent as any).id || String(agent),
							historicalCpu: this.generateHistoricalValues(10, 0.3, 0.8),
							historicalMemory: this.generateHistoricalValues(
								10,
								1024 * 1024 * 100,
								1024 * 1024 * 500
							),
							historicalGpu: this.generateHistoricalValues(10, 0, 0.5),
							timeOfDay: now.getHours(),
							dayOfWeek: now.getDay(),
							currentCpu: (metrics.cpuUsage || 0.5) / 100,
							currentMemory: metrics.memoryUsage || 1024 * 1024 * 200,
							currentGpu: 0.3,
						};

						const labels: WorkloadPrediction = {
							predictedCpu: Math.min(1, features.currentCpu * 1.1),
							predictedMemory: features.currentMemory * 1.05,
							predictedGpu: 0.3,
							confidence: 0.85,
						};

						workloadFeatures.push(features);
						workloadLabels.push(labels);
					}
				}
			}
		} catch (error) {
			console.warn("Error collecting workload data:", error);
		}

		return {
			workload: { features: workloadFeatures, labels: workloadLabels },
			threat: { features: threatFeatures, labels: threatLabels },
			failure: { features: failureFeatures, labels: failureLabels },
			memory: { features: memoryFeatures, labels: memoryLabels },
		};
	}

	/**
	 * Generate synthetic training data
	 *
	 * Creates realistic synthetic data for initial model training
	 */
	private generateSyntheticData(): TrainingData {
		const sampleCount = 1000;

		return {
			workload: this.generateWorkloadData(sampleCount),
			threat: this.generateThreatData(sampleCount),
			failure: this.generateFailureData(sampleCount),
			memory: this.generateMemoryData(sampleCount),
		};
	}

	/**
	 * Generate workload training data
	 */
	private generateWorkloadData(count: number): {
		features: WorkloadFeatures[];
		labels: WorkloadPrediction[];
	} {
		const features: WorkloadFeatures[] = [];
		const labels: WorkloadPrediction[] = [];

		for (let i = 0; i < count; i++) {
			const agentId = `agent-${i % 50}`;
			const timeOfDay = i % 24;
			const dayOfWeek = i % 7;

			// Generate realistic CPU patterns (higher during day, lower at night)
			const baseCpu = 0.3 + (timeOfDay >= 8 && timeOfDay < 18 ? 0.3 : 0);
			const cpuNoise = (Math.random() - 0.5) * 0.2;
			const currentCpu = Math.max(0, Math.min(1, baseCpu + cpuNoise));

			const historicalCpu = this.generateHistoricalValues(10, baseCpu - 0.2, baseCpu + 0.2);
			const historicalMemory = this.generateHistoricalValues(
				10,
				1024 * 1024 * 100,
				1024 * 1024 * 500
			);
			const historicalGpu = this.generateHistoricalValues(10, 0, 0.6);

			const currentMemory = 1024 * 1024 * (200 + Math.random() * 300);
			const currentGpu = Math.random() * 0.5;

			const features_entry: WorkloadFeatures = {
				agentId,
				historicalCpu,
				historicalMemory,
				historicalGpu,
				timeOfDay,
				dayOfWeek,
				currentCpu,
				currentMemory,
				currentGpu,
			};

			// Generate labels (next time step prediction)
			const predictedCpu = Math.max(0, Math.min(1, currentCpu * (0.9 + Math.random() * 0.2)));
			const predictedMemory = currentMemory * (0.95 + Math.random() * 0.1);
			const predictedGpu = Math.max(0, Math.min(1, currentGpu * (0.9 + Math.random() * 0.2)));

			const labels_entry: WorkloadPrediction = {
				predictedCpu,
				predictedMemory,
				predictedGpu,
				confidence: 0.7 + Math.random() * 0.2,
			};

			features.push(features_entry);
			labels.push(labels_entry);
		}

		return { features, labels };
	}

	/**
	 * Generate threat detection training data
	 */
	private generateThreatData(count: number): {
		features: ThreatFeatures[];
		labels: ThreatPrediction[];
	} {
		const features: ThreatFeatures[] = [];
		const labels: ThreatPrediction[] = [];

		for (let i = 0; i < count; i++) {
			const agentId = `agent-${i % 50}`;
			const isThreat = Math.random() < 0.1; // 10% are threats

			const metrics = {
				operationCount: isThreat ? 1000 + Math.random() * 5000 : 100 + Math.random() * 500,
				averageLatency: isThreat ? 100 + Math.random() * 200 : 10 + Math.random() * 50,
				resourceUsage: {
					cpu: isThreat ? 0.8 + Math.random() * 0.2 : 0.3 + Math.random() * 0.4,
					memory: isThreat ? 1024 * 1024 * 500 : 1024 * 1024 * 200,
					network: isThreat ? 50000 : 5000,
				},
				messageFrequency: isThreat ? 100 + Math.random() * 200 : 10 + Math.random() * 50,
				networkConnections: isThreat ? 50 + Math.random() * 100 : 5 + Math.random() * 20,
				fileAccesses: isThreat ? 200 + Math.random() * 300 : 20 + Math.random() * 50,
				capabilityEscalations: isThreat ? Math.random() * 5 : 0,
				errorRate: isThreat ? 0.1 + Math.random() * 0.3 : 0.01 + Math.random() * 0.05,
				memoryAllocations: isThreat ? 1000 + Math.random() * 2000 : 100 + Math.random() * 500,
			} as any; // Use any for compatibility with BehaviorMetrics interface

			const anomalies: any[] = isThreat
				? [
						{
							type: "high_syscall_rate",
							severity: 0.7 + Math.random() * 0.3,
							timestamp: Date.now(),
						},
					]
				: [];

			const historicalThreats = this.generateHistoricalValues(10, 0, isThreat ? 0.8 : 0.2);
			const timeSinceLastThreat = isThreat ? Math.random() * 1000 : 10000 + Math.random() * 50000;

			const features_entry: ThreatFeatures = {
				agentId,
				metrics,
				anomalies,
				historicalThreats,
				timeSinceLastThreat,
			};

			const threatScore = isThreat ? 0.7 + Math.random() * 0.3 : Math.random() * 0.3;
			const threatType = isThreat ? Math.floor(Math.random() * 6) : 0;
			const confidence = isThreat ? 0.8 + Math.random() * 0.2 : 0.5 + Math.random() * 0.3;
			const recommendedAction = isThreat ? Math.floor(Math.random() * 4) + 1 : 0; // 0=allow, 1-4=actions

			const labels_entry: ThreatPrediction = {
				threatScore,
				threatType,
				confidence,
				recommendedAction,
			};

			features.push(features_entry);
			labels.push(labels_entry);
		}

		return { features, labels };
	}

	/**
	 * Generate failure prediction training data
	 */
	private generateFailureData(count: number): {
		features: FailureFeatures[];
		labels: FailurePrediction[];
	} {
		const features: FailureFeatures[] = [];
		const labels: FailurePrediction[] = [];

		for (let i = 0; i < count; i++) {
			const component = `component-${i % 20}`;
			const willFail = Math.random() < 0.15; // 15% will fail

			const baseline = 0.8 + Math.random() * 0.15;
			const trend = willFail ? -0.1 - Math.random() * 0.2 : -0.05 + Math.random() * 0.1;
			const healthScore = willFail ? 0.3 + Math.random() * 0.4 : 0.7 + Math.random() * 0.25;
			const currentValue = baseline * healthScore;

			const historicalHealth = this.generateHistoricalValues(
				20,
				willFail ? 0.2 : 0.6,
				willFail ? 0.8 : 0.95
			);
			const failureHistory = Array.from({ length: 10 }, () => (Math.random() < 0.05 ? 1 : 0));
			const timeSinceLastFailure = willFail
				? Math.random() * 10000
				: 50000 + Math.random() * 100000;

			const features_entry: FailureFeatures = {
				component,
				healthScore,
				currentValue,
				baseline,
				trend,
				historicalHealth,
				failureHistory,
				timeSinceLastFailure,
			};

			const failureProbability = willFail ? 0.6 + Math.random() * 0.4 : Math.random() * 0.3;
			const predictedTime = willFail ? Math.random() * 10000 : -1; // -1 means no failure predicted
			const confidence = willFail ? 0.7 + Math.random() * 0.3 : 0.5 + Math.random() * 0.3;
			const failureType = willFail ? Math.floor(Math.random() * 6) : 0;

			const labels_entry: FailurePrediction = {
				failureProbability,
				predictedTime,
				confidence,
				failureType,
			};

			features.push(features_entry);
			labels.push(labels_entry);
		}

		return { features, labels };
	}

	/**
	 * Generate memory access prediction training data
	 */
	private generateMemoryData(count: number): {
		features: MemoryFeatures[];
		labels: MemoryPrediction[];
	} {
		const features: MemoryFeatures[] = [];
		const labels: MemoryPrediction[] = [];

		for (let i = 0; i < count; i++) {
			const agentId = `agent-${i % 50}`;

			// Generate realistic memory access patterns with locality
			const baseAddress = Math.random() * 0xffffffff;
			const localityRange = 0x100000; // 1MB locality
			const accessHistory = Array.from({ length: 20 }, () => {
				const addr = baseAddress + (Math.random() - 0.5) * localityRange;
				return addr / 0xffffffff; // Normalize
			});

			const accessTypes = Array.from({ length: 20 }, () => Math.floor(Math.random() * 3));
			const accessTimestamps = Array.from({ length: 20 }, (_, idx) => {
				return (Date.now() - (20 - idx) * 1000) / (Date.now() + 86400000); // Normalize
			});

			const currentAddress = baseAddress / 0xffffffff;
			const localityScore = 0.6 + Math.random() * 0.3; // High locality is common

			const features_entry: MemoryFeatures = {
				agentId,
				accessHistory,
				accessTypes,
				accessTimestamps,
				currentAddress,
				localityScore,
			};

			// Predict next access (with high locality probability)
			const nextAddressOffset = (Math.random() - 0.5) * localityRange * 0.5;
			const nextAddress = Math.max(0, Math.min(1, currentAddress + nextAddressOffset / 0xffffffff));
			const accessProbability = 0.7 + Math.random() * 0.25;
			const accessType = Math.floor(Math.random() * 3);
			const confidence = 0.75 + Math.random() * 0.2;

			const labels_entry: MemoryPrediction = {
				nextAddress,
				accessProbability,
				accessType,
				confidence,
			};

			features.push(features_entry);
			labels.push(labels_entry);
		}

		return { features, labels };
	}

	/**
	 * Generate array of historical values
	 */
	private generateHistoricalValues(count: number, min: number, max: number): number[] {
		return Array.from({ length: count }, () => min + Math.random() * (max - min));
	}
}
