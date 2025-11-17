/**
 * Agent orchestrator
 * Manages agent lifecycle operations
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { AgentSpec, AgentPolicy, AgentOperation } from "../types.js";

/**
 * Agent orchestrator
 * Handles spawn, clone, merge, split, upgrade, and specialize operations
 */
export class AgentOrchestrator {
	private readonly agents = new Map<string, AgentSpec>();
	private readonly policies = new Map<string, AgentPolicy>();
	private readonly operations = new Map<string, AgentOperation>();

	/**
	 * Set policy for agent type
	 */
	setPolicy(agentType: string, policy: AgentPolicy): void {
		this.policies.set(agentType, policy);
	}

	/**
	 * Get policy for agent type
	 */
	getPolicy(agentType: string): AgentPolicy | null {
		return this.policies.get(agentType) ?? null;
	}

	/**
	 * Spawn new agent
	 */
	spawn(spec: Omit<AgentSpec, "id">, policy?: AgentPolicy): AgentSpec {
		const agentType = spec.type;
		const effectivePolicy = policy ?? this.policies.get(agentType);

		if (effectivePolicy && !effectivePolicy.allowSpawn) {
			throw new Error(`Spawn not allowed for agent type: ${agentType}`);
		}

		// Check instance limit
		if (effectivePolicy) {
			const currentInstances = Array.from(this.agents.values()).filter((a) => a.type === agentType).length;
			if (currentInstances >= effectivePolicy.maxInstances) {
				throw new Error(`Maximum instances reached for agent type: ${agentType}`);
			}
		}

		const agentId = this.generateAgentId();
		const agent: AgentSpec = {
			...spec,
			id: agentId,
		};

		this.agents.set(agentId, agent);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "spawn",
			agentId,
			parameters: {},
			status: "completed",
			createdAt: Date.now(),
		});

		return agent;
	}

	/**
	 * Clone agent
	 */
	clone(agentId: string, modifications?: Partial<Omit<AgentSpec, "id" | "parentId">>): AgentSpec {
		const source = this.agents.get(agentId);
		if (!source) {
			throw new Error(`Agent not found: ${agentId}`);
		}

		const policy = this.policies.get(source.type);
		if (policy && !policy.allowClone) {
			throw new Error(`Clone not allowed for agent type: ${source.type}`);
		}

		const clonedId = this.generateAgentId();
		const cloned: AgentSpec = {
			...source,
			...modifications,
			id: clonedId,
			parentId: agentId,
		};

		this.agents.set(clonedId, cloned);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "clone",
			agentId: clonedId,
			targetId: agentId,
			parameters: modifications ?? {},
			status: "completed",
			createdAt: Date.now(),
		});

		return cloned;
	}

	/**
	 * Merge two agents
	 */
	merge(agentId1: string, agentId2: string): AgentSpec {
		const agent1 = this.agents.get(agentId1);
		const agent2 = this.agents.get(agentId2);

		if (!agent1 || !agent2) {
			throw new Error("Both agents must exist to merge");
		}

		if (agent1.type !== agent2.type) {
			throw new Error("Cannot merge agents of different types");
		}

		const policy = this.policies.get(agent1.type);
		if (policy && !policy.allowMerge) {
			throw new Error(`Merge not allowed for agent type: ${agent1.type}`);
		}

		// Merge capabilities
		const mergedCapabilities = Array.from(new Set([...agent1.capabilities, ...agent2.capabilities]));

		// Merge configuration
		const mergedConfig = { ...agent1.configuration, ...agent2.configuration };

		const mergedId = this.generateAgentId();
		const merged: AgentSpec = {
			id: mergedId,
			type: agent1.type,
			capabilities: mergedCapabilities,
			configuration: mergedConfig,
		};

		this.agents.set(mergedId, merged);

		// Remove original agents
		this.agents.delete(agentId1);
		this.agents.delete(agentId2);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "merge",
			agentId: mergedId,
			targetId: agentId2,
			parameters: { source1: agentId1, source2: agentId2 },
			status: "completed",
			createdAt: Date.now(),
		});

		return merged;
	}

	/**
	 * Split agent into multiple agents
	 */
	split(agentId: string, splits: Array<{ capabilities: readonly string[]; configuration: Readonly<Record<string, unknown>> }>): AgentSpec[] {
		const source = this.agents.get(agentId);
		if (!source) {
			throw new Error(`Agent not found: ${agentId}`);
		}

		const policy = this.policies.get(source.type);
		if (policy && !policy.allowSplit) {
			throw new Error(`Split not allowed for agent type: ${source.type}`);
		}

		const splitAgents: AgentSpec[] = [];

		for (const split of splits) {
			const splitId = this.generateAgentId();
			const splitAgent: AgentSpec = {
				id: splitId,
				type: source.type,
				capabilities: split.capabilities,
				configuration: { ...source.configuration, ...split.configuration },
				parentId: agentId,
			};

			this.agents.set(splitId, splitAgent);
			splitAgents.push(splitAgent);
		}

		// Remove original agent
		this.agents.delete(agentId);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "split",
			agentId,
			parameters: { splits: splits.length },
			status: "completed",
			createdAt: Date.now(),
		});

		return splitAgents;
	}

	/**
	 * Upgrade agent
	 */
	upgrade(agentId: string, upgrades: Partial<Omit<AgentSpec, "id">>): AgentSpec {
		const agent = this.agents.get(agentId);
		if (!agent) {
			throw new Error(`Agent not found: ${agentId}`);
		}

		const policy = this.policies.get(agent.type);
		if (policy && !policy.allowUpgrade) {
			throw new Error(`Upgrade not allowed for agent type: ${agent.type}`);
		}

		const upgraded: AgentSpec = {
			...agent,
			...upgrades,
		};

		this.agents.set(agentId, upgraded);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "upgrade",
			agentId,
			parameters: upgrades,
			status: "completed",
			createdAt: Date.now(),
		});

		return upgraded;
	}

	/**
	 * Specialize agent
	 */
	specialize(agentId: string, specialization: { capabilities: readonly string[]; configuration: Readonly<Record<string, unknown>> }): AgentSpec {
		const agent = this.agents.get(agentId);
		if (!agent) {
			throw new Error(`Agent not found: ${agentId}`);
		}

		const policy = this.policies.get(agent.type);
		if (policy && !policy.allowSpecialize) {
			throw new Error(`Specialize not allowed for agent type: ${agent.type}`);
		}

		const specialized: AgentSpec = {
			...agent,
			capabilities: specialization.capabilities,
			configuration: { ...agent.configuration, ...specialization.configuration },
		};

		this.agents.set(agentId, specialized);

		// Record operation
		this.recordOperation({
			id: this.generateOperationId(),
			type: "specialize",
			agentId,
			parameters: specialization,
			status: "completed",
			createdAt: Date.now(),
		});

		return specialized;
	}

	/**
	 * Get agent
	 */
	getAgent(agentId: string): AgentSpec | null {
		return this.agents.get(agentId) ?? null;
	}

	/**
	 * List all agents
	 */
	listAgents(): readonly AgentSpec[] {
		return Array.from(this.agents.values());
	}

	/**
	 * Remove agent
	 */
	removeAgent(agentId: string): boolean {
		return this.agents.delete(agentId);
	}

	/**
	 * Get operation history
	 */
	getOperations(agentId?: string): readonly AgentOperation[] {
		if (agentId) {
			return Array.from(this.operations.values()).filter((op) => op.agentId === agentId);
		}
		return Array.from(this.operations.values());
	}

	/**
	 * Record operation
	 */
	private recordOperation(operation: AgentOperation): void {
		this.operations.set(operation.id, operation);
	}

	/**
	 * Generate unique agent ID
	 */
	private generateAgentId(): string {
		const bytes = QuantumSafeCrypto.randomBytes(16);
		return `agent-${Array.from(bytes)
			.map((b) => b.toString(16).padStart(2, "0"))
			.join("")}`;
	}

	/**
	 * Generate unique operation ID
	 */
	private generateOperationId(): string {
		const bytes = QuantumSafeCrypto.randomBytes(16);
		return `op-${Array.from(bytes)
			.map((b) => b.toString(16).padStart(2, "0"))
			.join("")}`;
	}
}

