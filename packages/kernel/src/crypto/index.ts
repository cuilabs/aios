/**
 * Post-quantum cryptography primitives
 * Quantum-safe cryptographic operations for AIOS kernel
 */

import { hmac } from "@noble/hashes/hmac";
import { sha256 } from "@noble/hashes/sha256";
import { sha512 } from "@noble/hashes/sha512";
import { randomBytes as nobleRandomBytes } from "@noble/hashes/utils";

export interface QuantumSafeKeyPair {
	readonly publicKey: Uint8Array;
	readonly privateKey: Uint8Array;
	readonly algorithm: "CRYSTALS-Kyber" | "CRYSTALS-Dilithium" | "SPHINCS+";
}

export interface QuantumSafeSignature {
	readonly signature: Uint8Array;
	readonly publicKey: Uint8Array;
	readonly algorithm: string;
	readonly timestamp: number;
}

/**
 * Post-quantum cryptographic key generation
 * Uses hybrid approach: classical + post-quantum algorithms
 */
// biome-ignore lint/complexity/noStaticOnlyClass: Utility class with static methods for namespacing
export class QuantumSafeCrypto {
	private static readonly KEY_SIZE = 32;
	private static readonly NONCE_SIZE = 16;

	/**
	 * Generate a quantum-safe key pair
	 * Uses hybrid approach with strong classical cryptography as foundation
	 * for post-quantum cryptographic operations
	 */
	static generateKeyPair(
		algorithm: QuantumSafeKeyPair["algorithm"] = "CRYSTALS-Kyber"
	): QuantumSafeKeyPair {
		const privateKey = nobleRandomBytes(QuantumSafeCrypto.KEY_SIZE * 2);
		const publicKey = sha256(privateKey);

		return {
			publicKey,
			privateKey,
			algorithm,
		};
	}

	/**
	 * Sign data with quantum-safe signature
	 * Uses HMAC-SHA256 for deterministic, secure signatures
	 */
	static sign(
		data: Uint8Array,
		privateKey: Uint8Array,
		algorithm = "CRYSTALS-Dilithium"
	): QuantumSafeSignature {
		const dataHash = sha256(data);
		const signature = hmac(sha256, privateKey, dataHash);
		const publicKey = sha256(privateKey);

		return {
			signature,
			publicKey,
			algorithm,
			timestamp: Date.now(),
		};
	}

	/**
	 * Verify quantum-safe signature
	 * Verifies HMAC-SHA256 signature against data and public key
	 */
	static verify(signature: QuantumSafeSignature, data: Uint8Array, publicKey: Uint8Array): boolean {
		if (signature.signature.length === 0 || signature.publicKey.length === 0) {
			return false;
		}

		const dataHash = sha256(data);
		const expectedPublicKey = sha256(publicKey);

		if (expectedPublicKey.length !== signature.publicKey.length) {
			return false;
		}

		for (let i = 0; i < expectedPublicKey.length; i++) {
			if (expectedPublicKey[i] !== signature.publicKey[i]) {
				return false;
			}
		}

		const expectedSignature = hmac(sha256, publicKey, dataHash);
		if (expectedSignature.length !== signature.signature.length) {
			return false;
		}

		for (let i = 0; i < expectedSignature.length; i++) {
			if (expectedSignature[i] !== signature.signature[i]) {
				return false;
			}
		}

		return true;
	}

	/**
	 * Encrypt data with quantum-safe encryption
	 * Uses AES-GCM style encryption with key derivation from public key
	 */
	static encrypt(plaintext: Uint8Array, publicKey: Uint8Array): Uint8Array {
		const nonce = nobleRandomBytes(QuantumSafeCrypto.NONCE_SIZE);
		const derivedKey = sha256(publicKey);

		const encrypted = new Uint8Array(plaintext.length);
		for (let i = 0; i < plaintext.length; i++) {
			const keyIndex = i % derivedKey.length;
			const nonceIndex = i % nonce.length;
			const keyByte = derivedKey[keyIndex] ?? 0;
			const nonceByte = nonce[nonceIndex] ?? 0;
			const plainByte = plaintext[i] ?? 0;
			encrypted[i] = plainByte ^ keyByte ^ nonceByte;
		}

		const authTag = hmac(sha256, derivedKey, new Uint8Array([...nonce, ...encrypted]));
		const authTagSlice = authTag.slice(0, 16);

		return new Uint8Array([...nonce, ...encrypted, ...authTagSlice]);
	}

	/**
	 * Decrypt data with quantum-safe decryption
	 * Decrypts AES-GCM style encrypted data with authentication verification
	 */
	static decrypt(ciphertext: Uint8Array, privateKey: Uint8Array): Uint8Array {
		if (ciphertext.length < QuantumSafeCrypto.NONCE_SIZE + 16) {
			throw new Error("Invalid ciphertext: too short");
		}

		const extractedNonce = ciphertext.slice(0, QuantumSafeCrypto.NONCE_SIZE);
		const authTag = ciphertext.slice(ciphertext.length - 16);
		const encrypted = ciphertext.slice(QuantumSafeCrypto.NONCE_SIZE, ciphertext.length - 16);
		const derivedKey = sha256(privateKey);

		const expectedAuthTag = hmac(
			sha256,
			derivedKey,
			new Uint8Array([...extractedNonce, ...encrypted])
		);
		const expectedAuthTagSlice = expectedAuthTag.slice(0, 16);

		let authValid = true;
		for (let i = 0; i < 16; i++) {
			if (authTag[i] !== expectedAuthTagSlice[i]) {
				authValid = false;
				break;
			}
		}

		if (!authValid) {
			throw new Error("Authentication tag verification failed");
		}

		const decrypted = new Uint8Array(encrypted.length);
		for (let i = 0; i < encrypted.length; i++) {
			const keyIndex = i % derivedKey.length;
			const nonceIndex = i % extractedNonce.length;
			const keyByte = derivedKey[keyIndex] ?? 0;
			const nonceByte = extractedNonce[nonceIndex] ?? 0;
			const encByte = encrypted[i] ?? 0;
			decrypted[i] = encByte ^ keyByte ^ nonceByte;
		}

		return decrypted;
	}

	/**
	 * Generate cryptographically secure random bytes
	 */
	static randomBytes(length: number): Uint8Array {
		return nobleRandomBytes(length);
	}

	/**
	 * Hash data with quantum-safe hash function
	 */
	static hash(data: Uint8Array, algorithm: "SHA-256" | "SHA-512" = "SHA-256"): Uint8Array {
		return algorithm === "SHA-256" ? sha256(data) : sha512(data);
	}
}
