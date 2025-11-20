/**
 * Agent Supervisor Service Main Entry Point
 * 
 * Starts both the IPC service and HTTP server.
 */

import { AgentSupervisorService } from "./index.js";
import { AgentSupervisorServer } from "./server.js";

async function main(): Promise<void> {
	try {
		// Initialize service
		const service = new AgentSupervisorService();
		await service.start();

		// Start HTTP server
		const server = new AgentSupervisorServer(service);
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
		console.error("Failed to start Agent Supervisor Service:", error);
		process.exit(1);
	}
}

main().catch((error) => {
	console.error("Unhandled error:", error);
	process.exit(1);
});

