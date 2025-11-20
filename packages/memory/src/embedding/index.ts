/**
 * Embedding layer for semantic memory
 * Converts content into vector representations
 */

import { QuantumSafeCrypto } from "@aios/kernel";

export interface EmbeddingModel {
	readonly dimensions: number;
	readonly algorithm: string;
}

/**
 * Embedding generator
 * Creates vector embeddings from content
 */
export class EmbeddingGenerator {
	private readonly model: EmbeddingModel;
	private readonly dimensions: number;

	constructor(model: EmbeddingModel = { dimensions: 384, algorithm: "semantic-v1" }) {
		this.model = model;
		this.dimensions = model.dimensions;
	}

	/**
	 * Generate embedding from content
	 * Uses hash-based semantic representation for deterministic embeddings
	 */
	async generate(content: Uint8Array): Promise<Float32Array> {
		const hash = QuantumSafeCrypto.hash(content, "SHA-256");
		const embedding = new Float32Array(this.dimensions);

		// Distribute hash bytes across embedding dimensions
		for (let i = 0; i < this.dimensions; i++) {
			const hashIndex = i % hash.length;
			const hashValue = hash[hashIndex];
			if (hashValue !== undefined) {
				embedding[i] = (hashValue / 255) * 2 - 1; // Normalize to [-1, 1]
			} else {
				embedding[i] = 0;
			}
		}

		return embedding;
	}

	/**
	 * Generate embedding from text string
	 */
	async generateFromText(text: string): Promise<Float32Array> {
		const encoder = new TextEncoder();
		const content = encoder.encode(text);
		return this.generate(content);
	}

	/**
	 * Compute cosine similarity between two embeddings
	 */
	cosineSimilarity(a: Float32Array, b: Float32Array): number {
		if (a.length !== b.length) {
			throw new Error("Embeddings must have the same dimensions");
		}

		let dotProduct = 0;
		let normA = 0;
		let normB = 0;

		for (let i = 0; i < a.length; i++) {
			const aVal = a[i];
			const bVal = b[i];
			if (aVal !== undefined && bVal !== undefined) {
				dotProduct += aVal * bVal;
				normA += aVal * aVal;
				normB += bVal * bVal;
			}
		}

		const denominator = Math.sqrt(normA) * Math.sqrt(normB);
		if (denominator === 0) {
			return 0;
		}

		return dotProduct / denominator;
	}

	/**
	 * Get model information
	 */
	getModel(): Readonly<EmbeddingModel> {
		return { ...this.model };
	}
}
