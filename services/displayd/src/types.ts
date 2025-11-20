/**
 * Display Server Types
 */

/**
 * Window
 */
export interface Window {
	readonly windowId: string;
	readonly agentId: string;
	readonly title: string;
	readonly x: number;
	readonly y: number;
	readonly width: number;
	readonly height: number;
	readonly zIndex: number;
	readonly visible: boolean;
	readonly focused: boolean;
	readonly framebufferId: number;
}

/**
 * Display mode
 */
export interface DisplayMode {
	readonly width: number;
	readonly height: number;
	readonly refreshRate: number;
}

/**
 * Input event
 */
export interface InputEvent {
	readonly deviceId: number;
	readonly eventType:
		| "keyPress"
		| "keyRelease"
		| "mouseMove"
		| "mouseButton"
		| "mouseWheel"
		| "touch";
	readonly timestamp: number;
	readonly data: InputEventData;
}

/**
 * Input event data
 */
export interface InputEventData {
	readonly keycode?: number;
	readonly scancode?: number;
	readonly x?: number;
	readonly y?: number;
	readonly dx?: number;
	readonly dy?: number;
	readonly button?: number;
	readonly pressed?: boolean;
	readonly pressure?: number;
	readonly touchId?: number;
}

/**
 * Framebuffer configuration
 */
export interface FramebufferConfig {
	readonly framebufferId: number;
	readonly width: number;
	readonly height: number;
	readonly pitch: number;
	readonly bpp: number;
	readonly format: "ARGB32" | "RGB24" | "RGB16" | "RGB8";
	readonly physicalAddr: string;
	readonly size: number;
}
