/**
 * Threat Detection ML Model
 *
 * Uses TensorFlow.js to detect security threats based on agent behavior
 */

// Import tf dynamically to support both Node.js and browser
import { getMLModelManager } from "./index";

// Get tf from index (which handles Node.js vs browser)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let tf: any;
try {
	tf = require("@tensorflow/tfjs-node");
} catch {
	try {
		tf = require("@tensorflow/tfjs");
	} catch {
		tf = null;
	}
}
import type { BehaviorMetrics, BehavioralAnomaly } from "@aios/security";

/**
 * Threat detection input features
 */
export interface ThreatFeatures {
	readonly agentId: string;
	readonly metrics: BehaviorMetrics;
	readonly anomalies: readonly BehavioralAnomaly[];
	readonly historicalThreats: readonly number[]; // Last 10 threat scores
	readonly timeSinceLastThreat: number; // Milliseconds
}

/**
 * Threat detection output
 */
export interface ThreatPrediction {
	readonly threatScore: number; // 0.0 to 1.0
	readonly threatType: number; // 0-5 (enum index)
	readonly confidence: number; // 0.0 to 1.0
	readonly recommendedAction: number; // 0-4 (enum index)
}

/**
 * Threat Detector ML Model
 */
export class ThreatDetectorModel {
	private model: any | null = null;
	private readonly modelManager = getMLModelManager();

	/**
	 * Initialize model
	 */
	async initialize(): Promise<void> {
		this.model = await this.modelManager.loadModel("threat_detector");
	}

	/**
	 * Predict threat
	 */
	async predict(features: ThreatFeatures): Promise<ThreatPrediction> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare input tensor
		const input = this.prepareInput(features);

		// Run prediction
		const prediction = this.model!.predict(input) as any;

		// Extract predictions
		const values = await prediction.data();
		const threatScore = Math.max(0, Math.min(1, values[0]));
		const threatType = Math.round(Math.max(0, Math.min(5, values[1])));
		const confidence = Math.max(0, Math.min(1, values[2]));
		const recommendedAction = Math.round(Math.max(0, Math.min(4, values[3])));

		// Cleanup
		input.dispose();
		prediction.dispose();

		return {
			threatScore,
			threatType,
			confidence,
			recommendedAction,
		};
	}

	/**
	 * Prepare input tensor from features
	 */
	private prepareInput(features: ThreatFeatures): any {
		const inputArray: number[] = [];

		// Behavior metrics
		inputArray.push(features.metrics.averageLatency / 1000); // Normalize to seconds
		inputArray.push(features.metrics.errorRate);
		inputArray.push(features.metrics.resourceUsage);
		inputArray.push(features.metrics.messageFrequency / 1000); // Normalize

		// Anomaly features
		const anomalyCount = features.anomalies.length;
		const criticalAnomalies = features.anomalies.filter((a) => a.severity === "critical").length;
		const highAnomalies = features.anomalies.filter((a) => a.severity === "high").length;

		inputArray.push(anomalyCount / 10); // Normalize
		inputArray.push(criticalAnomalies / 10);
		inputArray.push(highAnomalies / 10);

		// Historical threats (10 values)
		for (let i = 0; i < 10; i++) {
			inputArray.push(features.historicalThreats[i] ?? 0);
		}

		// Time since last threat (normalized to hours)
		inputArray.push(features.timeSinceLastThreat / (1000 * 60 * 60));

		// Create tensor
		return tf.tensor2d([inputArray], [1, inputArray.length]);
	}

	/**
	 * Prepare input array (without tensor) for batch training
	 * Must match prepareInput() exactly
	 */
	private prepareInputArray(features: ThreatFeatures): number[] {
		const inputArray: number[] = [];

		// Behavior metrics
		inputArray.push(features.metrics.averageLatency / 1000); // Normalize to seconds
		inputArray.push(features.metrics.errorRate);
		inputArray.push(features.metrics.resourceUsage);
		inputArray.push(features.metrics.messageFrequency / 1000); // Normalize

		// Anomaly features
		const anomalyCount = features.anomalies.length;
		const criticalAnomalies = features.anomalies.filter((a) => a.severity === "critical").length;
		const highAnomalies = features.anomalies.filter((a) => a.severity === "high").length;

		inputArray.push(anomalyCount / 10); // Normalize
		inputArray.push(criticalAnomalies / 10);
		inputArray.push(highAnomalies / 10);

		// Historical threats (10 values)
		for (let i = 0; i < 10; i++) {
			inputArray.push(features.historicalThreats[i] ?? 0);
		}

		// Time since last threat (normalized to hours)
		inputArray.push(features.timeSinceLastThreat / (1000 * 60 * 60));

		return inputArray;
	}

	/**
	 * Train model on dataset
	 */
	async train(
		features: readonly ThreatFeatures[],
		labels: readonly ThreatPrediction[]
	): Promise<any> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare training data as 2D tensor [batchSize, featureCount]
		const featureArrays = features.map((f) => this.prepareInputArray(f));
		
		// Find the most common length and normalize all arrays to that length
		const lengths = featureArrays.map(arr => arr.length);
		const lengthCounts = new Map<number, number>();
		lengths.forEach(len => lengthCounts.set(len, (lengthCounts.get(len) || 0) + 1));
		const expectedLength = Array.from(lengthCounts.entries()).sort((a, b) => b[1] - a[1])[0][0];
		
		// Normalize all arrays to expected length
		const normalizedArrays = featureArrays.map(arr => {
			if (arr.length < expectedLength) {
				// Pad with zeros
				return [...arr, ...new Array(expectedLength - arr.length).fill(0)];
			} else if (arr.length > expectedLength) {
				// Truncate
				return arr.slice(0, expectedLength);
			}
			return arr;
		});
		
		const trainingData = tf.tensor2d(normalizedArrays);

		const labelData = tf.tensor2d(
			labels.map((l) => [l.threatScore, l.threatType, l.confidence, l.recommendedAction])
		);

		// Train model
		const history = await this.modelManager.trainModel(
			"threat_detector",
			trainingData,
			labelData,
			50 // epochs
		);

		// Cleanup
		trainingData.dispose();
		labelData.dispose();

		// Reload updated model
		this.model = await this.modelManager.loadModel("threat_detector");

		return history;
	}
}
