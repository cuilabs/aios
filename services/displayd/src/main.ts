/**
 * Display Server Main Entry Point
 */

import { DisplayServer } from "./server.js";

async function main(): Promise<void> {
	try {
		const server = new DisplayServer();
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
		console.error("Failed to start Display Server:", error);
		process.exit(1);
	}
}

main().catch((error) => {
	console.error("Unhandled error:", error);
	process.exit(1);
});
