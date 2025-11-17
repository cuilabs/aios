/**
 * Memory fabric types
 */

export interface MemoryEntry {
	readonly id: string;
	readonly agentId: string;
	readonly content: Uint8Array;
	readonly embedding: Float32Array;
	readonly metadata: Readonly<Record<string, unknown>>;
	readonly timestamp: number;
	readonly version: number;
	readonly accessLevel: "private" | "shared" | "public";
}

export interface MemoryQuery {
	readonly query: string;
	readonly embedding?: Float32Array;
	readonly limit?: number;
	readonly threshold?: number;
	readonly filters?: Readonly<Record<string, unknown>>;
	readonly accessLevel?: MemoryEntry["accessLevel"];
}

export interface MemoryGraph {
	readonly nodes: readonly MemoryNode[];
	readonly edges: readonly MemoryEdge[];
}

export interface MemoryNode {
	readonly id: string;
	readonly entryId: string;
	readonly type: "fact" | "concept" | "event" | "relation";
	readonly weight: number;
}

export interface MemoryEdge {
	readonly from: string;
	readonly to: string;
	readonly relation: string;
	readonly strength: number;
}

export interface MemoryVersion {
	readonly version: number;
	readonly entryId: string;
	readonly timestamp: number;
	readonly changes: Readonly<Record<string, unknown>>;
}

