/**
 * Core kernel types and interfaces
 */

export interface AgentResourceLimits {
	readonly maxMemoryBytes: number;
	readonly maxCpuPercent: number;
	readonly maxNetworkBandwidthBytes: number;
	readonly maxConcurrentOperations: number;
}

export interface SecureEnclave {
	readonly id: string;
	readonly attestation: string;
	readonly memoryProtection: boolean;
	readonly encryptionAtRest: boolean;
}

export interface KernelContext {
	readonly kernelVersion: string;
	readonly quantumSafeEnabled: boolean;
	readonly secureEnclaves: readonly SecureEnclave[];
	readonly resourceLimits: AgentResourceLimits;
}

export interface ScheduledTask {
	readonly id: string;
	readonly priority: number;
	readonly deadline: number;
	readonly agentId: string;
	readonly taskType: "compute" | "memory" | "network" | "io";
}

export interface SchedulingPolicy {
	readonly name: string;
	readonly deterministic: boolean;
	readonly fairness: "fifo" | "priority" | "deadline" | "round-robin";
	readonly quantum: number;
}
