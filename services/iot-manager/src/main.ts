/**
 * IoT Device Management Service Entry Point
 */

import { IoTManagerServer } from "./server.js";

const server = new IoTManagerServer();
server.start();

// Graceful shutdown
process.on("SIGINT", () => {
	console.log("Shutting down IoT Device Management Service...");
	process.exit(0);
});

process.on("SIGTERM", () => {
	console.log("Shutting down IoT Device Management Service...");
	process.exit(0);
});
