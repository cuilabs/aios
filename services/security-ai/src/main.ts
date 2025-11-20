/**
 * Security AI Service Entry Point
 */

import { SecurityAIServer } from "./server.js";

const server = new SecurityAIServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down Security AI Service...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down Security AI Service...");
	process.exit(0);
});
