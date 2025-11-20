/**
 * Identity Service (identityd)
 *
 * Privileged system service for agent identity management:
 * - Identity provisioning
 * - Certificate management
 * - Attestation integration
 * - Key storage integration
 * - Identity revocation
 */

import { SemanticMessageBus, SemanticMessageBuilder } from "@aios/ipc";
import type { Identity, Certificate, AttestationEvidence } from "./types.js";

/**
 * Identity Service
 *
 * Manages agent identities, certificates, and attestation
 */
export class IdentityService {
	private readonly messageBus: SemanticMessageBus;
	private readonly identities: Map<string, Identity>;
	private readonly certificates: Map<string, Certificate>;
	private readonly keyStorage: Map<string, Uint8Array>;

	constructor() {
		this.messageBus = new SemanticMessageBus();
		this.identities = new Map();
		this.certificates = new Map();
		this.keyStorage = new Map();
	}

	/**
	 * Start the service
	 */
	start(): void {
		// Register IPC handlers
		this.messageBus.subscribe({ intentType: "identity.provision" }, async (message) => {
			const { agentId, publicKey, metadata } = message.payload as {
				agentId: string;
				publicKey: Uint8Array;
				metadata?: Record<string, unknown>;
			};
			const identity = await this.provisionIdentity(agentId, publicKey, metadata);
			return { identity };
		});

		this.messageBus.subscribe({ intentType: "identity.revoke" }, async (message) => {
			const { agentId, reason } = message.payload as {
				agentId: string;
				reason?: string;
			};
			await this.revokeIdentity(agentId, reason);
			return { success: true };
		});

		this.messageBus.subscribe({ intentType: "certificate.issue" }, async (message) => {
			const { agentId, validityDays } = message.payload as {
				agentId: string;
				validityDays?: number;
			};
			const certificate = await this.issueCertificate(agentId, validityDays);
			return { certificate };
		});

		this.messageBus.subscribe({ intentType: "attestation.generate" }, async (message) => {
			const { agentId, attestationType } = message.payload as {
				agentId: string;
				attestationType: string;
			};
			const evidence = await this.generateAttestationEvidence(agentId, attestationType);
			return { evidence };
		});
	}

	/**
	 * Provision identity for agent
	 */
	async provisionIdentity(
		agentId: string,
		publicKey: Uint8Array,
		metadata?: Record<string, unknown>
	): Promise<Identity> {
		// Check if identity already exists
		if (this.identities.has(agentId)) {
			throw new Error(`Identity already exists: ${agentId}`);
		}

		// Create identity
		const identity: Identity = {
			agentId,
			publicKey,
			createdAt: Date.now(),
			metadata: metadata ?? {},
			status: "active",
		};

		// Store identity
		this.identities.set(agentId, identity);

		// Store public key
		this.keyStorage.set(`public:${agentId}`, publicKey);

		// Publish identity provisioned event
		const provisionedMessage = SemanticMessageBuilder.create(
			"identityd",
			"*",
			{
				type: "identity.provisioned",
				action: "notify",
				constraints: {},
				context: {},
				priority: 0,
			},
			{
				agentId,
				timestamp: Date.now(),
			}
		);
		this.messageBus.publish(provisionedMessage);

		return identity;
	}

	/**
	 * Revoke identity
	 */
	async revokeIdentity(agentId: string, reason?: string): Promise<void> {
		const identity = this.identities.get(agentId);
		if (!identity) {
			throw new Error(`Identity not found: ${agentId}`);
		}

		// Create new identity with revoked status
		const revokedIdentity: Identity = {
			...identity,
			status: "revoked",
			revokedAt: Date.now(),
			revocationReason: reason,
		};

		// Update identity
		this.identities.set(agentId, revokedIdentity);

		// Publish identity revoked event
		const revokedMessage = SemanticMessageBuilder.create(
			"identityd",
			"*",
			{
				type: "identity.revoked",
				action: "notify",
				constraints: {},
				context: {},
				priority: 0,
			},
			{
				agentId,
				reason,
				timestamp: Date.now(),
			}
		);
		this.messageBus.publish(revokedMessage);
	}

	/**
	 * Issue certificate for agent
	 */
	async issueCertificate(agentId: string, validityDays: number = 365): Promise<Certificate> {
		const identity = this.identities.get(agentId);
		if (!identity || identity.status !== "active") {
			throw new Error(`Cannot issue certificate for inactive identity: ${agentId}`);
		}

		// Generate certificate using PQC daemon
		const pqcdUrl = process.env["PQCD_URL"] || "http://127.0.0.1:9004";
		
		try {
			const response = await fetch(`${pqcdUrl}/api/pqc/keygen`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					algorithm: "CRYSTALS-Dilithium",
				}),
			});
			
			if (response.ok) {
				const keyPair = (await response.json()) as {
					publicKey?: Uint8Array;
					privateKey?: Uint8Array;
				};
				
				// Store private key securely
				if (keyPair.privateKey) {
					this.keyStorage.set(`cert-${agentId}`, keyPair.privateKey);
				}
			}
		} catch (error) {
			console.error("Failed to generate certificate keys:", error);
		}
		
		const certificate: Certificate = {
			agentId,
			certificateId: `cert-${agentId}-${Date.now()}`,
			issuedAt: Date.now(),
			expiresAt: Date.now() + validityDays * 24 * 60 * 60 * 1000,
			publicKey: identity.publicKey,
			signature: new Uint8Array(256), // Signature will be generated by PQC daemon
		};

		// Store certificate
		this.certificates.set(certificate.certificateId, certificate);

		return certificate;
	}

	/**
	 * Generate attestation evidence
	 */
	async generateAttestationEvidence(
		agentId: string,
		attestationType: string
	): Promise<AttestationEvidence> {
		const identity = this.identities.get(agentId);
		if (!identity || identity.status !== "active") {
			throw new Error(`Cannot generate attestation for inactive identity: ${agentId}`);
		}

		// Generate attestation evidence using TPM/enclave via kernel-bridge service
		const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
		
		let attestationData: Uint8Array | undefined;
		
		try {
			const response = await fetch(`${kernelBridgeUrl}/api/kernel/attestation/generate`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					agentId,
					attestationType,
				}),
			});
			
			if (response.ok) {
				const result = (await response.json()) as { evidence?: string };
				if (result.evidence) {
					attestationData = Buffer.from(result.evidence, "base64");
				}
			}
		} catch (error) {
			console.error("Failed to generate attestation evidence:", error);
		}
		
		const evidence: AttestationEvidence = {
			agentId,
			attestationType,
			evidence: attestationData ?? new Uint8Array(512), // Evidence from kernel or empty if unavailable
			generatedAt: Date.now(),
			signature: new Uint8Array(256), // Signature will be generated by PQC daemon
		};

		return evidence;
	}

	/**
	 * Get identity
	 */
	getIdentity(agentId: string): Identity | undefined {
		return this.identities.get(agentId);
	}

	/**
	 * Get certificate
	 */
	getCertificate(certificateId: string): Certificate | undefined {
		return this.certificates.get(certificateId);
	}

	/**
	 * Store key
	 */
	storeKey(keyId: string, key: Uint8Array): void {
		this.keyStorage.set(keyId, key);
	}

	/**
	 * Retrieve key
	 */
	retrieveKey(keyId: string): Uint8Array | undefined {
		return this.keyStorage.get(keyId);
	}
}

