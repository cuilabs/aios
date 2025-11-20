/**
 * Metrics types
 */

export interface SystemMetrics {
	readonly cpuUsage: number;
	readonly memoryUsage: number;
	readonly networkThroughput: number;
	readonly ioThroughput: number;
	readonly activeAgents: number;
}

export interface AgentMetrics {
	readonly agentId: number;
	readonly cpuCycles: number;
	readonly memoryAllocated: number;
	readonly networkBytes: number;
	readonly ioOperations: number;
}

