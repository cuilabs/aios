/**
 * AI-Powered UI/UX Service Entry Point
 */

import { UIAIServer } from "./server.js";

const server = new UIAIServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down AI-Powered UI/UX Service...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down AI-Powered UI/UX Service...");
	process.exit(0);
});

