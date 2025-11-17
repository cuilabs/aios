/**
 * Sparse semantic index for memory
 * Fast lookup and retrieval of memory entries
 */

import type { MemoryEntry, MemoryNode, MemoryEdge } from "../types.js";

export interface SemanticIndex {
	readonly entryIds: readonly string[];
	readonly nodes: readonly MemoryNode[];
	readonly edges: readonly MemoryEdge[];
}

/**
 * Sparse semantic index
 * Provides fast semantic search and graph traversal
 */
export class SparseSemanticIndex {
	private readonly index = new Map<string, SemanticIndex>();
	private readonly nodeIndex = new Map<string, MemoryNode>();
	private readonly edgeIndex = new Map<string, MemoryEdge[]>();

	/**
	 * Index a memory entry
	 */
	indexEntry(entry: MemoryEntry, nodes: readonly MemoryNode[], edges: readonly MemoryEdge[]): void {
		const semanticIndex: SemanticIndex = {
			entryIds: [entry.id],
			nodes,
			edges,
		};

		this.index.set(entry.id, semanticIndex);

		// Index nodes
		for (const node of nodes) {
			this.nodeIndex.set(node.id, node);
		}

		// Index edges
		for (const edge of edges) {
			const existing = this.edgeIndex.get(edge.from) ?? [];
			this.edgeIndex.set(edge.from, [...existing, edge]);
		}
	}

	/**
	 * Query index by node type
	 */
	findByNodeType(type: MemoryNode["type"]): readonly MemoryNode[] {
		const results: MemoryNode[] = [];
		for (const node of this.nodeIndex.values()) {
			if (node.type === type) {
				results.push(node);
			}
		}
		return results;
	}

	/**
	 * Find nodes connected to a given node
	 */
	findConnectedNodes(nodeId: string): readonly MemoryNode[] {
		const edges = this.edgeIndex.get(nodeId) ?? [];
		const connectedIds = new Set<string>();

		for (const edge of edges) {
			connectedIds.add(edge.to);
		}

		const nodes: MemoryNode[] = [];
		for (const nodeId of connectedIds) {
			const node = this.nodeIndex.get(nodeId);
			if (node) {
				nodes.push(node);
			}
		}

		return nodes;
	}

	/**
	 * Find edges by relation type
	 */
	findByRelation(relation: string): readonly MemoryEdge[] {
		const results: MemoryEdge[] = [];
		for (const edges of this.edgeIndex.values()) {
			for (const edge of edges) {
				if (edge.relation === relation) {
					results.push(edge);
				}
			}
		}
		return results;
	}

	/**
	 * Get semantic index for an entry
	 */
	getIndex(entryId: string): SemanticIndex | null {
		return this.index.get(entryId) ?? null;
	}

	/**
	 * Remove entry from index
	 */
	removeEntry(entryId: string): boolean {
		const semanticIndex = this.index.get(entryId);
		if (!semanticIndex) {
			return false;
		}

		// Remove nodes
		for (const node of semanticIndex.nodes) {
			this.nodeIndex.delete(node.id);
		}

		// Remove edges
		for (const edge of semanticIndex.edges) {
			const edges = this.edgeIndex.get(edge.from);
			if (edges) {
				const filtered = edges.filter((e) => e.to !== edge.to || e.relation !== edge.relation);
				if (filtered.length === 0) {
					this.edgeIndex.delete(edge.from);
				} else {
					this.edgeIndex.set(edge.from, filtered);
				}
			}
		}

		return this.index.delete(entryId);
	}

	/**
	 * Get index statistics
	 */
	getStats(): { entries: number; nodes: number; edges: number } {
		let totalEdges = 0;
		for (const edges of this.edgeIndex.values()) {
			totalEdges += edges.length;
		}

		return {
			entries: this.index.size,
			nodes: this.nodeIndex.size,
			edges: totalEdges,
		};
	}
}

