/**
 * IoT Device Management Service Types
 */

export interface IoTDevice {
	id: string;
	name: string;
	type: string;
	protocol: "mqtt" | "coap" | "http" | "websocket";
	address: string;
	status: "online" | "offline" | "error";
	capabilities: string[];
	metadata?: Record<string, unknown>;
}

export interface DeviceDiscoveryRequest {
	protocol?: string;
	timeout?: number;
}

export interface DeviceDiscoveryResponse {
	devices: IoTDevice[];
	discoveryTime: number;
}

export interface DeviceControlRequest {
	deviceId: string;
	action: string;
	parameters?: Record<string, unknown>;
}

export interface DeviceControlResponse {
	success: boolean;
	result?: unknown;
	error?: string;
}

export interface DeviceDataRequest {
	deviceId: string;
	metric?: string;
	timeRange?: { start: number; end: number };
}

export interface DeviceDataResponse {
	data: DeviceDataPoint[];
}

export interface DeviceDataPoint {
	timestamp: number;
	metric: string;
	value: unknown;
}

