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
	readonly id: number;
	readonly from: number;
	readonly to: number;
	readonly intent: SemanticIntent;
	readonly payload: Uint8Array;
	readonly timestamp: number;
}

