/**
 * Semantic message router
 *
 * Routes messages based on semantic intent
 */

import type { SemanticMessage } from "../types.js";

/**
 * Semantic message router
 *
 * Routes messages based on intent, not just agent ID
 */
export class MessageRouter {
	/**
	 * Route semantic message
	 */
	async route(message: SemanticMessage): Promise<void> {
		// Route based on semantic intent
		// Route to destination agent(s)
		const recipients = Array.isArray(message.to) ? message.to : [message.to];

		for (const recipientId of recipients) {
			await this.deliver(message, recipientId);
		}
	}

	/**
	 * Deliver message to agent
	 */
	private async deliver(message: SemanticMessage, agentId: string): Promise<void> {
		// Deliver message via kernel IPC
		// Kernel handles binary routing
		// Convert semantic message to binary IPC message
		const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";

		// Serialize semantic message to binary format
		const binaryData = new TextEncoder().encode(JSON.stringify(message));
		const metadata = new TextEncoder().encode(JSON.stringify(message.intent));

		// Send via kernel IPCSend syscall
		try {
			const response = await fetch(`${kernelBridgeUrl}/api/kernel/ipc/send`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					from: BigInt(message.from),
					to: BigInt(agentId),
					data: Array.from(binaryData),
					metadata: Array.from(metadata),
				}),
			});

			if (!response.ok) {
				console.error(`Failed to deliver message to agent ${agentId}: ${response.statusText}`);
			}
		} catch (error) {
			console.error(`Error delivering message to agent ${agentId}:`, error);
		}
	}
}
