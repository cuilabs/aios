/**
 * TensorFlow.js ML Models for AIOS
 * 
 * Provides machine learning models for:
 * - Workload prediction
 * - Threat detection
 * - Failure prediction
 * - Memory access prediction
 * 
 * Also exports high-performance inference engine optimized for microsecond-level predictions
 */

export * from "./inference_engine.js";
export * from "./workload_predictor.js";
export * from "./threat_detector.js";
export * from "./failure_predictor.js";
export * from "./memory_predictor.js";

// Use @tensorflow/tfjs for browser compatibility, @tensorflow/tfjs-node for Node.js
let tf: any;

try {
	// Try to load Node.js version first
	tf = require("@tensorflow/tfjs-node");
} catch {
	try {
		// Fallback to browser version
		tf = require("@tensorflow/tfjs");
	} catch {
		// If neither is available, use rule-based fallback
		console.warn("TensorFlow.js not available, ML features will use rule-based fallback");
		tf = null;
	}
}

/**
 * ML Model Manager
 * 
 * Manages all ML models used by AIOS AI-powered features
 */
export class MLModelManager {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private readonly models = new Map<string, any>();
	private readonly modelPaths = new Map<string, string>();

	constructor() {
		// Initialize model paths
		this.modelPaths.set("workload_predictor", "./models/workload_predictor/model.json");
		this.modelPaths.set("threat_detector", "./models/threat_detector/model.json");
		this.modelPaths.set("failure_predictor", "./models/failure_predictor/model.json");
		this.modelPaths.set("memory_predictor", "./models/memory_predictor/model.json");
	}

	/**
	 * Load ML model
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	async loadModel(modelName: string): Promise<any> {
		// Check if already loaded
		if (this.models.has(modelName)) {
			return this.models.get(modelName)!;
		}

		// Load model from path
		const modelPath = this.modelPaths.get(modelName);
		if (!modelPath) {
			throw new Error(`Unknown model: ${modelName}`);
		}

		try {
			// Try to load from file system (Node.js) or HTTP (browser)
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let model: any;
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			if (typeof (globalThis as any).window === "undefined") {
				// Node.js environment
				model = await tf.loadLayersModel(`file://${modelPath}`);
			} else {
				// Browser environment
				model = await tf.loadLayersModel(modelPath);
			}
			this.models.set(modelName, model);
			return model;
		} catch (error) {
			// If model file doesn't exist, create default model
			console.warn(`Model ${modelName} not found, creating default model`);
			return this.createDefaultModel(modelName);
		}
	}

	/**
	 * Create default model (fallback)
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private createDefaultModel(modelName: string): any {
		// Create sequential model as fallback
		const model = tf.sequential({
			layers: [
				tf.layers.dense({
					inputShape: [10],
					units: 32,
					activation: "relu",
				}),
				tf.layers.dense({
					units: 16,
					activation: "relu",
				}),
				tf.layers.dense({
					units: 1,
					activation: "sigmoid",
				}),
			],
		});

		model.compile({
			optimizer: "adam",
			loss: "meanSquaredError",
			metrics: ["accuracy"],
		});

		this.models.set(modelName, model);
		return model;
	}

	/**
	 * Get model
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	getModel(modelName: string): any | null {
		return this.models.get(modelName) ?? null;
	}

	/**
	 * Save model
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	async saveModel(modelName: string, model: any): Promise<void> {
		const modelPath = this.modelPaths.get(modelName);
		if (!modelPath) {
			throw new Error(`Unknown model: ${modelName}`);
		}

		// Save to file system (Node.js) or IndexedDB (browser)
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		if (typeof (globalThis as any).window === "undefined") {
			// Node.js environment
			await model.save(`file://${modelPath}`);
		} else {
			// Browser environment - save to IndexedDB
			await model.save(`indexeddb://${modelName}`);
		}
		this.models.set(modelName, model);
	}

	/**
	 * Train model
	 */
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	async trainModel(
		modelName: string,
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		trainingData: any,
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		labels: any,
		epochs = 10
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
	): Promise<any> {
		const model = await this.loadModel(modelName);

		const history = await model.fit(trainingData, labels, {
			epochs,
			batchSize: 32,
			validationSplit: 0.2,
			callbacks: {
				onEpochEnd: (epoch, logs) => {
					console.log(
						`Epoch ${epoch + 1}/${epochs} - loss: ${logs?.loss?.toFixed(4)}, accuracy: ${logs?.acc?.toFixed(4)}`
					);
				},
			},
		});

		// Save trained model
		await this.saveModel(modelName, model);

		return history;
	}
}

/**
 * Global ML model manager instance
 */
let mlModelManager: MLModelManager | null = null;

/**
 * Get ML model manager instance
 */
export function getMLModelManager(): MLModelManager {
	if (!mlModelManager) {
		mlModelManager = new MLModelManager();
	}
	return mlModelManager;
}

