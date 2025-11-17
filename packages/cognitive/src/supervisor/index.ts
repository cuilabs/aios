/**
 * Agent supervisor
 * Oversees agent lifecycle and coordination
 */

import type { AgentContext } from "../context/index.js";
import { ContextAllocator } from "../context/index.js";
import type { PlanningTask } from "../planning/index.js";
import { PlanningManager } from "../planning/index.js";

export interface AgentSupervisorConfig {
	readonly maxConcurrentAgents: number;
	readonly contextAllocator: ContextAllocator;
	readonly planningManager: PlanningManager;
}

export interface AgentStatus {
	readonly agentId: string;
	readonly contexts: readonly AgentContext[];
	readonly activeTasks: readonly PlanningTask[];
	readonly state: "active" | "idle" | "error";
}

/**
 * Agent supervisor
 * Manages agent lifecycle, contexts, and task coordination
 */
export class AgentSupervisor {
	private readonly config: AgentSupervisorConfig;
	private readonly activeAgents = new Set<string>();
	private readonly agentStatuses = new Map<string, AgentStatus>();

	constructor(config: AgentSupervisorConfig) {
		this.config = config;
	}

	/**
	 * Register agent
	 */
	registerAgent(agentId: string): boolean {
		if (this.activeAgents.size >= this.config.maxConcurrentAgents) {
			return false;
		}

		this.activeAgents.add(agentId);
		this.updateAgentStatus(agentId);
		return true;
	}

	/**
	 * Unregister agent
	 */
	unregisterAgent(agentId: string): boolean {
		const removed = this.activeAgents.delete(agentId);
		if (removed) {
			// Release all contexts
			const contexts = this.config.contextAllocator.getByAgent(agentId);
			for (const context of contexts) {
				this.config.contextAllocator.release(context.id);
			}
			this.agentStatuses.delete(agentId);
		}
		return removed;
	}

	/**
	 * Get agent status
	 */
	getAgentStatus(agentId: string): AgentStatus | null {
		return this.agentStatuses.get(agentId) ?? null;
	}

	/**
	 * Get all agent statuses
	 */
	getAllAgentStatuses(): readonly AgentStatus[] {
		return Array.from(this.agentStatuses.values());
	}

	/**
	 * Update agent status
	 */
	private updateAgentStatus(agentId: string): void {
		const contexts = this.config.contextAllocator.getByAgent(agentId);

		// Get active tasks (simplified - in production, query planning manager)
		const activeTasks: PlanningTask[] = [];

		const state: AgentStatus["state"] = contexts.length > 0 ? "active" : "idle";

		const status: AgentStatus = {
			agentId,
			contexts,
			activeTasks,
			state,
		};

		this.agentStatuses.set(agentId, status);
	}

	/**
	 * Get supervisor statistics
	 */
	getStats(): { totalAgents: number; activeAgents: number; totalContexts: number } {
		const contextStats = this.config.contextAllocator.getStats();

		return {
			totalAgents: this.activeAgents.size,
			activeAgents: Array.from(this.agentStatuses.values()).filter((s) => s.state === "active").length,
			totalContexts: contextStats.total,
		};
	}
}

