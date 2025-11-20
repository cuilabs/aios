/**
 * Workload Prediction ML Model
 * 
 * Uses TensorFlow.js to predict agent workload patterns
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
 * Workload prediction input features
 */
export interface WorkloadFeatures {
	readonly agentId: string;
	readonly historicalCpu: readonly number[]; // Last 10 CPU usage values
	readonly historicalMemory: readonly number[]; // Last 10 memory usage values
	readonly historicalGpu: readonly number[]; // Last 10 GPU usage values (optional)
	readonly timeOfDay: number; // Hour of day (0-23)
	readonly dayOfWeek: number; // Day of week (0-6)
	readonly currentCpu: number;
	readonly currentMemory: number;
	readonly currentGpu?: number;
}

/**
 * Workload prediction output
 */
export interface WorkloadPrediction {
	readonly predictedCpu: number; // 0.0 to 1.0
	readonly predictedMemory: number; // Bytes
	readonly predictedGpu?: number; // 0.0 to 1.0
	readonly confidence: number; // 0.0 to 1.0
}

/**
 * Workload Predictor ML Model
 */
export class WorkloadPredictorModel {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private model: any = null;
	private readonly modelManager = getMLModelManager();

	/**
	 * Initialize model
	 */
	async initialize(): Promise<void> {
		this.model = await this.modelManager.loadModel("workload_predictor");
	}

	/**
	 * Predict workload
	 */
	async predict(features: WorkloadFeatures): Promise<WorkloadPrediction> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare input tensor
		const input = this.prepareInput(features);

		// Run prediction
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const prediction = this.model!.predict(input) as any;

		// Extract predictions
		const values = await prediction.data();
		const predictedCpu = Math.max(0, Math.min(1, values[0]));
		const predictedMemory = Math.max(0, values[1]);
		const predictedGpu = features.currentGpu !== undefined ? Math.max(0, Math.min(1, values[2] ?? 0)) : undefined;
		const confidence = Math.max(0, Math.min(1, values[3] ?? 0.5));

		// Cleanup
		input.dispose();
		prediction.dispose();

		return {
			predictedCpu,
			predictedMemory,
			predictedGpu,
			confidence,
		};
	}

	/**
	 * Prepare input tensor from features
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private prepareInput(features: WorkloadFeatures): any {
		// Normalize features into a single vector
		const inputArray: number[] = [];

		// Historical CPU (10 values, normalized to 0-1)
		for (let i = 0; i < 10; i++) {
			inputArray.push(features.historicalCpu[i] ?? 0);
		}

		// Historical Memory (10 values, normalized)
		const maxMemory = Math.max(...features.historicalMemory, 1);
		for (let i = 0; i < 10; i++) {
			inputArray.push((features.historicalMemory[i] ?? 0) / maxMemory);
		}

		// Historical GPU (10 values, normalized to 0-1)
		if (features.historicalGpu) {
			for (let i = 0; i < 10; i++) {
				inputArray.push(features.historicalGpu[i] ?? 0);
			}
		} else {
			// Pad with zeros if no GPU data
			for (let i = 0; i < 10; i++) {
				inputArray.push(0);
			}
		}

		// Time features (normalized)
		inputArray.push(features.timeOfDay / 23); // 0-1
		inputArray.push(features.dayOfWeek / 6); // 0-1

		// Current values
		inputArray.push(features.currentCpu);
		inputArray.push(features.currentMemory / (1024 * 1024 * 1024)); // Normalize to GB

		if (features.currentGpu !== undefined) {
			inputArray.push(features.currentGpu);
		} else {
			inputArray.push(0);
		}

		// Create tensor (shape: [1, featureCount])
		return tf.tensor2d([inputArray], [1, inputArray.length]);
	}

	/**
	 * Train model on dataset
	 */
	async train(
		features: readonly WorkloadFeatures[],
		labels: readonly WorkloadPrediction[]
	): Promise<any> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare training data
		const trainingData = tf.stack(
			features.map((f) => this.prepareInput(f))
		) as any;

		const labelData = tf.tensor2d(
			labels.map((l) => [
				l.predictedCpu,
				l.predictedMemory / (1024 * 1024 * 1024), // Normalize
				l.predictedGpu ?? 0,
				l.confidence,
			])
		);

		// Train model
		const history = await this.modelManager.trainModel(
			"workload_predictor",
			trainingData,
			labelData,
			50 // epochs
		);

		// Cleanup
		trainingData.dispose();
		labelData.dispose();

		// Reload updated model
		this.model = await this.modelManager.loadModel("workload_predictor");

		return history;
	}
}

