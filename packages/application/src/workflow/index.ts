/**
 * Agent workflow manager
 * Orchestrates multi-step agent processes
 */

import type { Workflow, WorkflowStep } from "../types.js";

/**
 * Workflow manager
 * Manages agent workflows and step execution
 */
export class WorkflowManager {
	private readonly workflows = new Map<string, Workflow>();

	/**
	 * Create workflow
	 */
	create(name: string, steps: readonly Omit<WorkflowStep, "id" | "status">[]): Workflow {
		const workflowId = this.generateWorkflowId();

		const workflowSteps: WorkflowStep[] = steps.map((step, index) => ({
			id: `step-${workflowId}-${index}`,
			...step,
			status: "pending",
		}));

		const workflow: Workflow = {
			id: workflowId,
			name,
			steps: workflowSteps,
			status: "idle",
			createdAt: Date.now(),
		};

		this.workflows.set(workflowId, workflow);
		return workflow;
	}

	/**
	 * Get workflow
	 */
	get(workflowId: string): Workflow | null {
		return this.workflows.get(workflowId) ?? null;
	}

	/**
	 * Execute workflow
	 */
	async execute(workflowId: string, executor: (step: WorkflowStep) => Promise<Readonly<Record<string, unknown>>>): Promise<boolean> {
		const workflow = this.workflows.get(workflowId);
		if (!workflow) {
			return false;
		}

		this.updateStatus(workflowId, "running");

		try {
			const executedSteps = new Set<string>();

			// Execute steps in dependency order
			while (executedSteps.size < workflow.steps.length) {
				let progress = false;

				for (const step of workflow.steps) {
					if (executedSteps.has(step.id)) {
						continue;
					}

					// Check dependencies
					const allDepsMet = step.dependencies.every((depId) => executedSteps.has(depId));

					if (allDepsMet) {
						this.updateStepStatus(workflowId, step.id, "running");

						try {
							await executor(step);
							this.updateStepStatus(workflowId, step.id, "completed");
							executedSteps.add(step.id);
							progress = true;
						} catch (error) {
							this.updateStepStatus(workflowId, step.id, "failed");
							throw error;
						}
					}
				}

				if (!progress) {
					throw new Error("Circular dependency or missing dependencies detected");
				}
			}

			this.updateStatus(workflowId, "completed");
			return true;
		} catch (error) {
			this.updateStatus(workflowId, "failed");
			throw error;
		}
	}

	/**
	 * Update workflow status
	 */
	private updateStatus(workflowId: string, status: Workflow["status"]): void {
		const workflow = this.workflows.get(workflowId);
		if (workflow) {
			this.workflows.set(workflowId, { ...workflow, status });
		}
	}

	/**
	 * Update step status
	 */
	private updateStepStatus(workflowId: string, stepId: string, status: WorkflowStep["status"]): void {
		const workflow = this.workflows.get(workflowId);
		if (!workflow) {
			return;
		}

		const steps = workflow.steps.map((step) => {
			if (step.id === stepId) {
				return { ...step, status };
			}
			return step;
		});

		this.workflows.set(workflowId, { ...workflow, steps });
	}

	/**
	 * List workflows
	 */
	list(): readonly Workflow[] {
		return Array.from(this.workflows.values());
	}

	/**
	 * Remove workflow
	 */
	remove(workflowId: string): boolean {
		return this.workflows.delete(workflowId);
	}

	/**
	 * Generate unique workflow ID
	 */
	private generateWorkflowId(): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `workflow-${timestamp}-${random}`;
	}
}

