/**
 * Memory Fabric - Unified cognitive memory system
 * Combines vector stores, embeddings, and semantic indexes
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { MemoryEntry, MemoryQuery, MemoryGraph, MemoryVersion } from "../types.js";
import { VectorStore, type VectorStoreConfig } from "../vector/index.js";
import { SparseSemanticIndex } from "../index/index.js";
import { EmbeddingGenerator } from "../embedding/index.js";

export interface MemoryFabricConfig {
	readonly vectorStore: VectorStoreConfig;
	readonly enableVersioning: boolean;
	readonly enableEncryption: boolean;
	readonly distributed: boolean;
}

/**
 * Memory Fabric
 * Unified interface for cognitive memory operations
 */
export class MemoryFabric {
	private readonly vectorStore: VectorStore;
	private readonly semanticIndex: SparseSemanticIndex;
	private readonly embeddingGenerator: EmbeddingGenerator;
	private readonly config: MemoryFabricConfig;
	private readonly versions = new Map<string, MemoryVersion[]>();

	constructor(config: MemoryFabricConfig) {
		this.config = config;
		this.vectorStore = new VectorStore(config.vectorStore);
		this.semanticIndex = new SparseSemanticIndex();
		this.embeddingGenerator = new EmbeddingGenerator({
			dimensions: config.vectorStore.dimensions,
			algorithm: "semantic-v1",
		});
	}

	/**
	 * Store memory entry
	 */
	async store(
		agentId: string,
		content: Uint8Array,
		metadata: Readonly<Record<string, unknown>> = {},
		accessLevel: MemoryEntry["accessLevel"] = "private",
	): Promise<string> {
		const id = this.generateId();
		const embedding = await this.embeddingGenerator.generate(content);
		const timestamp = Date.now();

		// Encrypt if enabled
		let encryptedContent = content;
		if (this.config.enableEncryption) {
			const key = QuantumSafeCrypto.randomBytes(32);
			encryptedContent = QuantumSafeCrypto.encrypt(content, key);
		}

		const entry: MemoryEntry = {
			id,
			agentId,
			content: encryptedContent,
			embedding,
			metadata,
			timestamp,
			version: 1,
			accessLevel,
		};

		await this.vectorStore.add(entry);

		// Create semantic graph nodes and edges
		const graph = this.createMemoryGraph(entry);
		this.semanticIndex.indexEntry(entry, graph.nodes, graph.edges);

		// Version if enabled
		if (this.config.enableVersioning) {
			this.versions.set(id, [
				{
					version: 1,
					entryId: id,
					timestamp,
					changes: { created: true },
				},
			]);
		}

		return id;
	}

	/**
	 * Query memory fabric
	 */
	async query(query: MemoryQuery): Promise<readonly MemoryEntry[]> {
		return this.vectorStore.query(query);
	}

	/**
	 * Retrieve memory entry by ID
	 */
	async retrieve(entryId: string): Promise<MemoryEntry | null> {
		return this.vectorStore.get(entryId);
	}

	/**
	 * Update memory entry
	 */
	async update(
		entryId: string,
		updates: {
			content?: Uint8Array;
			metadata?: Readonly<Record<string, unknown>>;
			accessLevel?: MemoryEntry["accessLevel"];
		},
	): Promise<boolean> {
		const existing = await this.retrieve(entryId);
		if (!existing) {
			return false;
		}

		const newVersion = existing.version + 1;
		const content = updates.content ?? existing.content;
		const embedding = updates.content
			? await this.embeddingGenerator.generate(content)
			: existing.embedding;

		let encryptedContent = content;
		if (this.config.enableEncryption) {
			const key = QuantumSafeCrypto.randomBytes(32);
			encryptedContent = QuantumSafeCrypto.encrypt(content, key);
		}

		const updated: MemoryEntry = {
			...existing,
			content: encryptedContent,
			embedding,
			metadata: { ...existing.metadata, ...updates.metadata },
			accessLevel: updates.accessLevel ?? existing.accessLevel,
			version: newVersion,
		};

		// Remove old entry
		this.vectorStore.remove(entryId);
		this.semanticIndex.removeEntry(entryId);

		// Add updated entry
		await this.vectorStore.add(updated);

		// Update semantic index
		const graph = this.createMemoryGraph(updated);
		this.semanticIndex.indexEntry(updated, graph.nodes, graph.edges);

		// Record version
		if (this.config.enableVersioning) {
			const versions = this.versions.get(entryId) ?? [];
			versions.push({
				version: newVersion,
				entryId,
				timestamp: Date.now(),
				changes: updates,
			});
			this.versions.set(entryId, versions);
		}

		return true;
	}

	/**
	 * Delete memory entry
	 */
	async delete(entryId: string): Promise<boolean> {
		const removed = this.vectorStore.remove(entryId);
		if (removed) {
			this.semanticIndex.removeEntry(entryId);
			this.versions.delete(entryId);
		}
		return removed;
	}

	/**
	 * Get memory graph for an entry
	 */
	getMemoryGraph(entryId: string): MemoryGraph | null {
		const index = this.semanticIndex.getIndex(entryId);
		if (!index) {
			return null;
		}

		return {
			nodes: index.nodes,
			edges: index.edges,
		};
	}

	/**
	 * Get version history for an entry
	 */
	getVersions(entryId: string): readonly MemoryVersion[] {
		return this.versions.get(entryId) ?? [];
	}

	/**
	 * Get memory fabric statistics
	 */
	getStats(): { size: number; capacity: number; dimensions: number } {
		return this.vectorStore.getStats();
	}

	/**
	 * Generate unique ID
	 */
	private generateId(): string {
		const bytes = QuantumSafeCrypto.randomBytes(16);
		return Array.from(bytes)
			.map((b) => b.toString(16).padStart(2, "0"))
			.join("");
	}

	/**
	 * Create memory graph from entry
	 */
	private createMemoryGraph(entry: MemoryEntry): MemoryGraph {
		// Simplified graph creation
		// In production, use actual semantic analysis
		const nodeId = `node-${entry.id}`;
		const node: MemoryNode = {
			id: nodeId,
			entryId: entry.id,
			type: "concept",
			weight: 1.0,
		};

		return {
			nodes: [node],
			edges: [],
		};
	}
}

