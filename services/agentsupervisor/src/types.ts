/**
 * Agent Supervisor Service Types
 */

export interface AgentImage {
	readonly agentId: string;
	readonly imagePath: string;
	readonly loadedAt: number;
	readonly signature?: Uint8Array;
	readonly size: number;
	readonly checksum: Uint8Array;
}

export interface AgentStatus {
	agentId: string;
	status: "loaded" | "running" | "stopped" | "failed";
	startedAt?: number;
	resourceUsage: AgentResourceUsage;
}

export interface AgentResourceUsage {
	readonly cpu: number; // CPU usage percentage
	readonly memory: number; // Memory usage in bytes
	readonly network: number; // Network usage in bytes
	readonly io: number; // I/O usage in bytes
}

