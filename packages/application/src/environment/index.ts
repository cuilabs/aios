/**
 * Environment manager
 * Manages agent execution environments
 */

import type { Environment } from "../types.js";

/**
 * Environment manager
 * Creates and manages isolated environments for agent execution
 */
export class EnvironmentManager {
	private readonly environments = new Map<string, Environment>();

	/**
	 * Create environment
	 */
	create(
		name: string,
		agents: readonly string[] = [],
		resources: Readonly<Record<string, unknown>> = {},
		configuration: Readonly<Record<string, unknown>> = {},
	): Environment {
		const environmentId = this.generateEnvironmentId();

		const environment: Environment = {
			id: environmentId,
			name,
			agents,
			resources,
			configuration,
		};

		this.environments.set(environmentId, environment);
		return environment;
	}

	/**
	 * Get environment
	 */
	get(environmentId: string): Environment | null {
		return this.environments.get(environmentId) ?? null;
	}

	/**
	 * Add agent to environment
	 */
	addAgent(environmentId: string, agentId: string): boolean {
		const environment = this.environments.get(environmentId);
		if (!environment) {
			return false;
		}

		if (environment.agents.includes(agentId)) {
			return false;
		}

		this.environments.set(environmentId, {
			...environment,
			agents: [...environment.agents, agentId],
		});

		return true;
	}

	/**
	 * Remove agent from environment
	 */
	removeAgent(environmentId: string, agentId: string): boolean {
		const environment = this.environments.get(environmentId);
		if (!environment) {
			return false;
		}

		this.environments.set(environmentId, {
			...environment,
			agents: environment.agents.filter((id) => id !== agentId),
		});

		return true;
	}

	/**
	 * Update environment resources
	 */
	updateResources(environmentId: string, resources: Readonly<Record<string, unknown>>): boolean {
		const environment = this.environments.get(environmentId);
		if (!environment) {
			return false;
		}

		this.environments.set(environmentId, {
			...environment,
			resources: { ...environment.resources, ...resources },
		});

		return true;
	}

	/**
	 * List environments
	 */
	list(): readonly Environment[] {
		return Array.from(this.environments.values());
	}

	/**
	 * Remove environment
	 */
	remove(environmentId: string): boolean {
		return this.environments.delete(environmentId);
	}

	/**
	 * Generate unique environment ID
	 */
	private generateEnvironmentId(): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `env-${timestamp}-${random}`;
	}
}

