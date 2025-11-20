/**
 * Network Daemon Types
 */

export interface NetworkConfig {
	readonly dhcpEnabled: boolean;
	readonly dnsServers: readonly string[];
	readonly domain: string;
}

export interface NetworkInterface {
	readonly interfaceId: string;
	readonly ipAddress: string;
	readonly netmask: string;
	readonly gateway?: string;
	readonly macAddress: Uint8Array;
	readonly status: "up" | "down";
	readonly mtu?: number;
}

export interface DHCPLease {
	readonly interfaceId: string;
	readonly ipAddress: string;
	readonly netmask: string;
	readonly gateway: string;
	readonly dnsServers: readonly string[];
	readonly leaseTime: number; // Seconds
	readonly obtainedAt: number;
}

export interface DNSRecord {
	readonly hostname: string;
	readonly address: string;
	readonly resolvedAt: number;
	readonly expiresAt: number;
}
