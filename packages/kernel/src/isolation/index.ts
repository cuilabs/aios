/**
 * Resource isolation for agent processes
 * Ensures agents cannot interfere with each other or the kernel
 */

import type { AgentResourceLimits } from "../types.js";

export interface ResourceUsage {
	readonly memoryBytes: number;
	readonly cpuPercent: number;
	readonly networkBandwidthBytes: number;
	readonly concurrentOperations: number;
}

export interface IsolationContext {
	readonly agentId: string;
	readonly limits: AgentResourceLimits;
	readonly currentUsage: ResourceUsage;
	readonly violations: readonly string[];
}

/**
 * Resource isolation manager
 * Enforces limits and prevents resource exhaustion attacks
 */
export class ResourceIsolation {
	private readonly contexts = new Map<string, IsolationContext>();
	private readonly limits: AgentResourceLimits;

	constructor(defaultLimits: AgentResourceLimits) {
		this.limits = defaultLimits;
	}

	/**
	 * Create isolation context for an agent
	 */
	createContext(agentId: string, customLimits?: Partial<AgentResourceLimits>): IsolationContext {
		const limits: AgentResourceLimits = {
			maxMemoryBytes: customLimits?.maxMemoryBytes ?? this.limits.maxMemoryBytes,
			maxCpuPercent: customLimits?.maxCpuPercent ?? this.limits.maxCpuPercent,
			maxNetworkBandwidthBytes:
				customLimits?.maxNetworkBandwidthBytes ?? this.limits.maxNetworkBandwidthBytes,
			maxConcurrentOperations:
				customLimits?.maxConcurrentOperations ?? this.limits.maxConcurrentOperations,
		};

		const context: IsolationContext = {
			agentId,
			limits,
			currentUsage: {
				memoryBytes: 0,
				cpuPercent: 0,
				networkBandwidthBytes: 0,
				concurrentOperations: 0,
			},
			violations: [],
		};

		this.contexts.set(agentId, context);
		return context;
	}

	/**
	 * Check if agent can allocate resources
	 */
	canAllocate(agentId: string, resource: Partial<ResourceUsage>): boolean {
		const context = this.contexts.get(agentId);
		if (!context) {
			return false;
		}

		const { currentUsage, limits } = context;

		if (resource.memoryBytes !== undefined) {
			if (currentUsage.memoryBytes + resource.memoryBytes > limits.maxMemoryBytes) {
				return false;
			}
		}

		if (resource.cpuPercent !== undefined) {
			if (currentUsage.cpuPercent + resource.cpuPercent > limits.maxCpuPercent) {
				return false;
			}
		}

		if (resource.networkBandwidthBytes !== undefined) {
			if (
				currentUsage.networkBandwidthBytes + resource.networkBandwidthBytes >
				limits.maxNetworkBandwidthBytes
			) {
				return false;
			}
		}

		if (resource.concurrentOperations !== undefined) {
			if (
				currentUsage.concurrentOperations + resource.concurrentOperations >
				limits.maxConcurrentOperations
			) {
				return false;
			}
		}

		return true;
	}

	/**
	 * Allocate resources for an agent
	 */
	allocate(agentId: string, resource: Partial<ResourceUsage>): boolean {
		if (!this.canAllocate(agentId, resource)) {
			const context = this.contexts.get(agentId);
			if (context) {
				const violations = [
					...context.violations,
					`Resource limit exceeded: ${JSON.stringify(resource)}`,
				];
				this.contexts.set(agentId, { ...context, violations });
			}
			return false;
		}

		const context = this.contexts.get(agentId);
		if (!context) {
			return false;
		}

		const currentUsage: ResourceUsage = {
			memoryBytes: context.currentUsage.memoryBytes + (resource.memoryBytes ?? 0),
			cpuPercent: context.currentUsage.cpuPercent + (resource.cpuPercent ?? 0),
			networkBandwidthBytes:
				context.currentUsage.networkBandwidthBytes + (resource.networkBandwidthBytes ?? 0),
			concurrentOperations:
				context.currentUsage.concurrentOperations + (resource.concurrentOperations ?? 0),
		};

		this.contexts.set(agentId, { ...context, currentUsage });
		return true;
	}

	/**
	 * Release resources for an agent
	 */
	release(agentId: string, resource: Partial<ResourceUsage>): void {
		const context = this.contexts.get(agentId);
		if (!context) {
			return;
		}

		const currentUsage: ResourceUsage = {
			memoryBytes: Math.max(0, context.currentUsage.memoryBytes - (resource.memoryBytes ?? 0)),
			cpuPercent: Math.max(0, context.currentUsage.cpuPercent - (resource.cpuPercent ?? 0)),
			networkBandwidthBytes: Math.max(
				0,
				context.currentUsage.networkBandwidthBytes - (resource.networkBandwidthBytes ?? 0)
			),
			concurrentOperations: Math.max(
				0,
				context.currentUsage.concurrentOperations - (resource.concurrentOperations ?? 0)
			),
		};

		this.contexts.set(agentId, { ...context, currentUsage });
	}

	/**
	 * Get isolation context for an agent
	 */
	getContext(agentId: string): IsolationContext | null {
		return this.contexts.get(agentId) ?? null;
	}

	/**
	 * Remove isolation context
	 */
	removeContext(agentId: string): boolean {
		return this.contexts.delete(agentId);
	}

	/**
	 * Get context count
	 */
	getContextCount(): number {
		return this.contexts.size;
	}
}
