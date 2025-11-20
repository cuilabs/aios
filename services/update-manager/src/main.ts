/**
 * Autonomous Update Manager Entry Point
 */

import { UpdateManagerServer } from "./server.js";

const server = new UpdateManagerServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down Autonomous Update Manager...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down Autonomous Update Manager...");
	process.exit(0);
});

