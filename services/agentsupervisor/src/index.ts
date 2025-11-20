/**
 * Agent Supervisor Service (agentsupervisor)
 *
 * Privileged system service for agent management:
 * - Agent image loading
 * - Agent signature verification
 * - Agent lifecycle management
 * - Agent monitoring
 * - Agent resource tracking
 */

import { SemanticMessageBus } from "@aios/ipc";
import type { AgentImage, AgentStatus, AgentResourceUsage } from "./types.js";
import { readFile } from "fs/promises";
import { createHash } from "crypto";

/**
 * Agent Supervisor Service
 *
 * Manages agent lifecycle, monitoring, and resource tracking
 */
export class AgentSupervisorService {
	private readonly messageBus: SemanticMessageBus;
	private readonly agents: Map<string, AgentStatus>;
	private readonly agentImages: Map<string, AgentImage>;
	private readonly resourceUsage: Map<string, AgentResourceUsage>;

	constructor() {
		this.messageBus = new SemanticMessageBus();
		this.agents = new Map();
		this.agentImages = new Map();
		this.resourceUsage = new Map();
	}

	/**
	 * Start the service
	 */
	async start(): Promise<void> {
		// Register IPC handlers using MessageFilter (non-blocking)
		try {
			this.messageBus.subscribe(
				{ intentType: "agent.load" },
				async (message) => {
					const payload = message.payload as {
						imagePath?: string;
						agentId?: string;
						signature?: Uint8Array;
					};
					if (payload.imagePath && payload.agentId) {
						const image = await this.loadAgentImage(
							payload.imagePath,
							payload.agentId,
							payload.signature
						);
						// Note: subscribe callback doesn't return, would need to send response via message
					}
				}
			);
		} catch (error) {
			// IPC subscription is optional - don't fail if it's not available
			console.warn("Failed to subscribe to IPC events:", error);
		}

		try {
			this.messageBus.subscribe(
				{ intentType: "agent.start" },
				async (message) => {
					const payload = message.payload as { agentId?: string };
					if (payload.agentId) {
						await this.startAgent(payload.agentId);
					}
				}
			);

			this.messageBus.subscribe(
				{ intentType: "agent.stop" },
				async (message) => {
					const payload = message.payload as { agentId?: string };
					if (payload.agentId) {
						await this.stopAgent(payload.agentId);
					}
				}
			);

			this.messageBus.subscribe(
				{ intentType: "agent.status" },
				async (message) => {
					const payload = message.payload as { agentId?: string };
					if (payload.agentId) {
						const status = this.getAgentStatus(payload.agentId);
						// Note: subscribe callback doesn't return, would need to send response via message
					}
				}
			);
		} catch (error) {
			console.warn("Failed to subscribe to additional IPC events:", error);
		}

		// Start monitoring loop
		this.startMonitoringLoop();
	}

	/**
	 * Load agent image
	 */
	async loadAgentImage(
		imagePath: string,
		agentId: string,
		signature?: Uint8Array
	): Promise<AgentImage> {
		// Verify signature if provided
		if (signature) {
			const isValid = await this.verifySignature(imagePath, signature);
			if (!isValid) {
				throw new Error(`Invalid signature for agent image: ${agentId}`);
			}
		}

		// Load image from filesystem
		// Read agent image file from filesystem
		const imageData = await readFile(imagePath);
		const size = imageData.length;
		
		// Compute checksum (SHA-256)
		const checksum = createHash("sha256").update(imageData).digest();
		
		const image: AgentImage = {
			agentId,
			imagePath,
			loadedAt: Date.now(),
			signature,
			size,
			checksum: new Uint8Array(checksum),
		};

		// Store image
		this.agentImages.set(agentId, image);

		// Initialize agent status
		this.agents.set(agentId, {
			agentId,
			status: "loaded",
			startedAt: undefined,
			resourceUsage: {
				cpu: 0,
				memory: 0,
				network: 0,
				io: 0,
			},
		});

		return image;
	}

