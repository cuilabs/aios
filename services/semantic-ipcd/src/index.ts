/**
 * Semantic IPC Daemon (semantic-ipcd)
 *
 * Userland daemon that interprets semantic intent from binary IPC messages.
 * Kernel handles only binary routing; this daemon provides semantic layer.
 */

import { IntentInterpreter } from "./interpreter/index.js";
import { MessageRouter } from "./router/index.js";
import type { SemanticMessage } from "./types.js";

/**
 * Semantic IPC Daemon
 *
 * Runs as privileged system service, interprets binary IPC messages
 * and routes based on semantic intent.
 */
export class SemanticIPCDaemon {
	private readonly interpreter: IntentInterpreter;
	private readonly router: MessageRouter;

	constructor() {
		this.interpreter = new IntentInterpreter();
		this.router = new MessageRouter();
	}

	/**
	 * Start the daemon
	 */
	async start(): Promise<void> {
		// Register kernel IPC message handler
		// Uses kernel IPCRecv syscall via kernel-bridge service
		this.startMessageProcessing();
	}

	/**
	 * Start message processing loop
	 */
	private startMessageProcessing(): void {
		// Poll for binary IPC messages from kernel
		// Process each message through interpreter and router
		// Daemon agent ID is 0 (system service)
		const daemonAgentId = 0;
		const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";

		setInterval(async () => {
			// Receive messages from kernel IPC via kernel-bridge service
			try {
				const response = await fetch(`${kernelBridgeUrl}/api/kernel/ipc/recv`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ agentId: daemonAgentId }),
				});

				if (response.ok) {
					const result = (await response.json()) as {
						success: boolean;
						message?: BinaryIPCMessage;
					};
					if (result.success && result.message) {
						await this.processMessage(result.message);
					}
				}
			} catch (error) {
				// Log error but continue processing
				console.error("Error receiving IPC message:", error);
			}
		}, 100); // Every 100ms
	}

	/**
	 * Process binary IPC message
	 */
	async processMessage(binaryMessage: BinaryIPCMessage): Promise<void> {
		// Extract semantic intent from binary message
		const intent = this.interpreter.interpret(binaryMessage);

		// Create semantic message
		const semanticMessage: SemanticMessage = {
			id: binaryMessage.id,
			from: binaryMessage.from,
			to: binaryMessage.to,
			intent,
			payload: binaryMessage.data,
			timestamp: binaryMessage.timestamp,
		};

		// Route based on semantic intent
		await this.router.route(semanticMessage);
	}
}

interface BinaryIPCMessage {
	id: number;
	from: number;
	to: number;
	data: Uint8Array;
	metadata: Uint8Array;
	timestamp: number;
}
