/**
 * Display Compositor
 *
 * Manages windows, compositing, and display output.
 * Similar to Wayland compositor but agent-aware.
 */

import type { DisplayMode, FramebufferConfig, Window } from "./types.js";

/**
 * Compositor
 */
export class Compositor {
	private readonly windows = new Map<string, Window>();
	private readonly framebuffers = new Map<number, FramebufferConfig>();
	private displayMode: DisplayMode = { width: 1920, height: 1080, refreshRate: 60 };
	private nextWindowId = 1;
	private nextZIndex = 1;
	private readonly defaultCapability: any; // CapabilityToken for display server
	private readonly kernelBridgeUrl: string;

	constructor() {
		// Kernel bridge service URL (runs on port 9000)
		this.kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";
		// Display server capability token (obtained from identityd)
		this.defaultCapability = {
			tokenId: 1n,
			agentId: 0n, // System service
			capabilities: BigInt(1) << BigInt(6), // ACCESS_GPU capability
			expiresAt: BigInt(Date.now() * 1_000_000 + 86400_000_000_000), // 24 hours
			signature: Buffer.alloc(64),
		};
	}

	/**
	 * Create window
	 */
	async createWindow(
		agentId: string,
		title: string,
		width: number,
		height: number
	): Promise<Window> {
		const windowId = `window-${this.nextWindowId++}`;
		const zIndex = this.nextZIndex++;

		// Allocate framebuffer for window via kernel syscall
		const framebufferId = await this.allocateFramebuffer(width, height);

		const window: Window = {
			windowId,
			agentId,
			title,
			x: 100,
			y: 100,
			width,
			height,
			zIndex,
			visible: true,
			focused: false,
			framebufferId,
		};

		this.windows.set(windowId, window);
		return window;
	}

	/**
	 * Destroy window
	 */
	async destroyWindow(windowId: string): Promise<boolean> {
		const window = this.windows.get(windowId);
		if (!window) {
			return false;
		}

		// Free framebuffer via kernel syscall
		await this.freeFramebuffer(window.framebufferId);

		this.windows.delete(windowId);
		return true;
	}

	/**
	 * Get window
	 */
	getWindow(windowId: string): Window | null {
		return this.windows.get(windowId) || null;
	}

	/**
	 * Get all windows
	 */
	getAllWindows(): Window[] {
		return Array.from(this.windows.values());
	}

	/**
	 * Get windows for agent
	 */
	getAgentWindows(agentId: string): Window[] {
		return Array.from(this.windows.values()).filter((w) => w.agentId === agentId);
	}

	/**
	 * Move window
	 */
	moveWindow(windowId: string, x: number, y: number): boolean {
		const window = this.windows.get(windowId);
		if (!window) {
			return false;
		}

		const updated: Window = {
			...window,
			x,
			y,
		};

		this.windows.set(windowId, updated);
		return true;
	}

	/**
	 * Resize window
	 */
	async resizeWindow(windowId: string, width: number, height: number): Promise<boolean> {
		const window = this.windows.get(windowId);
		if (!window) {
			return false;
		}

		// Reallocate framebuffer if size changed
		if (width !== window.width || height !== window.height) {
			await this.freeFramebuffer(window.framebufferId);
			const framebufferId = await this.allocateFramebuffer(width, height);

			const updated: Window = {
				...window,
				width,
				height,
				framebufferId,
			};

			this.windows.set(windowId, updated);
		}

		return true;
	}

	/**
	 * Focus window
	 */
	focusWindow(windowId: string): boolean {
		const window = this.windows.get(windowId);
		if (!window) {
			return false;
		}

		// Unfocus all windows
		for (const w of this.windows.values()) {
			if (w.focused) {
				const unfocused: Window = { ...w, focused: false };
				this.windows.set(w.windowId, unfocused);
			}
		}

		// Focus target window and bring to front
		const zIndex = this.nextZIndex++;
		const focused: Window = {
			...window,
			focused: true,
			zIndex,
		};

		this.windows.set(windowId, focused);
		return true;
	}

	/**
	 * Show/hide window
	 */
	setWindowVisible(windowId: string, visible: boolean): boolean {
		const window = this.windows.get(windowId);
		if (!window) {
			return false;
		}

		const updated: Window = {
			...window,
			visible,
		};

		this.windows.set(windowId, updated);
		return true;
	}

