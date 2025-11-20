/**
 * ML Daemon Service Main Entry Point
 * 
 * High-performance ML inference service for AI-powered kernel subsystems
 */

import { MLDaemonServer } from "./server.js";

async function main(): Promise<void> {
	try {
		// Start HTTP server
		const server = new MLDaemonServer();
		await server.start();

		// Graceful shutdown
		process.on("SIGINT", async () => {
			console.log("\nReceived SIGINT, shutting down gracefully...");
			await server.stop();
			process.exit(0);
		});

		process.on("SIGTERM", async () => {
			console.log("\nReceived SIGTERM, shutting down gracefully...");
			await server.stop();
			process.exit(0);
		});
	} catch (error) {
		console.error("Failed to start ML Daemon Service:", error);
		process.exit(1);
	}
}

main().catch((error) => {
	console.error("Unhandled error:", error);
	process.exit(1);
});

