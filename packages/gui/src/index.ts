/**
 * GUI Agent Package
 *
 * Provides GUI agent implementation for managing windows and UI elements.
 * The GUI agent is a first-class agent that manages the graphical interface.
 */

/**
 * GUI Agent
 *
 * Manages windows, UI elements, and user interactions.
 * Communicates with display server via HTTP API.
 */
export class GUIAgent {
	private readonly displayServerUrl: string;
	private readonly agentId: string;
	private readonly windows = new Map<string, WindowInfo>();

	constructor(agentId: string, displayServerUrl = "http://127.0.0.1:9015") {
		this.agentId = agentId;
		this.displayServerUrl = displayServerUrl;
	}

	/**
	 * Create window
	 */
	async createWindow(title: string, width: number, height: number): Promise<WindowInfo> {
		const response = await fetch(`${this.displayServerUrl}/api/windows/create`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				agentId: this.agentId,
				title,
				width,
				height,
			}),
		});

		if (!response.ok) {
			throw new Error(`Failed to create window: ${response.statusText}`);
		}

		const result = (await response.json()) as { success: boolean; window: WindowInfo };
		const window = result.window;
		this.windows.set(window.windowId, window);
		return window;
	}

	/**
	 * Destroy window
	 */
	async destroyWindow(windowId: string): Promise<void> {
		const response = await fetch(`${this.displayServerUrl}/api/windows/${windowId}`, {
			method: "DELETE",
		});

		if (!response.ok) {
			throw new Error(`Failed to destroy window: ${response.statusText}`);
		}

		this.windows.delete(windowId);
	}

	/**
	 * Move window
	 */
	async moveWindow(windowId: string, x: number, y: number): Promise<void> {
		const response = await fetch(`${this.displayServerUrl}/api/windows/${windowId}/move`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ x, y }),
		});

		if (!response.ok) {
			throw new Error(`Failed to move window: ${response.statusText}`);
		}

		const window = this.windows.get(windowId);
		if (window) {
			window.x = x;
			window.y = y;
		}
	}

	/**
	 * Resize window
	 */
	async resizeWindow(windowId: string, width: number, height: number): Promise<void> {
		const response = await fetch(`${this.displayServerUrl}/api/windows/${windowId}/resize`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ width, height }),
		});

		if (!response.ok) {
			throw new Error(`Failed to resize window: ${response.statusText}`);
		}

		const window = this.windows.get(windowId);
		if (window) {
			window.width = width;
			window.height = height;
		}
	}

	/**
	 * Focus window
	 */
	async focusWindow(windowId: string): Promise<void> {
		const response = await fetch(`${this.displayServerUrl}/api/windows/${windowId}/focus`, {
			method: "POST",
		});

		if (!response.ok) {
			throw new Error(`Failed to focus window: ${response.statusText}`);
		}
	}

	/**
	 * Get windows
	 */
	getWindows(): WindowInfo[] {
		return Array.from(this.windows.values());
	}
}

/**
 * Window info
 */
export interface WindowInfo {
	windowId: string;
	agentId: string;
	title: string;
	x: number;
	y: number;
	width: number;
	height: number;
	zIndex: number;
	visible: boolean;
	focused: boolean;
	framebufferId: number;
}
