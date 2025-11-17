/**
 * Planning and execution manager
 * Creates and executes plans for agent goals
 */

import type { PlanningTask, PlanningStep, ExecutionPlan, ExecutionStep } from "../types.js";

/**
 * Planning manager
 * Creates execution plans from agent goals
 */
export class PlanningManager {
	private readonly tasks = new Map<string, PlanningTask>();
	private readonly executionPlans = new Map<string, ExecutionPlan>();

	/**
	 * Create planning task
	 */
	createTask(agentId: string, goal: string, steps: readonly Omit<PlanningStep, "id" | "status">[]): PlanningTask {
		const taskId = this.generateTaskId();

		const planningSteps: PlanningStep[] = steps.map((step, index) => ({
			id: `step-${taskId}-${index}`,
			...step,
			status: "pending",
		}));

		const task: PlanningTask = {
			id: taskId,
			agentId,
			goal,
			steps: planningSteps,
			status: "pending",
			createdAt: Date.now(),
		};

		this.tasks.set(taskId, task);
		return task;
	}

	/**
	 * Get planning task
	 */
	getTask(taskId: string): PlanningTask | null {
		return this.tasks.get(taskId) ?? null;
	}

	/**
	 * Update task status
	 */
	updateTaskStatus(taskId: string, status: PlanningTask["status"]): boolean {
		const task = this.tasks.get(taskId);
		if (!task) {
			return false;
		}

		this.tasks.set(taskId, { ...task, status });
		return true;
	}

	/**
	 * Update step status
	 */
	updateStepStatus(taskId: string, stepId: string, status: PlanningStep["status"], result?: Readonly<Record<string, unknown>>): boolean {
		const task = this.tasks.get(taskId);
		if (!task) {
			return false;
		}

		const steps = task.steps.map((step) => {
			if (step.id === stepId) {
				return { ...step, status, result };
			}
			return step;
		});

		this.tasks.set(taskId, { ...task, steps });
		return true;
	}

	/**
	 * Create execution plan from planning task
	 */
	createExecutionPlan(taskId: string): ExecutionPlan | null {
		const task = this.tasks.get(taskId);
		if (!task) {
			return null;
		}

		const executionSteps: ExecutionStep[] = task.steps.map((step) => ({
			stepId: step.id,
			action: step.action,
			parameters: step.result ?? {},
			timeout: 5000, // Default 5s timeout
		}));

		// Calculate estimated duration (simplified)
		const estimatedDuration = executionSteps.length * 1000; // 1s per step

		const plan: ExecutionPlan = {
			taskId,
			steps: executionSteps,
			estimatedDuration,
			requiredResources: {},
		};

		this.executionPlans.set(taskId, plan);
		return plan;
	}

	/**
	 * Get execution plan
	 */
	getExecutionPlan(taskId: string): ExecutionPlan | null {
		return this.executionPlans.get(taskId) ?? null;
	}

	/**
	 * Execute plan
	 */
	async executePlan(taskId: string, executor: (step: ExecutionStep) => Promise<Readonly<Record<string, unknown>>>): Promise<boolean> {
		const plan = this.executionPlans.get(taskId);
		if (!plan) {
			return false;
		}

		this.updateTaskStatus(taskId, "executing");

		try {
			for (const step of plan.steps) {
				// Check dependencies
				const task = this.tasks.get(taskId);
				if (task) {
					const stepData = task.steps.find((s) => s.id === step.stepId);
					if (stepData) {
						for (const depId of stepData.dependencies) {
							const depStep = task.steps.find((s) => s.id === depId);
							if (!depStep || depStep.status !== "completed") {
								throw new Error(`Dependency ${depId} not completed`);
							}
						}
					}
				}

				// Execute step
				this.updateStepStatus(taskId, step.stepId, "executing");

				const result = await executor(step);

				this.updateStepStatus(taskId, step.stepId, "completed", result);
			}

			this.updateTaskStatus(taskId, "completed");
			return true;
		} catch (error) {
			this.updateTaskStatus(taskId, "failed");
			throw error;
		}
	}

	/**
	 * Generate unique task ID
	 */
	private generateTaskId(): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `task-${timestamp}-${random}`;
	}
}

