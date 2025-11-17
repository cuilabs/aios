/**
 * Semantic IPC types
 */

export interface SemanticIntent {
	readonly type: string;
	readonly action: string;
	readonly constraints: Readonly<Record<string, unknown>>;
	readonly context: Readonly<Record<string, unknown>>;
	readonly priority: number;
}

export interface SemanticMessage {
	readonly id: string;
	readonly from: string;
	readonly to: string | string[];
	readonly intent: SemanticIntent;
	readonly payload: Readonly<Record<string, unknown>>;
	readonly timestamp: number;
	readonly ttl?: number;
	readonly requiresResponse: boolean;
}

export interface MessageResponse {
	readonly messageId: string;
	readonly from: string;
	readonly success: boolean;
	readonly result?: Readonly<Record<string, unknown>>;
	readonly error?: string;
	readonly timestamp: number;
}

export interface MessageHandler {
	readonly agentId: string;
	readonly handle: (message: SemanticMessage) => Promise<MessageResponse | null>;
}

export interface MessageFilter {
	readonly from?: string;
	readonly to?: string;
	readonly intentType?: string;
	readonly priority?: number;
}

