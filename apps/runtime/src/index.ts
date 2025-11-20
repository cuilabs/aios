/**
 * AIOS Main Runtime
 * The AIOS operating system runtime that integrates all layers
 */

import {
	EnvironmentManager,
	PipelineManager,
	ToolchainManager,
	WorkflowManager,
} from "@aios/application";
import { AgentSupervisor, ContextAllocator, PlanningManager } from "@aios/cognitive";
import type { AgentSupervisor as AgentSupervisorType } from "@aios/cognitive";
import { SemanticMessageBus } from "@aios/ipc";
import type { SemanticMessageBus as SemanticMessageBusType } from "@aios/ipc";
import {
	ResourceIsolation,
	SecureEnclaveManager,
	createDeterministicScheduler,
} from "@aios/kernel";
import { MemoryFabric } from "@aios/memory";
import { AgentOrchestrator } from "@aios/orchestration";
import {
	BehavioralAnalyzer,
	CapabilityManager,
	IdentityManager,
	TrustGraphManager,
} from "@aios/security";
import type { TrustGraphManager as TrustGraphManagerType } from "@aios/security";

export interface AIOSRuntimeConfig {
	readonly kernelVersion: string;
	readonly quantumSafeEnabled: boolean;
	readonly maxAgents: number;
	readonly memoryConfig: {
		readonly dimensions: number;
		readonly maxEntries: number;
		readonly enableVersioning: boolean;
		readonly enableEncryption: boolean;
	};
}

/**
 * AIOS Runtime
 * Main operating system runtime integrating all layers
 */
export class AIOSRuntime {
	private readonly config: AIOSRuntimeConfig;
	private readonly scheduler = createDeterministicScheduler("deadline");
	private readonly resourceIsolation: ResourceIsolation;
	private readonly secureEnclaves: SecureEnclaveManager;
	private readonly memoryFabric: MemoryFabric;
	private readonly messageBus: SemanticMessageBus;
	private readonly identityManager: IdentityManager;
	private readonly capabilityManager: CapabilityManager;
	private readonly behavioralAnalyzer: BehavioralAnalyzer;
	private readonly trustGraph: TrustGraphManager;
	private readonly contextAllocator: ContextAllocator;
	private readonly planningManager: PlanningManager;
	private readonly agentSupervisor: AgentSupervisor;
	private readonly orchestrator: AgentOrchestrator;
	private readonly workflowManager: WorkflowManager;
	private readonly pipelineManager: PipelineManager;
	private readonly environmentManager: EnvironmentManager;
	private readonly toolchainManager: ToolchainManager;

	constructor(config: AIOSRuntimeConfig) {
		this.config = config;

		// Initialize kernel layer
		this.resourceIsolation = new ResourceIsolation({
			maxMemoryBytes: 1024 * 1024 * 1024, // 1GB
			maxCpuPercent: 100,
			maxNetworkBandwidthBytes: 100 * 1024 * 1024, // 100MB/s
			maxConcurrentOperations: 100,
		});

		this.secureEnclaves = new SecureEnclaveManager();

		// Initialize memory fabric
		this.memoryFabric = new MemoryFabric({
			vectorStore: {
				dimensions: config.memoryConfig.dimensions,
				indexType: "flat",
				maxEntries: config.memoryConfig.maxEntries,
			},
			enableVersioning: config.memoryConfig.enableVersioning,
			enableEncryption: config.memoryConfig.enableEncryption,
			distributed: false,
		});

		// Initialize IPC
		this.messageBus = new SemanticMessageBus();

		// Initialize security
		this.identityManager = new IdentityManager();
		this.capabilityManager = new CapabilityManager(this.identityManager);
		this.behavioralAnalyzer = new BehavioralAnalyzer();
		this.trustGraph = new TrustGraphManager();

		// Initialize cognitive runtime
		this.contextAllocator = new ContextAllocator();
		this.planningManager = new PlanningManager();
		this.agentSupervisor = new AgentSupervisor({
			maxConcurrentAgents: config.maxAgents,
			contextAllocator: this.contextAllocator,
			planningManager: this.planningManager,
		});

		// Initialize orchestration
		this.orchestrator = new AgentOrchestrator();

		// Initialize application layer
		this.workflowManager = new WorkflowManager();
		this.pipelineManager = new PipelineManager();
		this.environmentManager = new EnvironmentManager();
		this.toolchainManager = new ToolchainManager();
	}

