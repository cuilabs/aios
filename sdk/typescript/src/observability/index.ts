/**
 * Observability API
 */

export class ObservabilityClient {
	/**
	 * Get metrics
	 */
	async metrics(): Promise<SystemMetrics> {
		// TODO: Get system metrics
		return {
			cpuUsage: 0,
			memoryUsage: 0,
		};
	}
}

export interface SystemMetrics {
	cpuUsage: number;
	memoryUsage: number;
}
