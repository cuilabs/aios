/**
 * Identity Service Types
 */

export interface Identity {
	readonly agentId: string;
	readonly publicKey: Uint8Array;
	readonly createdAt: number;
	readonly metadata: Record<string, unknown>;
	readonly status: "active" | "revoked" | "suspended";
	readonly revokedAt?: number;
	readonly revocationReason?: string;
}

export interface Certificate {
	readonly agentId: string;
	readonly certificateId: string;
	readonly issuedAt: number;
	readonly expiresAt: number;
	readonly publicKey: Uint8Array;
	readonly signature: Uint8Array;
}

export interface AttestationEvidence {
	readonly agentId: string;
	readonly attestationType: string;
	readonly evidence: Uint8Array;
	readonly generatedAt: number;
	readonly signature: Uint8Array;
}
