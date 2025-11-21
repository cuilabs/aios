/**
 * Failure Prediction ML Model
 *
 * Uses TensorFlow.js to predict system failures before they occur
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

/**
 * Failure prediction input features
 */
export interface FailureFeatures {
	readonly component: string;
	readonly healthScore: number; // 0.0 to 1.0
	readonly currentValue: number;
	readonly baseline: number;
	readonly trend: number; // -1 (degrading) to 1 (improving)
	readonly historicalHealth: readonly number[]; // Last 20 health scores
	readonly failureHistory: readonly number[]; // Last 10 failure events (0 or 1)
	readonly timeSinceLastFailure: number; // Milliseconds
}

/**
 * Failure prediction output
 */
export interface FailurePrediction {
	readonly failureProbability: number; // 0.0 to 1.0
	readonly predictedTime: number; // Milliseconds until failure (or -1 if no prediction)
	readonly confidence: number; // 0.0 to 1.0
	readonly failureType: number; // 0-5 (enum index)
}

/**
 * Failure Predictor ML Model
 */
export class FailurePredictorModel {
	private model: any | null = null;
	private readonly modelManager = getMLModelManager();

	/**
	 * Initialize model
	 */
	async initialize(): Promise<void> {
		this.model = await this.modelManager.loadModel("failure_predictor");
	}

	/**
	 * Predict failure
	 */
	async predict(features: FailureFeatures): Promise<FailurePrediction> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare input tensor
		const input = this.prepareInput(features);

		// Run prediction
		const prediction = this.model!.predict(input) as any;

		// Extract predictions
		const values = await prediction.data();
		const failureProbability = Math.max(0, Math.min(1, values[0]));
		const predictedTime = values[1] > 0 ? values[1] * 1000 : -1; // Convert to milliseconds
		const confidence = Math.max(0, Math.min(1, values[2]));
		const failureType = Math.round(Math.max(0, Math.min(5, values[3])));

		// Cleanup
		input.dispose();
		prediction.dispose();

		return {
			failureProbability,
			predictedTime,
			confidence,
			failureType,
		};
	}

	/**
	 * Prepare input tensor from features
	 */
	private prepareInput(features: FailureFeatures): any {
		const inputArray: number[] = [];

		// Current metrics
		inputArray.push(features.healthScore);
		inputArray.push(features.currentValue);
		inputArray.push(features.baseline);
		inputArray.push(features.trend);

		// Historical health (20 values)
		for (let i = 0; i < 20; i++) {
			inputArray.push(features.historicalHealth[i] ?? 0);
		}

		// Failure history (10 values)
		for (let i = 0; i < 10; i++) {
			inputArray.push(features.failureHistory[i] ?? 0);
		}

		// Time since last failure (normalized to days)
		inputArray.push(features.timeSinceLastFailure / (1000 * 60 * 60 * 24));

		// Create tensor
		return tf.tensor2d([inputArray], [1, inputArray.length]);
	}

	/**
	 * Prepare input array (without tensor) for batch training
	 */
	private prepareInputArray(features: FailureFeatures): number[] {
		const inputArray: number[] = [];

		// Current metrics
		inputArray.push(features.healthScore);
		inputArray.push(features.currentValue);
		inputArray.push(features.trend);

		// Historical health (last 10)
		for (let i = 0; i < 10; i++) {
			inputArray.push(features.historicalHealth[i] ?? 0);
		}

		return inputArray;
	}

	/**
	 * Train model on dataset
	 */
	async train(
		features: readonly FailureFeatures[],
		labels: readonly FailurePrediction[]
	): Promise<any> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare training data as 2D tensor [batchSize, featureCount]
		const featureArrays = features.map((f) => this.prepareInputArray(f));
		const trainingData = tf.tensor2d(featureArrays);

		const labelData = tf.tensor2d(
			labels.map((l) => [
				l.failureProbability,
				l.predictedTime > 0 ? l.predictedTime / 1000 : 0, // Normalize to seconds
				l.confidence,
				l.failureType,
			])
		);

		// Train model
		const history = await this.modelManager.trainModel(
			"failure_predictor",
			trainingData,
			labelData,
			50 // epochs
		);

		// Cleanup
		trainingData.dispose();
		labelData.dispose();

		// Reload updated model
		this.model = await this.modelManager.loadModel("failure_predictor");

		return history;
	}
}
