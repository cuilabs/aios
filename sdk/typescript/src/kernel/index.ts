/**
 * Kernel API bindings
 */

export class KernelClient {
	/**
	 * Allocate memory
	 */
	async allocateMemory(size: number): Promise<number> {
		// TODO: Call kernel syscall via IPC
		return 0;
	}

	/**
	 * Deallocate memory
	 */
	async deallocateMemory(addr: number, size: number): Promise<void> {
		// TODO: Call kernel syscall
	}
}

export class KernelError extends Error {
	constructor(public code: "OUT_OF_MEMORY" | "INVALID_ADDRESS" | "PERMISSION_DENIED") {
		super();
	}
}
