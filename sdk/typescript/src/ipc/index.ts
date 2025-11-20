/**
 * IPC API
 */

import type { SemanticMessageBus } from "@aios/ipc";

export class IPCClient {
	constructor(private messageBus: SemanticMessageBus) {}

	/**
	 * Send message
	 */
	async send(to: number, data: Uint8Array): Promise<void> {
		await this.messageBus.publish(`agent.${to}`, { data });
	}

	/**
	 * Receive message
	 */
	async receive(): Promise<IPCMessage | null> {
		// TODO: Receive IPC message
		return null;
	}
}

export interface IPCMessage {
	from: number;
	data: Uint8Array;
}
