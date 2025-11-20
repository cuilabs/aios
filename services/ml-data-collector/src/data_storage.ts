/**
 * Persistent Data Storage for ML Training Data
 * 
 * Stores collected training data in JSON files for later use in model training
 */

import * as fs from "fs/promises";
import * as path from "path";

export interface StoredWorkloadSample {
	readonly timestamp: number;
	readonly agentId: string;
	readonly features: {
		readonly historicalCpu: readonly number[];
		readonly historicalMemory: readonly number[];
		readonly historicalGpu: readonly number[];
		readonly timeOfDay: number;
		readonly dayOfWeek: number;
		readonly currentCpu: number;
		readonly currentMemory: number;
		readonly currentGpu: number;
	};
	readonly labels: {
		readonly predictedCpu: number;
		readonly predictedMemory: number;
		readonly predictedGpu: number;
		readonly confidence: number;
	};
}

export interface StoredThreatSample {
	readonly timestamp: number;
	readonly agentId: string;
	readonly features: {
		readonly metrics: {
			readonly operationCount: number;
			readonly averageLatency: number;
			readonly resourceUsage: {
				readonly cpu: number;
				readonly memory: number;
				readonly network: number;
			};
			readonly messageFrequency: number;
			readonly networkConnections: number;
			readonly fileAccesses: number;
			readonly capabilityEscalations: number;
			readonly errorRate: number;
			readonly memoryAllocations: number;
		};
		readonly anomalies: Array<{
			readonly type: string;
			readonly severity: number;
			readonly timestamp: number;
		}>;
		readonly historicalThreats: readonly number[];
		readonly timeSinceLastThreat: number;
	};
	readonly labels: {
		readonly threatScore: number;
		readonly threatType: number;
		readonly confidence: number;
		readonly recommendedAction: number;
	};
}

export interface StoredFailureSample {
	readonly timestamp: number;
	readonly component: string;
	readonly features: {
		readonly healthScore: number;
		readonly currentValue: number;
		readonly baseline: number;
		readonly trend: number;
		readonly historicalHealth: readonly number[];
		readonly failureHistory: readonly number[];
		readonly timeSinceLastFailure: number;
	};
	readonly labels: {
		readonly failureProbability: number;
		readonly predictedTime: number;
		readonly confidence: number;
		readonly failureType: number;
	};
}

export interface StoredMemorySample {
	readonly timestamp: number;
	readonly agentId: string;
	readonly features: {
		readonly accessHistory: readonly number[];
		readonly accessTypes: readonly number[];
		readonly accessTimestamps: readonly number[];
		readonly currentAddress: number;
		readonly localityScore: number;
	};
	readonly labels: {
		readonly nextAddress: number;
		readonly accessProbability: number;
		readonly accessType: number;
		readonly confidence: number;
	};
}

export interface StoredTrainingData {
	readonly workload: readonly StoredWorkloadSample[];
	readonly threat: readonly StoredThreatSample[];
	readonly failure: readonly StoredFailureSample[];
	readonly memory: readonly StoredMemorySample[];
}

/**
 * Data Storage Manager
 * 
 * Manages persistent storage of training data in JSON files
 */
export class DataStorageManager {
	private readonly dataDir: string;
	private readonly maxSamplesPerType: number;

	constructor(dataDir = "./data/ml-training", maxSamplesPerType = 100000) {
		this.dataDir = dataDir;
		this.maxSamplesPerType = maxSamplesPerType;
	}

	/**
	 * Initialize storage directories
	 */
	async initialize(): Promise<void> {
		await fs.mkdir(this.dataDir, { recursive: true });
		await fs.mkdir(path.join(this.dataDir, "workload"), { recursive: true });
		await fs.mkdir(path.join(this.dataDir, "threat"), { recursive: true });
		await fs.mkdir(path.join(this.dataDir, "failure"), { recursive: true });
		await fs.mkdir(path.join(this.dataDir, "memory"), { recursive: true });
	}

	/**
	 * Store workload sample
	 */
	async storeWorkloadSample(sample: StoredWorkloadSample): Promise<void> {
		const filePath = path.join(this.dataDir, "workload", "samples.json");
		const samples = await this.loadSamples<StoredWorkloadSample>(filePath);
		samples.push(sample);
		
		// Keep only the most recent samples
		const trimmed = samples.slice(-this.maxSamplesPerType);
		await fs.writeFile(filePath, JSON.stringify(trimmed, null, 2), "utf-8");
	}

