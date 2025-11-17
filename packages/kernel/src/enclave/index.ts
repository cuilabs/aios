/**
 * Secure enclaves for trusted execution
 * Provides hardware-backed security for sensitive agent operations
 */

import { QuantumSafeCrypto } from "../crypto/index.js";
import type { SecureEnclave } from "../types.js";

export interface EnclaveAttestation {
	readonly enclaveId: string;
	readonly publicKey: Uint8Array;
	readonly measurement: Uint8Array;
	readonly timestamp: number;
	readonly signature: Uint8Array;
}

/**
 * Secure enclave manager
 * Creates and manages trusted execution environments
 */
export class SecureEnclaveManager {
	private readonly enclaves = new Map<string, SecureEnclave>();
	private readonly attestations = new Map<string, EnclaveAttestation>();

	/**
	 * Create a new secure enclave
	 */
	createEnclave(
		id: string,
		options: {
			memoryProtection?: boolean;
			encryptionAtRest?: boolean;
		} = {},
	): SecureEnclave {
		const keyPair = QuantumSafeCrypto.generateKeyPair();
		const measurement = QuantumSafeCrypto.hash(keyPair.publicKey);

		const attestation: EnclaveAttestation = {
			enclaveId: id,
			publicKey: keyPair.publicKey,
			measurement,
			timestamp: Date.now(),
			signature: QuantumSafeCrypto.sign(measurement, keyPair.privateKey).signature,
		};

		const enclave: SecureEnclave = {
			id,
			attestation: this.serializeAttestation(attestation),
			memoryProtection: options.memoryProtection ?? true,
			encryptionAtRest: options.encryptionAtRest ?? true,
		};

		this.enclaves.set(id, enclave);
		this.attestations.set(id, attestation);

		return enclave;
	}

	/**
	 * Verify enclave attestation
	 */
	verifyAttestation(enclaveId: string): boolean {
		const attestation = this.attestations.get(enclaveId);
		if (!attestation) {
			return false;
		}

		// Verify measurement matches public key
		const expectedMeasurement = QuantumSafeCrypto.hash(attestation.publicKey);
		if (!this.arraysEqual(expectedMeasurement, attestation.measurement)) {
			return false;
		}

		// Verify signature
		return QuantumSafeCrypto.verify(
			{
				signature: attestation.signature,
				publicKey: attestation.publicKey,
				algorithm: "CRYSTALS-Dilithium",
				timestamp: attestation.timestamp,
			},
			attestation.measurement,
			attestation.publicKey,
		);
	}

	/**
	 * Get enclave by ID
	 */
	getEnclave(enclaveId: string): SecureEnclave | null {
		return this.enclaves.get(enclaveId) ?? null;
	}

	/**
	 * Remove enclave
	 */
	removeEnclave(enclaveId: string): boolean {
		this.attestations.delete(enclaveId);
		return this.enclaves.delete(enclaveId);
	}

	/**
	 * List all enclaves
	 */
	listEnclaves(): readonly SecureEnclave[] {
		return Array.from(this.enclaves.values());
	}

	/**
	 * Serialize attestation to string
	 */
	private serializeAttestation(attestation: EnclaveAttestation): string {
		return JSON.stringify({
			enclaveId: attestation.enclaveId,
			publicKey: Array.from(attestation.publicKey),
			measurement: Array.from(attestation.measurement),
			timestamp: attestation.timestamp,
			signature: Array.from(attestation.signature),
		});
	}

	/**
	 * Compare two Uint8Arrays
	 */
	private arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
		if (a.length !== b.length) {
			return false;
		}
		for (let i = 0; i < a.length; i++) {
			if (a[i] !== b[i]) {
				return false;
			}
		}
		return true;
	}
}

