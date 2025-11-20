/**
 * ML Data Collector Service Main Entry Point
 *
 * Runs the ML training data collection service
 */

import cors from "cors";
import express from "express";
import { MLDataCollectorService } from "./collector.js";
import { DataStorageManager } from "./data_storage.js";

const app = express();
app.use(cors());
app.use(express.json());

const collector = new MLDataCollectorService({
	metricsdUrl: process.env.METRICSD_URL || "http://127.0.0.1:9004",
	agentsupervisorUrl: process.env.AGENTSUPERVISOR_URL || "http://127.0.0.1:9001",
	securityAiUrl: process.env.SECURITY_AI_URL || "http://127.0.0.1:9010",
	collectionInterval: Number.parseInt(process.env.COLLECTION_INTERVAL || "5000", 10),
	dataDir: process.env.DATA_DIR || "./data/ml-training",
});

const PORT = Number.parseInt(process.env.PORT || "9016", 10);

// Health check endpoint
app.get("/health", (req, res) => {
	res.json({ status: "ok" });
});

// Statistics endpoint
app.get("/api/statistics", async (req, res) => {
	try {
		const stats = await collector.getStatistics();
		res.json(stats);
	} catch (error: any) {
		res.status(500).json({ error: error.message });
	}
});

// Start collection endpoint
app.post("/api/start", async (req, res) => {
	try {
		await collector.start();
		res.json({ status: "started" });
	} catch (error: any) {
		res.status(500).json({ error: error.message });
	}
});

// Stop collection endpoint
app.post("/api/stop", (req, res) => {
	try {
		collector.stop();
		res.json({ status: "stopped" });
	} catch (error: any) {
		res.status(500).json({ error: error.message });
	}
});

// Manual collection trigger
app.post("/api/collect", async (req, res) => {
	try {
		await collector.collect();
		res.json({ status: "collected" });
	} catch (error: any) {
		res.status(500).json({ error: error.message });
	}
});

// Start server
app.listen(PORT, async () => {
	console.log(`🚀 ML Data Collector Service listening on port ${PORT}`);

	// Auto-start collection
	await collector.start();

	console.log("✅ ML Data Collector Service started");
});

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("\n🛑 Shutting down ML Data Collector Service...");
	collector.stop();
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("\n🛑 Shutting down ML Data Collector Service...");
	collector.stop();
	process.exit(0);
});