	/**
	 * Store threat sample
	 */
	async storeThreatSample(sample: StoredThreatSample): Promise<void> {
		const filePath = path.join(this.dataDir, "threat", "samples.json");
		const samples = await this.loadSamples<StoredThreatSample>(filePath);
		samples.push(sample);
		
		const trimmed = samples.slice(-this.maxSamplesPerType);
		await fs.writeFile(filePath, JSON.stringify(trimmed, null, 2), "utf-8");
	}

	/**
	 * Store failure sample
	 */
	async storeFailureSample(sample: StoredFailureSample): Promise<void> {
		const filePath = path.join(this.dataDir, "failure", "samples.json");
		const samples = await this.loadSamples<StoredFailureSample>(filePath);
		samples.push(sample);
		
		const trimmed = samples.slice(-this.maxSamplesPerType);
		await fs.writeFile(filePath, JSON.stringify(trimmed, null, 2), "utf-8");
	}

	/**
	 * Store memory sample
	 */
	async storeMemorySample(sample: StoredMemorySample): Promise<void> {
		const filePath = path.join(this.dataDir, "memory", "samples.json");
		const samples = await this.loadSamples<StoredMemorySample>(filePath);
		samples.push(sample);
		
		const trimmed = samples.slice(-this.maxSamplesPerType);
		await fs.writeFile(filePath, JSON.stringify(trimmed, null, 2), "utf-8");
	}

	/**
	 * Load all stored training data
	 */
	async loadAllData(): Promise<StoredTrainingData> {
		const workload = await this.loadSamples<StoredWorkloadSample>(
			path.join(this.dataDir, "workload", "samples.json")
		);
		const threat = await this.loadSamples<StoredThreatSample>(
			path.join(this.dataDir, "threat", "samples.json")
		);
		const failure = await this.loadSamples<StoredFailureSample>(
			path.join(this.dataDir, "failure", "samples.json")
		);
		const memory = await this.loadSamples<StoredMemorySample>(
			path.join(this.dataDir, "memory", "samples.json")
		);

		return { workload, threat, failure, memory };
	}

	/**
	 * Get statistics about stored data
	 */
	async getStatistics(): Promise<{
		readonly workload: number;
		readonly threat: number;
		readonly failure: number;
		readonly memory: number;
		readonly oldestTimestamp: number;
		readonly newestTimestamp: number;
	}> {
		try {
			// Read file sizes first to avoid loading huge files
			const getCount = async (filePath: string): Promise<number> => {
				try {
					const content = await fs.readFile(filePath, "utf-8");
					const data = JSON.parse(content);
					return Array.isArray(data) ? data.length : 0;
				} catch {
					return 0;
				}
			};

			const workloadCount = await getCount(path.join(this.dataDir, "workload", "samples.json"));
			const threatCount = await getCount(path.join(this.dataDir, "threat", "samples.json"));
			const failureCount = await getCount(path.join(this.dataDir, "failure", "samples.json"));
			const memoryCount = await getCount(path.join(this.dataDir, "memory", "samples.json"));

			// Get timestamps only if we have samples (quick check)
			let oldestTimestamp = 0;
			let newestTimestamp = 0;

			if (workloadCount > 0 || threatCount > 0 || failureCount > 0 || memoryCount > 0) {
				const data = await this.loadAllData();
				const allTimestamps = [
					...data.workload.map(s => s.timestamp),
					...data.threat.map(s => s.timestamp),
					...data.failure.map(s => s.timestamp),
					...data.memory.map(s => s.timestamp),
				].filter(t => t > 0);

				if (allTimestamps.length > 0) {
					oldestTimestamp = Math.min(...allTimestamps);
					newestTimestamp = Math.max(...allTimestamps);
				}
			}

			return {
				workload: workloadCount,
				threat: threatCount,
				failure: failureCount,
				memory: memoryCount,
				oldestTimestamp,
				newestTimestamp,
			};
		} catch (error) {
			// Return zeros on error to prevent hanging
			console.error("Error getting statistics:", error);
			return {
				workload: 0,
				threat: 0,
				failure: 0,
				memory: 0,
				oldestTimestamp: 0,
				newestTimestamp: 0,
			};
		}
	}

	/**
	 * Load samples from file
	 */
	private async loadSamples<T>(filePath: string): Promise<T[]> {
		try {
			const content = await fs.readFile(filePath, "utf-8");
			return JSON.parse(content) as T[];
		} catch (error: any) {
			if (error.code === "ENOENT") {
				return [];
			}
			throw error;
		}
	}
}

