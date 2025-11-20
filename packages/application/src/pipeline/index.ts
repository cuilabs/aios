/**
 * Pipeline manager
 * Manages multi-stage agent pipelines
 */

import type { Pipeline, PipelineStage } from "../types.js";

/**
 * Pipeline manager
 * Manages agent pipelines with parallel and sequential stages
 */
export class PipelineManager {
	private readonly pipelines = new Map<string, Pipeline>();

	/**
	 * Create pipeline
	 */
	create(name: string, stages: readonly Omit<PipelineStage, "id" | "status">[]): Pipeline {
		const pipelineId = this.generatePipelineId();

		const pipelineStages: PipelineStage[] = stages.map((stage, index) => ({
			id: `stage-${pipelineId}-${index}`,
			...stage,
			status: "pending",
		}));

		const pipeline: Pipeline = {
			id: pipelineId,
			name,
			stages: pipelineStages,
			status: "idle",
			createdAt: Date.now(),
		};

		this.pipelines.set(pipelineId, pipeline);
		return pipeline;
	}

	/**
	 * Get pipeline
	 */
	get(pipelineId: string): Pipeline | null {
		return this.pipelines.get(pipelineId) ?? null;
	}

	/**
	 * Execute pipeline
	 */
	async execute(
		pipelineId: string,
		executor: (stage: PipelineStage) => Promise<void>
	): Promise<boolean> {
		const pipeline = this.pipelines.get(pipelineId);
		if (!pipeline) {
			return false;
		}

		this.updateStatus(pipelineId, "running");

		try {
			const executedStages = new Set<string>();

			// Execute stages in dependency order
			while (executedStages.size < pipeline.stages.length) {
				const readyStages = pipeline.stages.filter(
					(stage) =>
						!executedStages.has(stage.id) &&
						stage.dependencies.every((depId) => executedStages.has(depId))
				);

				if (readyStages.length === 0) {
					throw new Error("Circular dependency or missing dependencies detected");
				}

				// Execute parallel stages concurrently
				const parallelStages = readyStages.filter((s) => s.parallel);
				const sequentialStages = readyStages.filter((s) => !s.parallel);

				// Execute parallel stages
				if (parallelStages.length > 0) {
					await Promise.all(
						parallelStages.map(async (stage) => {
							this.updateStageStatus(pipelineId, stage.id, "running");
							try {
								await executor(stage);
								this.updateStageStatus(pipelineId, stage.id, "completed");
								executedStages.add(stage.id);
							} catch (error) {
								this.updateStageStatus(pipelineId, stage.id, "failed");
								throw error;
							}
						})
					);
				}

				// Execute sequential stages
				for (const stage of sequentialStages) {
					this.updateStageStatus(pipelineId, stage.id, "running");
					try {
						await executor(stage);
						this.updateStageStatus(pipelineId, stage.id, "completed");
						executedStages.add(stage.id);
					} catch (error) {
						this.updateStageStatus(pipelineId, stage.id, "failed");
						throw error;
					}
				}
			}

			this.updateStatus(pipelineId, "completed");
			return true;
		} catch (error) {
			this.updateStatus(pipelineId, "failed");
			throw error;
		}
	}

	/**
	 * Update pipeline status
	 */
	private updateStatus(pipelineId: string, status: Pipeline["status"]): void {
		const pipeline = this.pipelines.get(pipelineId);
		if (pipeline) {
			this.pipelines.set(pipelineId, { ...pipeline, status });
		}
	}

	/**
	 * Update stage status
	 */
	private updateStageStatus(
		pipelineId: string,
		stageId: string,
		status: PipelineStage["status"]
	): void {
		const pipeline = this.pipelines.get(pipelineId);
		if (!pipeline) {
			return;
		}

		const stages = pipeline.stages.map((stage) => {
			if (stage.id === stageId) {
				return { ...stage, status };
			}
			return stage;
		});

		this.pipelines.set(pipelineId, { ...pipeline, stages });
	}

	/**
	 * List pipelines
	 */
	list(): readonly Pipeline[] {
		return Array.from(this.pipelines.values());
	}

	/**
	 * Remove pipeline
	 */
	remove(pipelineId: string): boolean {
		return this.pipelines.delete(pipelineId);
	}

	/**
	 * Generate unique pipeline ID
	 */
	private generatePipelineId(): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `pipeline-${timestamp}-${random}`;
	}
}
