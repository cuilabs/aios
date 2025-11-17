/**
 * Context allocator
 * Manages agent execution contexts and memory
 */

import type { AgentContext } from "../types.js";

/**
 * Context allocator
 * Allocates and manages execution contexts for agents
 */
export class ContextAllocator {
	private readonly contexts = new Map<string, AgentContext>();
	private readonly maxContexts: number;

	constructor(maxContexts: number = 1000) {
		this.maxContexts = maxContexts;
	}

	/**
	 * Allocate context for agent
	 */
	allocate(agentId: string, initialMemory: Readonly<Record<string, unknown>> = {}): AgentContext {
		if (this.contexts.size >= this.maxContexts) {
			throw new Error("Maximum context limit reached");
		}

		const contextId = this.generateContextId(agentId);
		const now = Date.now();

		const context: AgentContext = {
			id: contextId,
			agentId,
			memory: initialMemory,
			state: "idle",
			priority: 0,
			createdAt: now,
			lastUpdated: now,
		};

		this.contexts.set(contextId, context);
		return context;
	}

	/**
	 * Get context by ID
	 */
	get(contextId: string): AgentContext | null {
		return this.contexts.get(contextId) ?? null;
	}

	/**
	 * Get contexts for agent
	 */
	getByAgent(agentId: string): readonly AgentContext[] {
		const results: AgentContext[] = [];
		for (const context of this.contexts.values()) {
			if (context.agentId === agentId) {
				results.push(context);
			}
		}
		return results;
	}

	/**
	 * Update context
	 */
	update(contextId: string, updates: Partial<Omit<AgentContext, "id" | "createdAt">>): boolean {
		const context = this.contexts.get(contextId);
		if (!context) {
			return false;
		}

		const updated: AgentContext = {
			...context,
			...updates,
			lastUpdated: Date.now(),
		};

		this.contexts.set(contextId, updated);
		return true;
	}

	/**
	 * Update context memory
	 */
	updateMemory(contextId: string, memory: Readonly<Record<string, unknown>>): boolean {
		const context = this.contexts.get(contextId);
		if (!context) {
			return false;
		}

		return this.update(contextId, {
			memory: { ...context.memory, ...memory },
		});
	}

	/**
	 * Release context
	 */
	release(contextId: string): boolean {
		return this.contexts.delete(contextId);
	}

	/**
	 * Get all contexts
	 */
	getAll(): readonly AgentContext[] {
		return Array.from(this.contexts.values());
	}

	/**
	 * Get context statistics
	 */
	getStats(): { total: number; byState: Readonly<Record<string, number>> } {
		const byState: Record<string, number> = {};

		for (const context of this.contexts.values()) {
			byState[context.state] = (byState[context.state] ?? 0) + 1;
		}

		return {
			total: this.contexts.size,
			byState,
		};
	}

	/**
	 * Generate unique context ID
	 */
	private generateContextId(agentId: string): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `ctx-${agentId}-${timestamp}-${random}`;
	}
}

