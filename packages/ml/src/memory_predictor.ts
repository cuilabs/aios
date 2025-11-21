/**
 * Memory Access Prediction ML Model
 *
 * Uses TensorFlow.js to predict memory access patterns
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
 * Memory prediction input features
 */
export interface MemoryFeatures {
	readonly agentId: string;
	readonly accessHistory: readonly number[]; // Last 20 virtual addresses (normalized)
	readonly accessTypes: readonly number[]; // Last 20 access types (0=read, 1=write, 2=execute)
	readonly accessTimestamps: readonly number[]; // Last 20 timestamps (normalized)
	readonly currentAddress: number; // Current virtual address (normalized)
	readonly localityScore: number; // 0.0 to 1.0
}

/**
 * Memory prediction output
 */
export interface MemoryPrediction {
	readonly nextAddress: number; // Predicted next virtual address (normalized)
	readonly accessProbability: number; // 0.0 to 1.0
	readonly accessType: number; // 0-2 (read, write, execute)
	readonly confidence: number; // 0.0 to 1.0
}

/**
 * Memory Predictor ML Model
 */
export class MemoryPredictorModel {
	private model: any | null = null;
	private readonly modelManager = getMLModelManager();

	/**
	 * Initialize model
	 */
	async initialize(): Promise<void> {
		this.model = await this.modelManager.loadModel("memory_predictor");
	}

	/**
	 * Predict next memory access
	 */
	async predict(features: MemoryFeatures): Promise<MemoryPrediction> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare input tensor
		const input = this.prepareInput(features);

		// Run prediction
		const prediction = this.model!.predict(input) as any;

		// Extract predictions
		const values = await prediction.data();
		const nextAddress = Math.max(0, Math.min(1, values[0]));
		const accessProbability = Math.max(0, Math.min(1, values[1]));
		const accessType = Math.round(Math.max(0, Math.min(2, values[2])));
		const confidence = Math.max(0, Math.min(1, values[3]));

		// Cleanup
		input.dispose();
		prediction.dispose();

		return {
			nextAddress,
			accessProbability,
			accessType,
			confidence,
		};
	}

	/**
	 * Prepare input tensor from features
	 */
	private prepareInput(features: MemoryFeatures): any {
		const inputArray: number[] = [];

		// Access history (20 addresses, normalized)
		for (let i = 0; i < 20; i++) {
			inputArray.push(features.accessHistory[i] ?? 0);
		}

		// Access types (20 values, one-hot encoded)
		for (let i = 0; i < 20; i++) {
			const type = features.accessTypes[i] ?? 0;
			// One-hot encode: [read, write, execute]
			inputArray.push(type === 0 ? 1 : 0);
			inputArray.push(type === 1 ? 1 : 0);
			inputArray.push(type === 2 ? 1 : 0);
		}

		// Access timestamps (20 values, normalized)
		const maxTimestamp = Math.max(...features.accessTimestamps, 1);
		for (let i = 0; i < 20; i++) {
			inputArray.push((features.accessTimestamps[i] ?? 0) / maxTimestamp);
		}

		// Current address
		inputArray.push(features.currentAddress);

		// Locality score
		inputArray.push(features.localityScore);

		// Create tensor
		return tf.tensor2d([inputArray], [1, inputArray.length]);
	}

	/**
	 * Prepare input array (without tensor) for batch training
	 */
	private prepareInputArray(features: MemoryFeatures): number[] {
		const inputArray: number[] = [];

		// Access history (20 addresses, normalized)
		for (let i = 0; i < 20; i++) {
			inputArray.push(features.accessHistory[i] ?? 0);
		}

		// Access types (20 values)
		for (let i = 0; i < 20; i++) {
			inputArray.push(features.accessTypes[i] ?? 0);
		}

		// Access timestamps (20 values, normalized)
		const maxTimestamp = Math.max(...features.accessTimestamps, 1);
		for (let i = 0; i < 20; i++) {
			inputArray.push(features.accessTimestamps[i] ? features.accessTimestamps[i] / maxTimestamp : 0);
		}

		// Current access pattern
		inputArray.push(features.currentAddress);
		inputArray.push(features.localityScore);

		return inputArray;
	}

	/**
	 * Train model on dataset
	 */
	async train(
		features: readonly MemoryFeatures[],
		labels: readonly MemoryPrediction[]
	): Promise<any> {
		if (!this.model) {
			await this.initialize();
		}

		// Prepare training data as 2D tensor [batchSize, featureCount]
		const featureArrays = features.map((f) => this.prepareInputArray(f));
		const trainingData = tf.tensor2d(featureArrays);

		const labelData = tf.tensor2d(
			labels.map((l) => [l.nextAddress, l.accessProbability, l.accessType, l.confidence])
		);

		// Train model
		const history = await this.modelManager.trainModel(
			"memory_predictor",
			trainingData,
			labelData,
			50 // epochs
		);

		// Cleanup
		trainingData.dispose();
		labelData.dispose();

		// Reload updated model
		this.model = await this.modelManager.loadModel("memory_predictor");

		return history;
	}
}
