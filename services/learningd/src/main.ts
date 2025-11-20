/**
 * Adaptive Learning Service Entry Point
 */

import { LearningServer } from "./server.js";

const server = new LearningServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down Adaptive Learning Service...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down Adaptive Learning Service...");
	process.exit(0);
});
