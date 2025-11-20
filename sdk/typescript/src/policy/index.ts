/**
 * Policy API
 */

export class PolicyClient {
	/**
	 * Check policy
	 */
	async check(agentId: number, operation: string): Promise<boolean> {
		// TODO: Check policy via policy engine
		return true;
	}
}
