/**
 * Semantic message bus
 * Routes messages between agents based on intent and context
 */

import { SemanticMessageBuilder } from "../message/index.js";
import type { MessageFilter, MessageHandler, MessageResponse, SemanticMessage } from "../types.js";

export interface BusMetrics {
	totalMessages: number;
	deliveredMessages: number;
	failedMessages: number;
	averageLatency: number;
}

/**
 * Semantic message bus
 * Handles routing and delivery of semantic messages between agents
 */
export class SemanticMessageBus {
	private readonly handlers = new Map<string, MessageHandler>();
	private readonly pendingResponses = new Map<string, (response: MessageResponse) => void>();
	private readonly messageQueue: SemanticMessage[] = [];
	private readonly metrics: BusMetrics = {
		totalMessages: 0,
		deliveredMessages: 0,
		failedMessages: 0,
		averageLatency: 0,
	};
	private readonly subscriptions = new Map<
		string,
		Array<{ filter: MessageFilter; callback: (message: SemanticMessage) => void }>
	>();

	/**
	 * Register message handler for an agent
	 */
	register(handler: MessageHandler): void {
		this.handlers.set(handler.agentId, handler);
	}

	/**
	 * Unregister message handler
	 */
	unregister(agentId: string): boolean {
		return this.handlers.delete(agentId);
	}

	/**
	 * Send semantic message
	 */
	async send(message: SemanticMessage): Promise<MessageResponse | null> {
		// Validate message
		const validation = SemanticMessageBuilder.validate(message);
		if (!validation.valid) {
			this.metrics.failedMessages++;
			throw new Error(`Invalid message: ${validation.errors.join(", ")}`);
		}

		// Check expiration
		if (SemanticMessageBuilder.isExpired(message)) {
			this.metrics.failedMessages++;
			throw new Error("Message expired");
		}

		this.metrics.totalMessages++;
		const startTime = Date.now();

		const recipients = Array.isArray(message.to) ? message.to : [message.to];
		const responses: MessageResponse[] = [];

		for (const recipientId of recipients) {
			const handler = this.handlers.get(recipientId);
			if (!handler) {
				this.metrics.failedMessages++;
				continue;
			}

			try {
				const response = await handler.handle(message);
				if (response) {
					responses.push(response);
				}
				this.metrics.deliveredMessages++;
			} catch (error) {
				this.metrics.failedMessages++;
				const errorResponse = SemanticMessageBuilder.createResponse(
					message,
					recipientId,
					false,
					undefined,
					error instanceof Error ? error.message : "Unknown error"
				);
				responses.push(errorResponse);
			}
		}

		// Update latency metrics
		const latency = Date.now() - startTime;
		this.metrics.averageLatency =
			(this.metrics.averageLatency * (this.metrics.totalMessages - 1) + latency) /
			this.metrics.totalMessages;

		// Return first response if only one recipient, otherwise return null
		return responses.length === 1 ? (responses[0] ?? null) : null;
	}

	/**
	 * Send message and wait for response
	 */
	async sendAndWait(message: SemanticMessage, timeout = 5000): Promise<MessageResponse> {
		return new Promise((resolve, reject) => {
			const timeoutId = setTimeout(() => {
				this.pendingResponses.delete(message.id);
				reject(new Error("Message timeout"));
			}, timeout);

			this.pendingResponses.set(message.id, (response) => {
				clearTimeout(timeoutId);
				this.pendingResponses.delete(message.id);
				resolve(response);
			});

			this.send(message).catch((error) => {
				clearTimeout(timeoutId);
				this.pendingResponses.delete(message.id);
				reject(error);
			});
		});
	}

	/**
	 * Publish message to queue (async delivery)
	 */
	publish(message: SemanticMessage): void {
		const validation = SemanticMessageBuilder.validate(message);
		if (!validation.valid) {
			throw new Error(`Invalid message: ${validation.errors.join(", ")}`);
		}

		this.messageQueue.push(message);
		this.processQueue().catch((error) => {
			console.error("Error processing message queue:", error);
		});
	}

	/**
	 * Subscribe to messages matching filter
	 * Implements event emitter pattern for message subscriptions
	 */
	subscribe(filter: MessageFilter, callback: (message: SemanticMessage) => void): () => void {
		const subscriptionId = this.generateSubscriptionId();
		const subscriptions = this.subscriptions.get(subscriptionId) ?? [];
		subscriptions.push({ filter, callback });
		this.subscriptions.set(subscriptionId, subscriptions);

		for (const message of this.messageQueue) {
			if (this.matchesFilter(message, filter)) {
				callback(message);
			}
		}

		return () => {
			const subs = this.subscriptions.get(subscriptionId);
			if (subs) {
				const index = subs.findIndex((s) => s.callback === callback);
				if (index !== -1) {
					subs.splice(index, 1);
					if (subs.length === 0) {
						this.subscriptions.delete(subscriptionId);
					}
				}
			}
		};
	}

	private generateSubscriptionId(): string {
		return `sub-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
	}

	/**
	 * Get bus metrics
	 */
	getMetrics(): Readonly<BusMetrics> {
		return { ...this.metrics };
	}

	/**
	 * Process message queue
	 */
	private async processQueue(): Promise<void> {
		while (this.messageQueue.length > 0) {
			const message = this.messageQueue.shift();
			if (message) {
				await this.send(message);
			}
		}
	}

	/**
	 * Check if message matches filter
	 */
	private matchesFilter(message: SemanticMessage, filter: MessageFilter): boolean {
		if (filter.from && message.from !== filter.from) {
			return false;
		}

		const recipients = Array.isArray(message.to) ? message.to : [message.to];
		if (filter.to && !recipients.includes(filter.to)) {
			return false;
		}

		if (filter.intentType && message.intent.type !== filter.intentType) {
			return false;
		}

		if (filter.priority !== undefined && message.intent.priority !== filter.priority) {
			return false;
		}

		return true;
	}
}
