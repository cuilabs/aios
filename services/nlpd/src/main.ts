/**
 * NLP Integration Service Entry Point
 */

import { NLPServer } from "./server.js";

const server = new NLPServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down NLP Integration Service...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down NLP Integration Service...");
	process.exit(0);
});
