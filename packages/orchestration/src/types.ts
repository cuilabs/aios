/**
 * Agent orchestration types
 */

export interface AgentSpec {
	readonly id: string;
	readonly type: string;
	readonly capabilities: readonly string[];
	readonly configuration: Readonly<Record<string, unknown>>;
	readonly parentId?: string;
}

export interface AgentPolicy {
	readonly allowSpawn: boolean;
	readonly allowClone: boolean;
	readonly allowMerge: boolean;
	readonly allowSplit: boolean;
	readonly allowUpgrade: boolean;
	readonly allowSpecialize: boolean;
	readonly maxInstances: number;
	readonly resourceLimits: Readonly<Record<string, unknown>>;
}

export interface AgentOperation {
	readonly id: string;
	readonly type: "spawn" | "clone" | "merge" | "split" | "upgrade" | "specialize";
	readonly agentId: string;
	readonly targetId?: string;
	readonly parameters: Readonly<Record<string, unknown>>;
	readonly status: "pending" | "executing" | "completed" | "failed";
	readonly createdAt: number;
}

