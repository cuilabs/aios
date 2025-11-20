/**
 * ML Model Training Script
 *
 * Enterprise-grade production training pipeline for TensorFlow.js models
 * Trains all AIOS AI-powered feature models with real or synthetic data
 */

import * as fs from "fs";
import * as path from "path";
import { TrainingDataCollector } from "./data_collector";
import { FailurePredictorModel } from "./failure_predictor";
import { MemoryPredictorModel } from "./memory_predictor";
import { ThreatDetectorModel } from "./threat_detector";
import { WorkloadPredictorModel } from "./workload_predictor";

/**
 * Train all models with comprehensive training pipeline
 */
async function trainAllModels(): Promise<void> {
	console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
	console.log("🚀 AIOS ML Model Training Pipeline");
	console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
	console.log("");

	const startTime = Date.now();
	const dataCollector = new TrainingDataCollector();

	// Step 1: Collect training data
	console.log("📊 Step 1: Collecting training data...");
	const data = await dataCollector.collectAll();
	console.log(`   ✓ Workload samples: ${data.workload.features.length}`);
	console.log(`   ✓ Threat samples: ${data.threat.features.length}`);
	console.log(`   ✓ Failure samples: ${data.failure.features.length}`);
	console.log(`   ✓ Memory samples: ${data.memory.features.length}`);
	console.log("");

	// Ensure model directories exist
	const modelDir = path.join(__dirname, "../models");
	const modelDirs = [
		path.join(modelDir, "workload_predictor"),
		path.join(modelDir, "threat_detector"),
		path.join(modelDir, "failure_predictor"),
		path.join(modelDir, "memory_predictor"),
	];

	for (const dir of modelDirs) {
		fs.mkdirSync(dir, { recursive: true });
	}

	// Step 2: Train workload predictor
	if (data.workload.features.length > 0) {
		console.log("🧠 Step 2: Training Workload Predictor Model...");
		console.log(`   Training on ${data.workload.features.length} samples...`);
		const workloadModel = new WorkloadPredictorModel();
		await workloadModel.initialize();
		const workloadHistory = await workloadModel.train(data.workload.features, data.workload.labels);
		const finalLoss = workloadHistory.history.loss?.[workloadHistory.history.loss.length - 1];
		const finalAcc = workloadHistory.history.acc?.[workloadHistory.history.acc.length - 1];
		console.log(
			`   ✓ Workload predictor trained - Loss: ${finalLoss?.toFixed(4) || "N/A"}, Acc: ${finalAcc?.toFixed(4) || "N/A"}`
		);
		console.log("");
	} else {
		console.log("⚠️  Skipping workload predictor (no training data)");
		console.log("");
	}

	// Step 3: Train threat detector
	if (data.threat.features.length > 0) {
		console.log("🛡️  Step 3: Training Threat Detector Model...");
		console.log(`   Training on ${data.threat.features.length} samples...`);
		const threatModel = new ThreatDetectorModel();
		await threatModel.initialize();
		const threatHistory = await threatModel.train(data.threat.features, data.threat.labels);
		const finalLoss = threatHistory.history.loss?.[threatHistory.history.loss.length - 1];
		const finalAcc = threatHistory.history.acc?.[threatHistory.history.acc.length - 1];
		console.log(
			`   ✓ Threat detector trained - Loss: ${finalLoss?.toFixed(4) || "N/A"}, Acc: ${finalAcc?.toFixed(4) || "N/A"}`
		);
		console.log("");
	} else {
		console.log("⚠️  Skipping threat detector (no training data)");
		console.log("");
	}

	// Step 4: Train failure predictor
	if (data.failure.features.length > 0) {
		console.log("⚠️  Step 4: Training Failure Predictor Model...");
		console.log(`   Training on ${data.failure.features.length} samples...`);
		const failureModel = new FailurePredictorModel();
		await failureModel.initialize();
		const failureHistory = await failureModel.train(data.failure.features, data.failure.labels);
		const finalLoss = failureHistory.history.loss?.[failureHistory.history.loss.length - 1];
		const finalAcc = failureHistory.history.acc?.[failureHistory.history.acc.length - 1];
		console.log(
			`   ✓ Failure predictor trained - Loss: ${finalLoss?.toFixed(4) || "N/A"}, Acc: ${finalAcc?.toFixed(4) || "N/A"}`
		);
		console.log("");
	} else {
		console.log("⚠️  Skipping failure predictor (no training data)");
		console.log("");
	}

	// Step 5: Train memory predictor
	if (data.memory.features.length > 0) {
		console.log("🧠 Step 5: Training Memory Predictor Model...");
		console.log(`   Training on ${data.memory.features.length} samples...`);
		const memoryModel = new MemoryPredictorModel();
		await memoryModel.initialize();
		const memoryHistory = await memoryModel.train(data.memory.features, data.memory.labels);
		const finalLoss = memoryHistory.history.loss?.[memoryHistory.history.loss.length - 1];
		const finalAcc = memoryHistory.history.acc?.[memoryHistory.history.acc.length - 1];
		console.log(
			`   ✓ Memory predictor trained - Loss: ${finalLoss?.toFixed(4) || "N/A"}, Acc: ${finalAcc?.toFixed(4) || "N/A"}`
		);
		console.log("");
	} else {
		console.log("⚠️  Skipping memory predictor (no training data)");
		console.log("");
	}

	const duration = ((Date.now() - startTime) / 1000).toFixed(2);
	console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
	console.log("✅ All models trained successfully!");
	console.log(`   Total duration: ${duration}s`);
	console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

// Run training if executed directly
if (require.main === module) {
	trainAllModels().catch(console.error);
}

export { trainAllModels };
