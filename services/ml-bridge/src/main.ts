/**
 * ML Bridge Service Main Entry Point
 */

import { MLBridgeServer } from "./server.js";

async function main(): Promise<void> {
	try {
		const server = new MLBridgeServer();
		await server.start();

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
		console.error("Failed to start ML Bridge Service:", error);
		process.exit(1);
	}
}

main().catch((error) => {
	console.error("Unhandled error:", error);
	process.exit(1);
});
