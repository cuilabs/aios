/**
 * Planning Service (/svc/planner)
 *
 * Privileged system service for agent planning and execution.
 * Runs in userland, not as kernel syscall.
 */

import { PlanningManager } from "@aios/cognitive";
import type { ExecutionPlan, PlanningTask } from "./types.js";

/**
 * Planning Service
 *
 * Provides planning and execution services for agents.
 * Runs as privileged system service in userland.
 */
export class PlanningService {
	private readonly planningManager: PlanningManager;

	constructor() {
		this.planningManager = new PlanningManager();
	}

	/**
	 * Create planning task
	 *
	 * Called via userland API, not kernel syscall
	 */
	async createTask(
		agentId: string,
		goal: string,
		steps: readonly Omit<PlanningTask["steps"][number], "id" | "status">[]
	): Promise<PlanningTask> {
		return this.planningManager.createTask(agentId, goal, steps);
	}

	/**
	 * Execute planning task
	 *
	 * Called via userland API, not kernel syscall
	 */
	async executeTask(
		taskId: string,
		executor: (step: ExecutionPlan["steps"][number]) => Promise<Readonly<Record<string, unknown>>>
	): Promise<boolean> {
		return this.planningManager.executePlan(taskId, executor);
	}

	/**
	 * Get planning task status
	 */
	async getTaskStatus(taskId: string): Promise<PlanningTask | null> {
		return this.planningManager.getTask(taskId);
	}

	/**
	 * Start the service
	 */
	async start(): Promise<void> {
		// Register IPC handlers
		// Start task execution loop
		this.startExecutionLoop();
	}

	/**
	 * Task execution loop
	 */
	private startExecutionLoop(): void {
		setInterval(async () => {
			// Process pending tasks
			// Implementation depends on PlanningManager API
		}, 1000); // Every second
	}
}
