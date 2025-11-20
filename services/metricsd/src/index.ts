/**
 * Metrics Daemon (metricsd)
 *
 * Service for collecting and exposing system metrics
 */

import { SemanticMessageBus, SemanticMessageBuilder } from "@aios/ipc";
import type { SystemMetrics, AgentMetrics } from "./types.js";

/**
 * Metrics Daemon
 */
export class MetricsDaemon {
	private readonly messageBus: SemanticMessageBus;
	private readonly metrics: Map<string, SystemMetrics>;
	private readonly agentMetrics: Map<number, AgentMetrics>;

	constructor() {
		this.messageBus = new SemanticMessageBus();
		this.metrics = new Map();
		this.agentMetrics = new Map();
	}

	/**
	 * Start the daemon
	 */
	async start(): Promise<void> {
		// Subscribe to observability events
		this.messageBus.subscribe(
			{ intentType: "observability.metrics" },
			async (message) => {
				const payload = message.payload as Record<string, unknown>;
				const metrics: SystemMetrics = {
					cpuUsage: (payload.cpuUsage as number) ?? 0,
					memoryUsage: (payload.memoryUsage as number) ?? 0,
					networkThroughput: (payload.networkThroughput as number) ?? 0,
					ioThroughput: (payload.ioThroughput as number) ?? 0,
					activeAgents: (payload.activeAgents as number) ?? 0,
				};
				this.metrics.set("system", metrics);
			}
		);

		// Start metrics collection loop
		this.startCollectionLoop();
	}

	/**
	 * Start metrics collection loop
	 */
	private startCollectionLoop(): void {
		setInterval(async () => {
			// Collect metrics from kernel observability system via kernel-bridge service
			const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
			
			try {
				const response = await fetch(`${kernelBridgeUrl}/api/kernel/observability/metrics`, {
					method: "GET",
				});
				
				if (response.ok) {
					const kernelMetrics = (await response.json()) as {
						cpuUsage?: number;
						memoryUsage?: number;
						networkThroughput?: number;
						ioThroughput?: number;
						activeAgents?: number;
					};
					
					const metrics: SystemMetrics = {
						cpuUsage: kernelMetrics.cpuUsage ?? 0,
						memoryUsage: kernelMetrics.memoryUsage ?? 0,
						networkThroughput: kernelMetrics.networkThroughput ?? 0,
						ioThroughput: kernelMetrics.ioThroughput ?? 0,
						activeAgents: kernelMetrics.activeAgents ?? this.agentMetrics.size,
					};
					
					this.metrics.set("system", metrics);
					
					// Publish metrics via semantic IPC
					const metricsMessage = SemanticMessageBuilder.create(
						"metricsd",
						"*",
						{
							type: "observability.metrics",
							action: "notify",
							constraints: {},
							context: {},
							priority: 1,
						},
						{
							cpuUsage: metrics.cpuUsage,
							memoryUsage: metrics.memoryUsage,
							networkThroughput: metrics.networkThroughput,
							ioThroughput: metrics.ioThroughput,
							activeAgents: metrics.activeAgents,
						}
					);
					this.messageBus.publish(metricsMessage);
				} else {
					// Fallback: use default values if kernel query fails
					const metrics: SystemMetrics = {
						cpuUsage: 0,
						memoryUsage: 0,
						networkThroughput: 0,
						ioThroughput: 0,
						activeAgents: this.agentMetrics.size,
					};
					this.metrics.set("system", metrics);
				}
			} catch (error) {
				console.error("Failed to collect metrics from kernel:", error);
				// Use default values on error
				const metrics: SystemMetrics = {
					cpuUsage: 0,
					memoryUsage: 0,
					networkThroughput: 0,
					ioThroughput: 0,
					activeAgents: this.agentMetrics.size,
				};
				this.metrics.set("system", metrics);
			}
		}, 5000); // Every 5 seconds
	}

	/**
	 * Get system metrics
	 */
	getSystemMetrics(): SystemMetrics | undefined {
		return this.metrics.get("system");
	}

	/**
	 * Get agent metrics
	 */
	getAgentMetrics(agentId: number): AgentMetrics | undefined {
		return this.agentMetrics.get(agentId);
	}
}

