/**
 * Security model types
 */

export interface AgentIdentity {
	readonly id: string;
	readonly publicKey: Uint8Array;
	readonly certificate: string;
	readonly attestation: string;
	readonly createdAt: number;
}

export interface CapabilityToken {
	readonly id: string;
	readonly agentId: string;
	readonly capabilities: readonly string[];
	readonly permissions: Readonly<Record<string, unknown>>;
	readonly expiresAt?: number;
	readonly signature: Uint8Array;
}

export interface BehavioralProfile {
	readonly agentId: string;
	readonly patterns: Readonly<Record<string, number>>;
	readonly baseline: Readonly<Record<string, number>>;
	readonly anomalies: readonly BehavioralAnomaly[];
	readonly lastUpdated: number;
}

export interface BehavioralAnomaly {
	readonly type: string;
	readonly severity: "low" | "medium" | "high" | "critical";
	readonly description: string;
	readonly timestamp: number;
	readonly metrics: Readonly<Record<string, unknown>>;
}

export interface TrustRelationship {
	readonly from: string;
	readonly to: string;
	readonly level: number; // 0-1
	readonly capabilities: readonly string[];
	readonly expiresAt?: number;
	readonly signature: Uint8Array;
}

export interface TrustGraph {
	readonly nodes: readonly TrustNode[];
	readonly edges: readonly TrustEdge[];
}

export interface TrustNode {
	readonly agentId: string;
	readonly reputation: number;
	readonly verified: boolean;
}

export interface TrustEdge {
	readonly from: string;
	readonly to: string;
	readonly trustLevel: number;
	readonly capabilities: readonly string[];
}

