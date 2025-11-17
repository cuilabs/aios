/**
 * Deterministic task scheduler for AIOS kernel
 * Provides predictable, fair scheduling for agent processes
 */

import type { ScheduledTask, SchedulingPolicy } from "../types.js";

export interface SchedulerMetrics {
	readonly totalTasks: number;
	readonly completedTasks: number;
	readonly averageLatency: number;
	readonly throughput: number;
}

/**
 * Deterministic scheduler for agent tasks
 * Ensures predictable execution order and resource allocation
 */
export class DeterministicScheduler {
	private readonly tasks: ScheduledTask[] = [];
	private readonly policy: SchedulingPolicy;
	private readonly metrics: SchedulerMetrics = {
		totalTasks: 0,
		completedTasks: 0,
		averageLatency: 0,
		throughput: 0,
	};

	constructor(policy: SchedulingPolicy) {
		this.policy = policy;
	}

	/**
	 * Schedule a task for execution
	 */
	schedule(task: ScheduledTask): void {
		this.tasks.push(task);
		this.metrics.totalTasks++;
		this.sortTasks();
	}

	/**
	 * Get next task to execute based on scheduling policy
	 */
	next(): ScheduledTask | null {
		if (this.tasks.length === 0) {
			return null;
		}

		const task = this.tasks.shift();
		if (task) {
			this.metrics.completedTasks++;
			this.updateMetrics();
		}
		return task ?? null;
	}

	/**
	 * Remove a task from the queue
	 */
	remove(taskId: string): boolean {
		const index = this.tasks.findIndex((t) => t.id === taskId);
		if (index !== -1) {
			this.tasks.splice(index, 1);
			return true;
		}
		return false;
	}

	/**
	 * Get current queue length
	 */
	getQueueLength(): number {
		return this.tasks.length;
	}

	/**
	 * Get scheduler metrics
	 */
	getMetrics(): Readonly<SchedulerMetrics> {
		return { ...this.metrics };
	}

	/**
	 * Sort tasks based on scheduling policy
	 */
	private sortTasks(): void {
		switch (this.policy.fairness) {
			case "fifo":
				// First in, first out - already sorted by insertion order
				break;
			case "priority":
				this.tasks.sort((a, b) => b.priority - a.priority);
				break;
			case "deadline":
				this.tasks.sort((a, b) => a.deadline - b.deadline);
				break;
			case "round-robin":
				// Round-robin is handled by the scheduler loop
				break;
		}
	}

	/**
	 * Update scheduler metrics
	 */
	private updateMetrics(): void {
		const now = Date.now();
		// Simplified metrics calculation
		this.metrics.throughput = this.metrics.completedTasks / (now / 1000);
	}
}

/**
 * Create a default deterministic scheduler
 */
export function createDeterministicScheduler(
	fairness: SchedulingPolicy["fairness"] = "deadline",
): DeterministicScheduler {
	const policy: SchedulingPolicy = {
		name: "deterministic",
		deterministic: true,
		fairness,
		quantum: 10, // 10ms time slice
	};

	return new DeterministicScheduler(policy);
}

