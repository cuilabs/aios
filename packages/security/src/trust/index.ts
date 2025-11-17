/**
 * Agent-to-agent trust graph
 * Manages trust relationships and capabilities between agents
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { TrustRelationship, TrustGraph, TrustNode, TrustEdge } from "../types.js";

/**
 * Trust graph manager
 * Maintains and queries trust relationships between agents
 */
export class TrustGraphManager {
	private readonly relationships = new Map<string, TrustRelationship>();
	private readonly nodes = new Map<string, TrustNode>();
	private readonly edges = new Map<string, TrustEdge[]>();

	/**
	 * Establish trust relationship
	 */
	establishTrust(
		from: string,
		to: string,
		level: number,
		capabilities: readonly string[],
		expiresAt?: number,
	): TrustRelationship {
		const relationshipId = `${from}:${to}`;

		const relationshipData = {
			from,
			to,
			level,
			capabilities,
			expiresAt,
		};

		const data = new TextEncoder().encode(JSON.stringify(relationshipData));
		const signature = QuantumSafeCrypto.hash(data);

		const relationship: TrustRelationship = {
			from,
			to,
			level: Math.max(0, Math.min(1, level)), // Clamp to [0, 1]
			capabilities,
			expiresAt,
			signature,
		};

		this.relationships.set(relationshipId, relationship);

		// Update graph
		this.updateGraphNode(from);
		this.updateGraphNode(to);
		this.updateGraphEdge(from, to, level, capabilities);

		return relationship;
	}

	/**
	 * Get trust relationship
	 */
	getTrust(from: string, to: string): TrustRelationship | null {
		const relationshipId = `${from}:${to}`;
		const relationship = this.relationships.get(relationshipId);

		// Check expiration
		if (relationship && relationship.expiresAt && relationship.expiresAt < Date.now()) {
			this.relationships.delete(relationshipId);
			return null;
		}

		return relationship ?? null;
	}

	/**
	 * Check if agent trusts another agent for a capability
	 */
	hasTrust(from: string, to: string, capability: string): boolean {
		const relationship = this.getTrust(from, to);
		if (!relationship) {
			return false;
		}

		return relationship.level > 0.5 && relationship.capabilities.includes(capability);
	}

	/**
	 * Revoke trust relationship
	 */
	revokeTrust(from: string, to: string): boolean {
		const relationshipId = `${from}:${to}`;
		const removed = this.relationships.delete(relationshipId);

		if (removed) {
			this.removeGraphEdge(from, to);
		}

		return removed;
	}

	/**
	 * Get trust graph
	 */
	getTrustGraph(): TrustGraph {
		return {
			nodes: Array.from(this.nodes.values()),
			edges: Array.from(this.edges.values()).flat(),
		};
	}

	/**
	 * Get trust path between agents
	 */
	findTrustPath(from: string, to: string, minTrustLevel: number = 0.5): string[] | null {
		// Simplified path finding using BFS
		const visited = new Set<string>();
		const queue: Array<{ agent: string; path: string[] }> = [{ agent: from, path: [from] }];

		while (queue.length > 0) {
			const { agent, path } = queue.shift()!;

			if (agent === to) {
				return path;
			}

			if (visited.has(agent)) {
				continue;
			}
			visited.add(agent);

			const edges = this.edges.get(agent) ?? [];
			for (const edge of edges) {
				if (edge.trustLevel >= minTrustLevel && !visited.has(edge.to)) {
					queue.push({ agent: edge.to, path: [...path, edge.to] });
				}
			}
		}

		return null;
	}

	/**
	 * Update graph node
	 */
	private updateGraphNode(agentId: string): void {
		if (!this.nodes.has(agentId)) {
			this.nodes.set(agentId, {
				agentId,
				reputation: 0.5,
				verified: false,
			});
		}
	}

	/**
	 * Update graph edge
	 */
	private updateGraphEdge(from: string, to: string, level: number, capabilities: readonly string[]): void {
		const edges = this.edges.get(from) ?? [];
		const existingIndex = edges.findIndex((e) => e.to === to);

		const edge: TrustEdge = {
			from,
			to,
			trustLevel: level,
			capabilities,
		};

		if (existingIndex !== -1) {
			edges[existingIndex] = edge;
		} else {
			edges.push(edge);
		}

		this.edges.set(from, edges);
	}

	/**
	 * Remove graph edge
	 */
	private removeGraphEdge(from: string, to: string): void {
		const edges = this.edges.get(from);
		if (edges) {
			const filtered = edges.filter((e) => e.to !== to);
			if (filtered.length === 0) {
				this.edges.delete(from);
			} else {
				this.edges.set(from, filtered);
			}
		}
	}
}