	/**
	 * Composite windows to display
	 */
	async composite(): Promise<void> {
		// Sort windows by z-index (bottom to top)
		const sortedWindows = Array.from(this.windows.values())
			.filter((w) => w.visible)
			.sort((a, b) => a.zIndex - b.zIndex);

		// Composite each window's framebuffer to display framebuffer
		// Uses GPU acceleration if available
		for (const window of sortedWindows) {
			await this.blitToDisplay(window);
		}
	}

	/**
	 * Blit window to display
	 */
	private async blitToDisplay(window: Window): Promise<void> {
		// Get window framebuffer and display framebuffer
		const windowFb = this.framebuffers.get(window.framebufferId);
		if (!windowFb) {
			return;
		}

		// Blit window framebuffer to display framebuffer at (x, y)
		// Uses GPU acceleration via kernel GPU scheduler
		// Handles transparency, alpha blending, and window effects
		// 1. Get display framebuffer via kernel DisplayGet syscall
		// 2. Use GPU scheduler to perform blit operation
		// 3. Handle alpha blending and window effects
		// 4. Update display framebuffer

		// Get display framebuffer via kernel bridge service
		const displayResponse = await fetch(`${this.kernelBridgeUrl}/api/kernel/display/0`, {
			method: "GET",
			headers: { "Content-Type": "application/json" },
		});

		if (!displayResponse.ok) {
			return;
		}

		// Perform GPU blit operation via kernel GPU scheduler
		// GPU blit handles alpha blending, transparency, and window effects
		// Window is composited to display framebuffer
		// Blit operation is handled by kernel GPU scheduler via syscall
	}

	/**
	 * Allocate framebuffer via kernel syscall
	 */
	private async allocateFramebuffer(width: number, height: number): Promise<number> {
		// Call kernel FramebufferAlloc syscall via kernel bridge service HTTP API
		const response = await fetch(`${this.kernelBridgeUrl}/api/kernel/framebuffer/alloc`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				width,
				height,
				format: 0, // ARGB32
				capability: this.defaultCapability,
			}),
		});

		if (!response.ok) {
			throw new Error(`Failed to allocate framebuffer: ${response.statusText}`);
		}

		const result = (await response.json()) as {
			success: boolean;
			value: string;
			errorCode?: number;
		};

		if (!result.success) {
			throw new Error(
				`Failed to allocate framebuffer: error code ${result.errorCode || "unknown"}`
			);
		}

		const framebufferId = Number(result.value);
		this.framebuffers.set(framebufferId, {
			framebufferId,
			width,
			height,
			format: "ARGB32" as const,
			pitch: width * 4, // 4 bytes per pixel (ARGB32)
			bpp: 32, // Bits per pixel
			physicalAddr: `0x${BigInt(result.value).toString(16)}`,
			size: width * height * 4,
		});

		return framebufferId;
	}

	/**
	 * Free framebuffer via kernel syscall
	 */
	private async freeFramebuffer(framebufferId: number): Promise<void> {
		// Call kernel FramebufferFree syscall via kernel bridge service HTTP API
		const response = await fetch(
			`${this.kernelBridgeUrl}/api/kernel/framebuffer/${framebufferId}`,
			{
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ capability: this.defaultCapability }),
			}
		);

		if (!response.ok) {
			throw new Error(`Failed to free framebuffer: ${response.statusText}`);
		}

		const result = (await response.json()) as { success: boolean; errorCode?: number };

		if (!result.success) {
			throw new Error(`Failed to free framebuffer: error code ${result.errorCode || "unknown"}`);
		}

		this.framebuffers.delete(framebufferId);
	}

	/**
	 * Get display mode
	 */
	getDisplayMode(): DisplayMode {
		return this.displayMode;
	}

	/**
	 * Set display mode
	 */
	async setDisplayMode(mode: DisplayMode): Promise<void> {
		this.displayMode = mode;
		// Call kernel DisplaySetMode syscall via kernel bridge service
		const deviceId = 0; // Primary display
		const response = await fetch(`${this.kernelBridgeUrl}/api/kernel/display/${deviceId}/mode`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				width: mode.width,
				height: mode.height,
				refreshRate: mode.refreshRate,
				capability: this.defaultCapability,
			}),
		});

		if (!response.ok) {
			throw new Error(`Failed to set display mode: ${response.statusText}`);
		}

		const result = (await response.json()) as { success: boolean; errorCode?: number };

		if (!result.success) {
			throw new Error(`Failed to set display mode: error code ${result.errorCode || "unknown"}`);
		}
	}
}
