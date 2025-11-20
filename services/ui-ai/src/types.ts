/**
 * AI-Powered UI/UX Service Types
 */

export interface Gesture {
	type: "eye_tracking" | "hand_motion" | "facial_expression" | "voice";
	data: Record<string, unknown>;
	timestamp: number;
}

export interface GestureRecognitionRequest {
	gesture: Gesture;
	context?: Record<string, unknown>;
}

export interface GestureRecognitionResponse {
	recognized: boolean;
	action?: string;
	confidence: number;
	parameters?: Record<string, unknown>;
}

export interface InterfaceAdjustmentRequest {
	userId: string;
	context: Record<string, unknown>;
	preferences?: Record<string, unknown>;
}

export interface InterfaceAdjustmentResponse {
	adjustments: InterfaceAdjustment[];
	reason: string;
}

export interface InterfaceAdjustment {
	element: string;
	property: string;
	value: unknown;
	priority: number;
}

export interface NotificationFilterRequest {
	notifications: Notification[];
	context: Record<string, unknown>;
}

export interface NotificationFilterResponse {
	filtered: Notification[];
	prioritized: Notification[];
}

export interface Notification {
	id: string;
	type: string;
	priority: number;
	content: string;
	timestamp: number;
}
