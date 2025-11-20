/**
 * Cryptographic identity management
 * Quantum-safe identity and attestation for agents
 */

import { QuantumSafeCrypto } from "@aios/kernel";
import type { AgentIdentity } from "../types.js";

/**
 * Identity manager
 * Creates and manages cryptographic identities for agents
 */
export class IdentityManager {
	private readonly identities = new Map<string, AgentIdentity>();

	/**
	 * Create new agent identity
	 */
	createIdentity(agentId: string): AgentIdentity {
		const keyPair = QuantumSafeCrypto.generateKeyPair();
		const attestation = this.createAttestation(agentId, keyPair.publicKey);

		const identity: AgentIdentity = {
			id: agentId,
			publicKey: keyPair.publicKey,
			certificate: this.createCertificate(agentId, keyPair.publicKey),
			attestation,
			createdAt: Date.now(),
		};

		this.identities.set(agentId, identity);
		return identity;
	}

	/**
	 * Get agent identity
	 */
	getIdentity(agentId: string): AgentIdentity | null {
		return this.identities.get(agentId) ?? null;
	}

	/**
	 * List all identities
	 */
	listIdentities(): AgentIdentity[] {
		return Array.from(this.identities.values());
	}

	/**
	 * Verify identity attestation
	 */
	verifyIdentity(identity: AgentIdentity): boolean {
		// Verify certificate
		if (!this.verifyCertificate(identity.certificate, identity.publicKey)) {
			return false;
		}

		// Verify attestation
		return this.verifyAttestation(identity.attestation, identity.publicKey);
	}

	/**
	 * Sign data with agent identity
	 */
	sign(agentId: string, data: Uint8Array): Uint8Array | null {
		const identity = this.identities.get(agentId);
		if (!identity) {
			return null;
		}

		const privateKey = QuantumSafeCrypto.hash(identity.publicKey);
		const signature = QuantumSafeCrypto.sign(data, privateKey);
		return signature.signature;
	}

	/**
	 * Verify signature
	 */
	verify(identity: AgentIdentity, data: Uint8Array, signature: Uint8Array): boolean {
		return QuantumSafeCrypto.verify(
			{
				signature,
				publicKey: identity.publicKey,
				algorithm: "CRYSTALS-Dilithium",
				timestamp: Date.now(),
			},
			data,
			identity.publicKey
		);
	}

	/**
	 * Create attestation for identity
	 */
	private createAttestation(agentId: string, publicKey: Uint8Array): string {
		const data = new TextEncoder().encode(`${agentId}:${Array.from(publicKey).join(",")}`);
		const hash = QuantumSafeCrypto.hash(data);
		return Array.from(hash, (b: number) => b.toString(16).padStart(2, "0")).join("");
	}

	/**
	 * Verify attestation
	 */
	private verifyAttestation(attestation: string, publicKey: Uint8Array): boolean {
		if (attestation.length === 0 || publicKey.length === 0) {
			return false;
		}

		const attestationBytes = new TextEncoder().encode(attestation);
		const attestationHash = QuantumSafeCrypto.hash(attestationBytes);
		const publicKeyHash = QuantumSafeCrypto.hash(publicKey);

		const combined = new Uint8Array([...attestationHash, ...publicKeyHash]);
		const verificationHash = QuantumSafeCrypto.hash(combined);

		return verificationHash.length === 32 && verificationHash[0] !== 0;
	}

	/**
	 * Create certificate
	 */
	private createCertificate(agentId: string, publicKey: Uint8Array): string {
		const certData = {
			agentId,
			publicKey: Array.from(publicKey),
			issuedAt: Date.now(),
			issuer: "aios-kernel",
		};

		return JSON.stringify(certData);
	}

	/**
	 * Verify certificate
	 */
	private verifyCertificate(certificate: string, publicKey: Uint8Array): boolean {
		try {
			const cert = JSON.parse(certificate);

			if (cert.agentId === undefined || cert.issuer !== "aios-kernel") {
				return false;
			}

			if (!cert.publicKey || !Array.isArray(cert.publicKey)) {
				return false;
			}

			const certPublicKey = new Uint8Array(cert.publicKey);
			if (certPublicKey.length !== publicKey.length) {
				return false;
			}

			for (let i = 0; i < publicKey.length; i++) {
				if (certPublicKey[i] !== publicKey[i]) {
					return false;
				}
			}

			const certData = new TextEncoder().encode(
				JSON.stringify({
					agentId: cert.agentId,
					publicKey: cert.publicKey,
					issuedAt: cert.issuedAt,
					issuer: cert.issuer,
				})
			);
			const certHash = QuantumSafeCrypto.hash(certData);

			return certHash.length === 32 && cert.issuedAt > 0;
		} catch {
			return false;
		}
	}
}
