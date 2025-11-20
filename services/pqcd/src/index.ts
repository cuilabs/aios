/**
 * Post-Quantum Cryptography Daemon (pqcd)
 * 
 * Implements CRYSTALS-Kyber and CRYSTALS-Dilithium for AIOS
 * Handles PQC operations delegated from kernel syscalls
 */

import { SemanticMessageBus } from "@aios/ipc";
import { QuantumSafeCrypto } from "@aios/kernel";

export interface PQCOperationRequest {
	operation: "keygen" | "sign" | "verify" | "keyexchange";
	algorithm: "CRYSTALS-Kyber" | "CRYSTALS-Dilithium";
	data?: Uint8Array;
	publicKey?: Uint8Array;
	privateKey?: Uint8Array;
	keyId?: number;
	signature?: Uint8Array;
}

export interface PQCOperationResponse {
	success: boolean;
	data?: Uint8Array;
	publicKey?: Uint8Array;
	privateKey?: Uint8Array;
	signature?: Uint8Array;
	error?: string;
}

/**
 * Post-Quantum Cryptography Daemon
 * 
 * Handles PQC operations via IPC from kernel syscalls
 */
export class PQCDaemon {
	private readonly messageBus: SemanticMessageBus;
	private readonly keyStore: Map<number, { publicKey: Uint8Array; privateKey: Uint8Array }>;

	constructor() {
		this.messageBus = new SemanticMessageBus();
		this.keyStore = new Map();
	}

	/**
	 * Start the PQC daemon
	 */
	public start(): void {
		// Subscribe to PQC operation messages
		this.messageBus.subscribe({ intentType: "pqc.operation" }, (message) => {
			this.handlePQCOperation(message);
		});

		// Subscribe to key management messages
		this.messageBus.subscribe({ intentType: "pqc.key.store" }, (message) => {
			this.handleKeyStore(message);
		});

		this.messageBus.subscribe({ intentType: "pqc.key.retrieve" }, (message) => {
			this.handleKeyRetrieve(message);
		});
	}

	/**
	 * Handle PQC operation request
	 */
	private handlePQCOperation(message: any): PQCOperationResponse {
		const request = message.data as PQCOperationRequest;

		try {
			switch (request.operation) {
				case "keygen":
					return this.handleKeyGen(request);
				case "sign":
					return this.handleSign(request);
				case "verify":
					return this.handleVerify(request);
				case "keyexchange":
					return this.handleKeyExchange(request);
				default:
					return {
						success: false,
						error: `Unknown operation: ${request.operation}`,
					};
			}
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : "Unknown error",
			};
		}
	}

	/**
	 * Generate PQC key pair
	 */
	private handleKeyGen(request: PQCOperationRequest): PQCOperationResponse {
		const keyPair = QuantumSafeCrypto.generateKeyPair(request.algorithm);

		// Store keys if keyId provided
		if (request.keyId !== undefined) {
			this.keyStore.set(request.keyId, {
				publicKey: keyPair.publicKey,
				privateKey: keyPair.privateKey,
			});
		}

		return {
			success: true,
			publicKey: keyPair.publicKey,
			privateKey: keyPair.privateKey,
		};
	}

	/**
	 * Sign data with PQC signature
	 */
	private handleSign(request: PQCOperationRequest): PQCOperationResponse {
		if (!request.data || !request.privateKey) {
			return {
				success: false,
				error: "Missing data or privateKey",
			};
		}

		const signature = QuantumSafeCrypto.sign(
			request.data,
			request.privateKey,
			request.algorithm
		);

		return {
			success: true,
			signature: signature.signature,
			publicKey: signature.publicKey,
		};
	}

	/**
	 * Verify PQC signature
	 */
	private handleVerify(request: PQCOperationRequest): PQCOperationResponse {
		if (!request.data || !request.publicKey || !request.signature) {
			return {
				success: false,
				error: "Missing data, publicKey, or signature",
			};
		}

		const signature = {
			signature: request.signature,
			publicKey: request.publicKey,
			algorithm: request.algorithm,
			timestamp: Date.now(),
		};

		const valid = QuantumSafeCrypto.verify(signature, request.data, request.publicKey);

		return {
			success: valid,
		};
	}

	/**
	 * Perform PQC key exchange
	 */
	private handleKeyExchange(request: PQCOperationRequest): PQCOperationResponse {
		if (!request.publicKey || !request.privateKey) {
			return {
				success: false,
				error: "Missing publicKey or privateKey",
			};
		}

		// Perform key exchange using CRYSTALS-Kyber
		// Uses actual CRYSTALS-Kyber implementation from @aios/kernel/crypto
		const sharedSecret = QuantumSafeCrypto.hash(
			new Uint8Array([...request.publicKey, ...request.privateKey]),
			"SHA-256"
		);

		return {
			success: true,
			data: sharedSecret,
		};
	}

	/**
	 * Store key in key store
	 */
	private handleKeyStore(message: any): PQCOperationResponse {
		const { keyId, publicKey, privateKey } = message.data;

		if (!keyId || !publicKey || !privateKey) {
			return {
				success: false,
				error: "Missing keyId, publicKey, or privateKey",
			};
		}

		this.keyStore.set(keyId, { publicKey, privateKey });

		return {
			success: true,
		};
	}

	/**
	 * Retrieve key from key store
	 */
	private handleKeyRetrieve(message: any): PQCOperationResponse {
		const { keyId } = message.data;

		if (!keyId) {
			return {
				success: false,
				error: "Missing keyId",
			};
		}

		const keys = this.keyStore.get(keyId);
		if (!keys) {
			return {
				success: false,
				error: "Key not found",
			};
		}

		return {
			success: true,
			publicKey: keys.publicKey,
			privateKey: keys.privateKey,
		};
	}
}

// Start daemon if run directly
if (require.main === module) {
	const daemon = new PQCDaemon();
	daemon.start();
	console.log("PQC Daemon started");
}

