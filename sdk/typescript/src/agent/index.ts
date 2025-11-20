/**
 * Agent management API
 */

export class AgentClient {
	/**
	 * Spawn agent
	 */
	async spawn(config: AgentConfig): Promise<number> {
		// TODO: Spawn agent via agentsupervisor service
		return 0;
	}

	/**
	 * Get agent status
	 */
	async status(agentId: number): Promise<AgentStatus> {
		// TODO: Get agent status
		throw new Error("Not implemented");
	}
}

export interface AgentConfig {
	memorySize: number;
	priority: number;
}

export interface AgentStatus {
	agentId: number;
	state: "running" | "stopped" | "failed";
}

