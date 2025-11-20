/**
 * Kernel Syscall Bridge
 * 
 * Provides access to kernel syscalls via FFI or HTTP API fallback.
 * Attempts to load kernel library via FFI, falls back to HTTP API if unavailable.
 */

import ffi from "ffi-napi";

/**
 * Syscall result structure (matches kernel SyscallResult)
 */
interface SyscallResult {
	success: boolean;
	value: bigint;
	errorCode: number;
	asyncHandle: bigint;
	dataLen: number;
}

/**
 * Capability token structure (matches kernel CapabilityToken)
 */
interface CapabilityToken {
	tokenId: bigint;
	agentId: bigint;
	capabilities: bigint;
	expiresAt: bigint;
	signature: Buffer;
}

/**
 * Kernel syscall bridge
 * 
 * Provides access to kernel syscalls via FFI or HTTP API.
 */
export class SyscallBridge {
	private readonly kernelLib: any;
	private readonly useFFI: boolean;

	constructor() {
		// Try to load kernel library via FFI
		// Attempts to load libaios.so (Linux), libaios.dylib (macOS), or libaios.dll (Windows)
		try {
			this.kernelLib = ffi.Library("libaios", {
				framebuffer_alloc: ["SyscallResult", ["u32", "u32", "u32", "CapabilityToken"]],
				framebuffer_free: ["SyscallResult", ["u64", "CapabilityToken"]],
				framebuffer_get: ["SyscallResult", ["u64", "CapabilityToken"]],
				display_get: ["SyscallResult", ["u64", "CapabilityToken"]],
				display_set_mode: ["SyscallResult", ["u64", "u32", "u32", "CapabilityToken"]],
				input_read: ["SyscallResult", ["usize", "CapabilityToken"]],
				input_get_devices: ["SyscallResult", ["CapabilityToken"]],
				ipc_send: ["SyscallResult", ["u64", "u64", "array", "array", "CapabilityToken"]],
				ipc_recv: ["SyscallResult", ["u64", "CapabilityToken"]],
			});
			this.useFFI = true;
		} catch {
			// FFI not available, use HTTP API fallback
			this.useFFI = false;
		}
	}

	/**
	 * Allocate framebuffer
	 */
	async allocateFramebuffer(
		width: number,
		height: number,
		format: number,
		capability: CapabilityToken
	): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			// Direct syscall via FFI
			return this.kernelLib.framebuffer_alloc(width, height, format, capability);
		}

		// HTTP API fallback (for development/testing)
		const response = await fetch("http://127.0.0.1:9000/api/kernel/framebuffer/alloc", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ width, height, format, capability }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Free framebuffer
	 */
	async freeFramebuffer(framebufferId: bigint, capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.framebuffer_free(framebufferId, capability);
		}

		const response = await fetch(`http://127.0.0.1:9000/api/kernel/framebuffer/${framebufferId}`, {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ capability }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Get framebuffer config
	 */
	async getFramebuffer(framebufferId: bigint, capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.framebuffer_get(framebufferId, capability);
		}

		const response = await fetch(`http://127.0.0.1:9000/api/kernel/framebuffer/${framebufferId}`, {
			method: "GET",
			headers: { "Content-Type": "application/json" },
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Get display device
	 */
	async getDisplay(deviceId: bigint, capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.display_get(deviceId, capability);
		}

		const response = await fetch(`http://127.0.0.1:9000/api/kernel/display/${deviceId}`, {
			method: "GET",
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Set display mode
	 */
	async setDisplayMode(
		deviceId: bigint,
		width: number,
		height: number,
		capability: CapabilityToken
	): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.display_set_mode(deviceId, width, height, capability);
		}

		const response = await fetch(`http://127.0.0.1:9000/api/kernel/display/${deviceId}/mode`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ width, height }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Read input events
	 */
	async readInput(maxEvents: number, capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.input_read(maxEvents, capability);
		}

		const response = await fetch("http://127.0.0.1:9000/api/kernel/input/read", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ maxEvents, capability }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Get input devices
	 */
	async getInputDevices(capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			return this.kernelLib.input_get_devices(capability);
		}

		const response = await fetch("http://127.0.0.1:9000/api/kernel/input/devices", {
			method: "GET",
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Send IPC message
	 */
	async sendIPC(
		from: bigint,
		to: bigint,
		data: number[],
		metadata: number[],
		capability: CapabilityToken
	): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			// Direct syscall via FFI
			return this.kernelLib.ipc_send(from, to, data, metadata, capability);
		}

		const response = await fetch("http://127.0.0.1:9000/api/kernel/ipc/send", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ from, to, data, metadata, capability }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}

	/**
	 * Receive IPC message
	 */
	async receiveIPC(agentId: bigint, capability: CapabilityToken): Promise<SyscallResult> {
		if (this.useFFI && this.kernelLib) {
			// Direct syscall via FFI
			return this.kernelLib.ipc_recv(agentId, capability);
		}

		const response = await fetch("http://127.0.0.1:9000/api/kernel/ipc/recv", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ agentId, capability }),
		});

		if (!response.ok) {
			return {
				success: false,
				value: 0n,
				errorCode: response.status,
				asyncHandle: 0n,
				dataLen: 0,
			};
		}

		return (await response.json()) as SyscallResult;
	}
}

