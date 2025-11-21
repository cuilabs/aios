/**
 * Intent interpreter
 *
 * Interprets semantic intent from binary IPC messages
 */

import type { SemanticIntent } from "../types.js";

interface BinaryIPCMessage {
	data: Uint8Array;
	metadata: Uint8Array;
}

/**
 * Intent interpreter
 *
 * Extracts semantic intent from binary messages
 */
export class IntentInterpreter {
	/**
	 * Interpret binary message and extract semantic intent
	 */
	interpret(message: BinaryIPCMessage): SemanticIntent {
		// Parse metadata to extract intent
		const metadata = this.parseMetadata(message.metadata);

		return {
			type: (metadata.type as string) ?? "unknown",
			action: (metadata.action as string) ?? "unknown",
			constraints: (metadata.constraints as Readonly<Record<string, unknown>>) ?? {},
			context: (metadata.context as Readonly<Record<string, unknown>>) ?? {},
			priority: (metadata.priority as number) ?? 0,
		};
	}

	/**
	 * Parse metadata from binary format
	 */
	private parseMetadata(metadata: Uint8Array): Record<string, unknown> {
		try {
			const text = new TextDecoder().decode(metadata);
			return JSON.parse(text);
		} catch {
			return {};
		}
	}
}
