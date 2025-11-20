/**
 * Planning service types
 */

export interface PlanningTask {
	readonly id: string;
	readonly agentId: string;
	readonly goal: string;
	readonly steps: readonly PlanningStep[];
	readonly status: "pending" | "executing" | "completed" | "failed";
	readonly createdAt: number;
}

export interface PlanningStep {
	readonly id: string;
	readonly action: string;
	readonly dependencies: readonly string[];
	readonly status: "pending" | "executing" | "completed" | "failed";
	readonly result?: Readonly<Record<string, unknown>>;
}

export interface ExecutionPlan {
	readonly taskId: string;
	readonly steps: readonly ExecutionStep[];
	readonly estimatedDuration: number;
	readonly requiredResources: Readonly<Record<string, unknown>>;
}

export interface ExecutionStep {
	readonly stepId: string;
	readonly action: string;
	readonly parameters: Readonly<Record<string, unknown>>;
	readonly timeout: number;
}
