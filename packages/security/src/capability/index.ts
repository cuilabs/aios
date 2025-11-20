/**
 * Capability token system
 * Fine-grained permissions for agent operations
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { CapabilityToken } from "../types.js";

/**
 * Capability manager
 * Issues and validates capability tokens
 */
export class CapabilityManager {
	private readonly tokens = new Map<string, CapabilityToken>();
	private readonly identityManager: {
		sign: (agentId: string, data: Uint8Array) => Uint8Array | null;
	};

	constructor(identityManager: { sign: (agentId: string, data: Uint8Array) => Uint8Array | null }) {
		this.identityManager = identityManager;
	}

	/**
	 * Issue capability token
	 */
	issueToken(
		agentId: string,
		capabilities: readonly string[],
		permissions: Readonly<Record<string, unknown>> = {},
		expiresAt?: number
	): CapabilityToken {
		const tokenId = this.generateTokenId();
		const tokenData = {
			id: tokenId,
			agentId,
			capabilities,
			permissions,
			expiresAt,
		};

		const data = new TextEncoder().encode(JSON.stringify(tokenData));
		const signature = this.identityManager.sign(agentId, data);

		if (!signature) {
			throw new Error("Failed to sign capability token");
		}

		const token: CapabilityToken = {
			id: tokenId,
			agentId,
			capabilities,
			permissions,
			expiresAt,
			signature,
		};

		this.tokens.set(tokenId, token);
		return token;
	}

	/**
	 * Validate capability token
	 */
	validateToken(token: CapabilityToken): { valid: boolean; reason?: string } {
		// Check expiration
		if (token.expiresAt !== undefined && token.expiresAt < Date.now()) {
			return { valid: false, reason: "Token expired" };
		}

		// Verify signature
		const tokenData = {
			id: token.id,
			agentId: token.agentId,
			capabilities: token.capabilities,
			permissions: token.permissions,
			expiresAt: token.expiresAt,
		};

		const data = new TextEncoder().encode(JSON.stringify(tokenData));
		const dataHash = QuantumSafeCrypto.hash(data);

		if (token.signature.length === 0) {
			return { valid: false, reason: "Invalid signature: empty" };
		}

		const identity = this.identityManager.sign(token.agentId, dataHash);
		if (!identity) {
			return { valid: false, reason: "Invalid identity" };
		}

		const expectedSignature = QuantumSafeCrypto.hash(new Uint8Array([...dataHash, ...identity]));
		if (expectedSignature.length !== token.signature.length) {
			return { valid: false, reason: "Invalid signature: length mismatch" };
		}

		for (let i = 0; i < expectedSignature.length; i++) {
			if (expectedSignature[i] !== token.signature[i]) {
				return { valid: false, reason: "Invalid signature: verification failed" };
			}
		}

		return { valid: true };
	}

	/**
	 * Check if token has capability
	 */
	hasCapability(token: CapabilityToken, capability: string): boolean {
		if (!this.validateToken(token).valid) {
			return false;
		}

		return token.capabilities.includes(capability);
	}

	/**
	 * Revoke capability token
	 */
	revokeToken(tokenId: string): boolean {
		return this.tokens.delete(tokenId);
	}

	/**
	 * Get token by ID
	 */
	getToken(tokenId: string): CapabilityToken | null {
		return this.tokens.get(tokenId) ?? null;
	}

	/**
	 * Generate unique token ID
	 */
	private generateTokenId(): string {
		const bytes = QuantumSafeCrypto.randomBytes(16);
		return `cap-${Array.from(bytes, (b: number) => b.toString(16).padStart(2, "0")).join("")}`;
	}
}