	/**
	 * Get runtime information
	 */
	getInfo(): Readonly<{
		config: AIOSRuntimeConfig;
		kernel: {
			scheduler: { queueLength: number };
			isolation: { contexts: number };
			enclaves: number;
		};
		memory: { stats: { size: number; capacity: number; dimensions: number } };
		ipc: { metrics: ReturnType<SemanticMessageBusType["getMetrics"]> };
		security: {
			identities: number;
			trustGraph: ReturnType<TrustGraphManagerType["getTrustGraph"]>;
		};
		cognitive: { stats: ReturnType<AgentSupervisorType["getStats"]> };
		orchestration: { agents: number };
		application: {
			workflows: number;
			pipelines: number;
			environments: number;
			toolchains: number;
		};
	}> {
		return {
			config: this.config,
			kernel: {
				scheduler: { queueLength: this.scheduler.getQueueLength() },
				isolation: { contexts: this.resourceIsolation.getContextCount?.() ?? 0 },
				enclaves: this.secureEnclaves.listEnclaves().length,
			},
			memory: { stats: this.memoryFabric.getStats() },
			ipc: { metrics: this.messageBus.getMetrics() },
			security: {
				identities: this.identityManager.listIdentities?.()?.length ?? 0,
				trustGraph: this.trustGraph.getTrustGraph(),
			},
			cognitive: { stats: this.agentSupervisor.getStats() },
			orchestration: { agents: this.orchestrator.listAgents().length },
			application: {
				workflows: this.workflowManager.list().length,
				pipelines: this.pipelineManager.list().length,
				environments: this.environmentManager.list().length,
				toolchains: this.toolchainManager.list().length,
			},
		};
	}

	/**
	 * Get kernel scheduler
	 */
	getScheduler() {
		return this.scheduler;
	}

	/**
	 * Get resource isolation
	 */
	getResourceIsolation() {
		return this.resourceIsolation;
	}

	/**
	 * Get secure enclaves
	 */
	getSecureEnclaves() {
		return this.secureEnclaves;
	}

	/**
	 * Get memory fabric
	 */
	getMemoryFabric() {
		return this.memoryFabric;
	}

	/**
	 * Get message bus
	 */
	getMessageBus() {
		return this.messageBus;
	}

	/**
	 * Get identity manager
	 */
	getIdentityManager() {
		return this.identityManager;
	}

	/**
	 * Get capability manager
	 */
	getCapabilityManager() {
		return this.capabilityManager;
	}

	/**
	 * Get behavioral analyzer
	 */
	getBehavioralAnalyzer() {
		return this.behavioralAnalyzer;
	}

	/**
	 * Get trust graph
	 */
	getTrustGraph() {
		return this.trustGraph;
	}

	/**
	 * Get context allocator
	 */
	getContextAllocator() {
		return this.contextAllocator;
	}

	/**
	 * Get planning manager
	 */
	getPlanningManager() {
		return this.planningManager;
	}

	/**
	 * Get agent supervisor
	 */
	getAgentSupervisor() {
		return this.agentSupervisor;
	}

	/**
	 * Get orchestrator
	 */
	getOrchestrator() {
		return this.orchestrator;
	}

	/**
	 * Get workflow manager
	 */
	getWorkflowManager() {
		return this.workflowManager;
	}

	/**
	 * Get pipeline manager
	 */
	getPipelineManager() {
		return this.pipelineManager;
	}

	/**
	 * Get environment manager
	 */
	getEnvironmentManager() {
		return this.environmentManager;
	}

	/**
	 * Get toolchain manager
	 */
	getToolchainManager() {
		return this.toolchainManager;
	}
}

/**
 * Create default AIOS runtime
 */
export function createAIOSRuntime(config?: Partial<AIOSRuntimeConfig>): AIOSRuntime {
	const defaultConfig: AIOSRuntimeConfig = {
		kernelVersion: "0.1.0",
		quantumSafeEnabled: true,
		maxAgents: 1000,
		memoryConfig: {
			dimensions: 384,
			maxEntries: 100000,
			enableVersioning: true,
			enableEncryption: true,
		},
	};

	return new AIOSRuntime({ ...defaultConfig, ...config });
}

// Main entry point
if (import.meta.url === `file://${process.argv[1]}`) {
	const runtime = createAIOSRuntime();
	console.log("AIOS Runtime initialized");
	console.log(JSON.stringify(runtime.getInfo(), null, 2));
}