	/**
	 * Verify agent image signature
	 */
	private async verifySignature(imagePath: string, signature: Uint8Array): Promise<boolean> {
		// Verify signature using public key from identityd service
		// Query identityd service for agent's public key
		const identitydUrl = process.env["IDENTITYD_URL"] || "http://127.0.0.1:9003";
		
		try {
			const response = await fetch(`${identitydUrl}/api/identity/${imagePath}`, {
				method: "GET",
			});
			
			if (response.ok) {
				const identity = (await response.json()) as { publicKey?: string };
				if (identity.publicKey) {
					// Use PQC daemon to verify signature
					const pqcdUrl = process.env["PQCD_URL"] || "http://127.0.0.1:9004";
					const imageData = await readFile(imagePath);
					const verifyResponse = await fetch(`${pqcdUrl}/api/pqc/verify`, {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({
							algorithm: "CRYSTALS-Dilithium",
							data: Array.from(imageData),
							publicKey: Array.from(Buffer.from(identity.publicKey, "base64")),
							signature: Array.from(signature),
						}),
					});
					
					if (verifyResponse.ok) {
						const result = (await verifyResponse.json()) as { success: boolean };
						return result.success;
					}
				}
			}
		} catch (error) {
			console.error("Signature verification failed:", error);
		}
		
		return false;
	}

	/**
	 * Start agent
	 */
	async startAgent(agentId: string): Promise<void> {
		const status = this.agents.get(agentId);
		if (!status) {
			throw new Error(`Agent not loaded: ${agentId}`);
		}

		if (status.status === "running") {
			return; // Already running
		}

		// Update status
		status.status = "running";
		status.startedAt = Date.now();

		// Publish agent started event via semantic IPC
		await this.messageBus.publish({
			id: `event-${Date.now()}`,
			from: "agentsupervisor",
			to: "all",
			intent: {
				type: "agent.started",
				priority: 1,
				context: {},
			},
			payload: { agentId, status: "running" },
			timestamp: Date.now(),
		});
	}

	/**
	 * Stop agent
	 */
	async stopAgent(agentId: string): Promise<void> {
		const status = this.agents.get(agentId);
		if (!status) {
			throw new Error(`Agent not found: ${agentId}`);
		}

		if (status.status === "stopped") {
			return; // Already stopped
		}

		// Update status
		status.status = "stopped";
		status.startedAt = undefined;

		// Publish agent stopped event via semantic IPC
		await this.messageBus.publish({
			id: `event-${Date.now()}`,
			from: "agentsupervisor",
			to: "all",
			intent: {
				type: "agent.stopped",
				priority: 1,
				context: {},
			},
			payload: { agentId, status: "stopped" },
			timestamp: Date.now(),
		});
	}

	/**
	 * Get agent status
	 */
	getAgentStatus(agentId: string): AgentStatus | undefined {
		return this.agents.get(agentId);
	}

	/**
	 * Get all agent IDs
	 */
	getAllAgentIds(): readonly string[] {
		return Array.from(this.agents.keys());
	}

	/**
	 * Remove agent completely (kill)
	 */
	removeAgent(agentId: string): boolean {
		return this.agents.delete(agentId);
	}

	/**
	 * Update resource usage
	 */
	updateResourceUsage(agentId: string, usage: AgentResourceUsage): void {
		const status = this.agents.get(agentId);
		if (status) {
			status.resourceUsage = usage;
			this.resourceUsage.set(agentId, usage);
		}
	}

	/**
	 * Start monitoring loop
	 */
	private startMonitoringLoop(): void {
		setInterval(async () => {
			// Monitor all running agents
			const agentIds = Array.from(this.agents.keys());
			for (const agentId of agentIds) {
				const status = this.agents.get(agentId);
				if (status && status.status === "running") {
					// Query kernel for actual resource usage via kernel-bridge service
					const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
					
					try {
						const response = await fetch(`${kernelBridgeUrl}/api/kernel/agent/${agentId}/resources`, {
							method: "GET",
						});
						
						if (response.ok) {
							const resources = (await response.json()) as {
								cpu?: number;
								memory?: number;
								gpu?: number;
							};
							
							this.updateResourceUsage(agentId, {
								cpu: resources.cpu ?? 0,
								memory: resources.memory ?? 0,
								gpu: resources.gpu ?? 0,
							});
						} else {
							// Fallback: use default values if kernel query fails
							this.updateResourceUsage(agentId, {
								cpu: 0,
								memory: 0,
								gpu: 0,
							});
						}
					} catch (error) {
						console.error(`Failed to query resources for agent ${agentId}:`, error);
						this.updateResourceUsage(agentId, {
							cpu: 0,
							memory: 0,
							gpu: 0,
						});
					}
						network: Math.random() * 1024 * 1024, // Random KB
						io: Math.random() * 1024 * 1024, // Random KB
					});
				}
			}
		}, 5000); // Every 5 seconds
	}
}

