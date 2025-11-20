/**
 * Application layer types
 */

export interface Workflow {
	readonly id: string;
	readonly name: string;
	readonly steps: readonly WorkflowStep[];
	readonly status: "idle" | "running" | "completed" | "failed";
	readonly createdAt: number;
}

export interface WorkflowStep {
	readonly id: string;
	readonly agentId: string;
	readonly action: string;
	readonly parameters: Readonly<Record<string, unknown>>;
	readonly dependencies: readonly string[];
	readonly status: "pending" | "running" | "completed" | "failed";
}

export interface Pipeline {
	readonly id: string;
	readonly name: string;
	readonly stages: readonly PipelineStage[];
	readonly status: "idle" | "running" | "completed" | "failed";
	readonly createdAt: number;
}

export interface PipelineStage {
	readonly id: string;
	readonly name: string;
	readonly agents: readonly string[];
	readonly parallel: boolean;
	readonly dependencies: readonly string[];
	readonly status: "pending" | "running" | "completed" | "failed";
}

export interface Environment {
	readonly id: string;
	readonly name: string;
	readonly agents: readonly string[];
	readonly resources: Readonly<Record<string, unknown>>;
	readonly configuration: Readonly<Record<string, unknown>>;
}

export interface Toolchain {
	readonly id: string;
	readonly name: string;
	readonly tools: readonly Tool[];
	readonly configuration: Readonly<Record<string, unknown>>;
}

export interface Tool {
	readonly id: string;
	readonly name: string;
	readonly type: string;
	readonly capabilities: readonly string[];
	readonly configuration: Readonly<Record<string, unknown>>;
}
