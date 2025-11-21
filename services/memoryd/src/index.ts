/**
 * Memory Fabric Service (/svc/memoryd)
 *
 * Privileged userland service that implements semantic memory fabric.
 * Uses kernel primitives (FrameAlloc, PageMap, AgentPoolAlloc) for low-level operations.
 * Provides high-level semantic memory operations via IPC.
 */

import { SemanticMessageBus } from "@aios/ipc";
import { MemoryFabric } from "@aios/memory";

/**
 * Memory Fabric Service
 *
 * Provides semantic memory operations:
 * - Store memory entries with embeddings
 * - Query by semantic similarity
 * - Memory versioning
 * - Memory graphs
 */
export class MemoryFabricService {
	private readonly memoryFabric: MemoryFabric;
	private readonly messageBus: SemanticMessageBus;

	constructor() {
		this.memoryFabric = new MemoryFabric({
			vectorStore: {
				dimensions: 384,
				indexType: "flat",
				maxEntries: 100000,
			},
			enableVersioning: true,
			enableEncryption: true,
			distributed: false,
		});

		this.messageBus = new SemanticMessageBus();
	}

	/**
	 * Start the service
	 */
	async start(): Promise<void> {
		// Register IPC handlers using MessageFilter
		this.messageBus.subscribe({ intentType: "memory.store" }, async (message) => {
			const payload = message.payload as {
				agentId?: string;
				content?: Uint8Array;
				metadata?: Record<string, unknown>;
			};
			if (payload.agentId && payload.content) {
				await this.store(payload.agentId, payload.content, payload.metadata);
			}
		});

		this.messageBus.subscribe({ intentType: "memory.query" }, async (message) => {
			const query = message.payload as {
				query?: string;
				limit?: number;
				threshold?: number;
			};
			if (query.query) {
				await this.query({
					query: query.query,
					limit: query.limit,
					threshold: query.threshold,
				});
			}
		});

		// Register kernel memory fabric integration handlers
		this.messageBus.subscribe({ intentType: "memory.fabric.create_region" }, async (message) => {
			// Call kernel memory fabric syscall via kernel-bridge service
			const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
			const payload = message.payload as { agentId?: string; size?: number; regionType?: string };

			if (payload.agentId && payload.size) {
				try {
					const response = await fetch(
						`${kernelBridgeUrl}/api/kernel/memory/fabric/create_region`,
						{
							method: "POST",
							headers: { "Content-Type": "application/json" },
							body: JSON.stringify({
								agentId: payload.agentId,
								size: payload.size,
								regionType: payload.regionType || "ephemeral",
							}),
						}
					);

					if (!response.ok) {
						console.error("Failed to create memory fabric region:", response.statusText);
					}
				} catch (_error) {
					console.error("Error creating memory fabric region:", error);
				}
			}
		});

		this.messageBus.subscribe(
			{ intentType: "memory.fabric.create_shared_page" },
			async (message) => {
				// Call kernel memory fabric syscall via kernel-bridge service
				const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
				const payload = message.payload as { agents?: string[] };

				if (payload.agents && payload.agents.length > 0) {
					try {
						const response = await fetch(
							`${kernelBridgeUrl}/api/kernel/memory/fabric/create_shared_page`,
							{
								method: "POST",
								headers: { "Content-Type": "application/json" },
								body: JSON.stringify({ agents: payload.agents }),
							}
						);

						if (!response.ok) {
							console.error("Failed to create shared page:", response.statusText);
						}
					} catch (_error) {
						console.error("Error creating shared page:", error);
					}
				}
			}
		);

		this.messageBus.subscribe({ intentType: "memory.fabric.tag_region" }, async () => {
			// Tag memory region
		});

		this.messageBus.subscribe({ intentType: "memory.fabric.create_lease" }, async () => {
			// Create memory lease
		});

		// Start periodic cleanup
		this.startPeriodicCleanup();
	}

	/**
	 * Periodic cleanup of expired memory
	 */
	private startPeriodicCleanup(): void {
		setInterval(() => {
			// Cleanup expired memory entries
			// Implementation depends on MemoryFabric API
		}, 60000); // Every minute
	}

	/**
	 * Store memory entry
	 *
	 * Called via IPC, not kernel syscall
	 */
	async store(
		agentId: string,
		content: Uint8Array,
		metadata: Readonly<Record<string, unknown>> = {}
	): Promise<string> {
		// Uses kernel AgentPoolAlloc for memory allocation
		// Implements semantic memory fabric on top
		return this.memoryFabric.store(agentId, content, metadata);
	}

	/**
	 * Query memory fabric
	 *
	 * Called via IPC, not kernel syscall
	 */
	async query(query: {
		query: string;
		limit?: number;
		threshold?: number;
	}): Promise<readonly unknown[]> {
		return this.memoryFabric.query(query);
	}
}
