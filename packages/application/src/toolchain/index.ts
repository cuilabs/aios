/**
 * Toolchain manager
 * Manages agent toolchains and tools
 */

import type { Tool, Toolchain } from "../types.js";

/**
 * Toolchain manager
 * Manages collections of tools for agent workflows
 */
export class ToolchainManager {
	private readonly toolchains = new Map<string, Toolchain>();
	private readonly tools = new Map<string, Tool>();

	/**
	 * Register tool
	 */
	registerTool(tool: Tool): void {
		this.tools.set(tool.id, tool);
	}

	/**
	 * Get tool
	 */
	getTool(toolId: string): Tool | null {
		return this.tools.get(toolId) ?? null;
	}

	/**
	 * Create toolchain
	 */
	create(
		name: string,
		toolIds: readonly string[],
		configuration: Readonly<Record<string, unknown>> = {}
	): Toolchain {
		const toolchainId = this.generateToolchainId();

		// Validate all tools exist
		const tools: Tool[] = [];
		for (const toolId of toolIds) {
			const tool = this.tools.get(toolId);
			if (!tool) {
				throw new Error(`Tool not found: ${toolId}`);
			}
			tools.push(tool);
		}

		const toolchain: Toolchain = {
			id: toolchainId,
			name,
			tools,
			configuration,
		};

		this.toolchains.set(toolchainId, toolchain);
		return toolchain;
	}

	/**
	 * Get toolchain
	 */
	get(toolchainId: string): Toolchain | null {
		return this.toolchains.get(toolchainId) ?? null;
	}

	/**
	 * Add tool to toolchain
	 */
	addTool(toolchainId: string, toolId: string): boolean {
		const toolchain = this.toolchains.get(toolchainId);
		const tool = this.tools.get(toolId);

		if (!toolchain || !tool) {
			return false;
		}

		if (toolchain.tools.some((t) => t.id === toolId)) {
			return false;
		}

		this.toolchains.set(toolchainId, {
			...toolchain,
			tools: [...toolchain.tools, tool],
		});

		return true;
	}

	/**
	 * Remove tool from toolchain
	 */
	removeTool(toolchainId: string, toolId: string): boolean {
		const toolchain = this.toolchains.get(toolchainId);
		if (!toolchain) {
			return false;
		}

		this.toolchains.set(toolchainId, {
			...toolchain,
			tools: toolchain.tools.filter((t) => t.id !== toolId),
		});

		return true;
	}

	/**
	 * List toolchains
	 */
	list(): readonly Toolchain[] {
		return Array.from(this.toolchains.values());
	}

	/**
	 * List tools
	 */
	listTools(): readonly Tool[] {
		return Array.from(this.tools.values());
	}

	/**
	 * Remove toolchain
	 */
	remove(toolchainId: string): boolean {
		return this.toolchains.delete(toolchainId);
	}

	/**
	 * Generate unique toolchain ID
	 */
	private generateToolchainId(): string {
		const timestamp = Date.now();
		const random = Math.random().toString(36).substring(2, 9);
		return `toolchain-${timestamp}-${random}`;
	}
}
