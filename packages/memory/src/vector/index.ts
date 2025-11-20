/**
 * Vector store for memory entries
 * Efficient storage and retrieval of high-dimensional vectors
 */

import { EmbeddingGenerator } from "../embedding/index.js";
import type { MemoryEntry, MemoryQuery } from "../types.js";

export interface VectorStoreConfig {
	readonly dimensions: number;
	readonly indexType: "flat" | "hnsw" | "ivf";
	readonly maxEntries: number;
}

/**
 * Vector store implementation
 * Stores and retrieves memory entries by semantic similarity
 */
export class VectorStore {
	private readonly entries = new Map<string, MemoryEntry>();
	private readonly embeddingGenerator: EmbeddingGenerator;
	private readonly config: VectorStoreConfig;

	constructor(config: VectorStoreConfig) {
		this.config = config;
		this.embeddingGenerator = new EmbeddingGenerator({
			dimensions: config.dimensions,
			algorithm: "semantic-v1",
		});
	}

	/**
	 * Add memory entry to vector store
	 */
	async add(entry: MemoryEntry): Promise<void> {
		if (this.entries.size >= this.config.maxEntries) {
			throw new Error("Vector store capacity exceeded");
		}

		// Ensure embedding exists
		let embedding = entry.embedding;
		if (embedding.length === 0) {
			embedding = await this.embeddingGenerator.generate(entry.content);
		}

		const entryWithEmbedding: MemoryEntry = {
			...entry,
			embedding,
		};

		this.entries.set(entry.id, entryWithEmbedding);
	}

	/**
	 * Query vector store by semantic similarity
	 */
	async query(query: MemoryQuery): Promise<readonly MemoryEntry[]> {
		let queryEmbedding = query.embedding;
		if (!queryEmbedding) {
			const encoder = new TextEncoder();
			const content = encoder.encode(query.query);
			queryEmbedding = await this.embeddingGenerator.generate(content);
		}

		const results: Array<{ entry: MemoryEntry; similarity: number }> = [];

		for (const entry of this.entries.values()) {
			// Apply access level filter
			if (query.accessLevel && entry.accessLevel !== query.accessLevel) {
				continue;
			}

			// Apply metadata filters
			if (query.filters) {
				let matches = true;
				for (const [key, value] of Object.entries(query.filters)) {
					if (entry.metadata[key] !== value) {
						matches = false;
						break;
					}
				}
				if (!matches) {
					continue;
				}
			}

			const similarity = this.embeddingGenerator.cosineSimilarity(queryEmbedding, entry.embedding);

			// Apply similarity threshold
			if (query.threshold !== undefined && similarity < query.threshold) {
				continue;
			}

			results.push({ entry, similarity });
		}

		// Sort by similarity (descending)
		results.sort((a, b) => b.similarity - a.similarity);

		// Apply limit
		const limit = query.limit ?? 10;
		return results.slice(0, limit).map((r) => r.entry);
	}

	/**
	 * Get entry by ID
	 */
	get(entryId: string): MemoryEntry | null {
		return this.entries.get(entryId) ?? null;
	}

	/**
	 * Remove entry from store
	 */
	remove(entryId: string): boolean {
		return this.entries.delete(entryId);
	}

	/**
	 * Get store statistics
	 */
	getStats(): { size: number; capacity: number; dimensions: number } {
		return {
			size: this.entries.size,
			capacity: this.config.maxEntries,
			dimensions: this.config.dimensions,
		};
	}

	/**
	 * Clear all entries
	 */
	clear(): void {
		this.entries.clear();
	}
}
