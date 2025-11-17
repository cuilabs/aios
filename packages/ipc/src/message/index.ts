/**
 * Semantic message construction and validation
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { SemanticMessage, SemanticIntent, MessageResponse } from "../types.js";

/**
 * Semantic message builder
 */
export class SemanticMessageBuilder {
	/**
	 * Create a semantic message
	 */
	static create(
		from: string,
		to: string | string[],
		intent: SemanticIntent,
		payload: Readonly<Record<string, unknown>> = {},
		options: {
			ttl?: number;
			requiresResponse?: boolean;
		} = {},
	): SemanticMessage {
		const id = this.generateMessageId();

		return {
			id,
			from,
			to,
			intent,
			payload,
			timestamp: Date.now(),
			ttl: options.ttl,
			requiresResponse: options.requiresResponse ?? false,
		};
	}

	/**
	 * Create response message
	 */
	static createResponse(
		originalMessage: SemanticMessage,
		from: string,
		success: boolean,
		result?: Readonly<Record<string, unknown>>,
		error?: string,
	): MessageResponse {
		return {
			messageId: originalMessage.id,
			from,
			success,
			result,
			error,
			timestamp: Date.now(),
		};
	}

	/**
	 * Validate message structure
	 */
	static validate(message: SemanticMessage): { valid: boolean; errors: string[] } {
		const errors: string[] = [];

		if (!message.id || message.id.length === 0) {
			errors.push("Message ID is required");
		}

		if (!message.from || message.from.length === 0) {
			errors.push("Sender ID is required");
		}

		if (!message.to || (Array.isArray(message.to) && message.to.length === 0)) {
			errors.push("Recipient ID(s) required");
		}

		if (!message.intent) {
			errors.push("Intent is required");
		} else {
			if (!message.intent.type || message.intent.type.length === 0) {
				errors.push("Intent type is required");
			}
			if (!message.intent.action || message.intent.action.length === 0) {
				errors.push("Intent action is required");
			}
		}

		if (message.timestamp <= 0) {
			errors.push("Valid timestamp is required");
		}

		if (message.ttl !== undefined && message.ttl <= 0) {
			errors.push("TTL must be positive if provided");
		}

		return {
			valid: errors.length === 0,
			errors,
		};
	}

	/**
	 * Check if message is expired
	 */
	static isExpired(message: SemanticMessage): boolean {
		if (message.ttl === undefined) {
			return false;
		}

		const age = Date.now() - message.timestamp;
		return age > message.ttl;
	}

	/**
	 * Generate unique message ID
	 */
	private static generateMessageId(): string {
		const bytes = QuantumSafeCrypto.randomBytes(16);
		return `msg-${Array.from(bytes)
			.map((b) => b.toString(16).padStart(2, "0"))
			.join("")}`;
	}
}

