/**
 * Network Daemon (networkd)
 *
 * Privileged system service for network management:
 * - Network configuration
 * - DHCP client
 * - DNS resolution
 * - Network policy enforcement
 * - Network monitoring
 */

import { SemanticMessageBuilder, SemanticMessageBus } from "@aios/ipc";
import type { DHCPLease, DNSRecord, NetworkConfig, NetworkInterface } from "./types.js";

/**
 * Network Daemon
 *
 * Manages network configuration, DHCP, DNS, and monitoring
 */
export class NetworkDaemon {
	private readonly messageBus: SemanticMessageBus;
	private readonly interfaces: Map<string, NetworkInterface>;
	private readonly dhcpLeases: Map<string, DHCPLease>;
	private readonly dnsCache: Map<string, DNSRecord>;
	private readonly networkConfig: NetworkConfig;

	constructor() {
		this.messageBus = new SemanticMessageBus();
		this.interfaces = new Map();
		this.dhcpLeases = new Map();
		this.dnsCache = new Map();
		this.networkConfig = {
			dhcpEnabled: true,
			dnsServers: ["8.8.8.8", "8.8.4.4"], // Google DNS
			domain: "aios.local",
		};
	}

	/**
	 * Start the daemon
	 */
	async start(): Promise<void> {
		// Register IPC handlers
		this.messageBus.subscribe({ intentType: "network.configure" }, async (message) => {
			const { interfaceId, config } = message.payload as {
				interfaceId: string;
				config: Partial<NetworkInterface>;
			};
			await this.configureInterface(interfaceId, config);
			return { success: true };
		});

		this.messageBus.subscribe({ intentType: "network.dhcp.request" }, async (message) => {
			const { interfaceId } = message.payload as { interfaceId: string };
			const lease = await this.requestDHCPLease(interfaceId);
			return { lease };
		});

		this.messageBus.subscribe({ intentType: "network.dns.resolve" }, async (message) => {
			const { hostname } = message.payload as { hostname: string };
			const address = await this.resolveDNS(hostname);
			return { address };
		});

		// Start DHCP client if enabled
		if (this.networkConfig.dhcpEnabled) {
			this.startDHCPClient();
		}

		// Start DNS resolution service
		this.startDNSService();

		// Start network monitoring
		this.startNetworkMonitoring();
	}

	/**
	 * Configure network interface
	 */
	async configureInterface(interfaceId: string, config: Partial<NetworkInterface>): Promise<void> {
		const iface = this.interfaces.get(interfaceId);
		if (!iface) {
			// Create new interface
			const newInterface: NetworkInterface = {
				interfaceId,
				ipAddress: config.ipAddress ?? "0.0.0.0",
				netmask: config.netmask ?? "255.255.255.0",
				gateway: config.gateway,
				macAddress: config.macAddress ?? new Uint8Array(6),
				status: "down",
				...config,
			};
			this.interfaces.set(interfaceId, newInterface);
		} else {
			// Update existing interface
			Object.assign(iface, config);
		}

		// Publish interface configured event via semantic IPC
		this.messageBus.publish({
			id: `network-config-${Date.now()}`,
			from: "networkd",
			to: "all",
			intent: {
				type: "network.interface.configured",
				action: "configure",
				constraints: {},
				context: {},
				priority: 1,
			},
			payload: {
				interfaceId,
				timestamp: Date.now(),
			},
			timestamp: Date.now(),
			requiresResponse: false,
		});
	}

	/**
	 * Request DHCP lease
	 */
	async requestDHCPLease(interfaceId: string): Promise<DHCPLease> {
		// Send DHCP DISCOVER/DHCP REQUEST packets via kernel network stack
		// Use kernel-bridge service to send DHCP packets
		const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";

		try {
			// Send DHCP DISCOVER packet
			const discoverResponse = await fetch(`${kernelBridgeUrl}/api/kernel/network/dhcp/discover`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ interfaceId }),
			});

			if (discoverResponse.ok) {
				const offer = (await discoverResponse.json()) as {
					ipAddress?: string;
					netmask?: string;
					gateway?: string;
					dnsServers?: string[];
					leaseTime?: number;
				};

				// Send DHCP REQUEST packet
				const requestResponse = await fetch(`${kernelBridgeUrl}/api/kernel/network/dhcp/request`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({
						interfaceId,
						ipAddress: offer.ipAddress,
					}),
				});

				if (requestResponse.ok) {
					const lease: DHCPLease = {
						interfaceId,
						ipAddress: offer.ipAddress || "0.0.0.0",
						netmask: offer.netmask || "255.255.255.0",
						gateway: offer.gateway || "0.0.0.0",
						dnsServers: offer.dnsServers || [],
						leaseTime: offer.leaseTime || 3600,
						obtainedAt: Date.now(),
					};

					this.dhcpLeases.set(interfaceId, lease);
					return lease;
				}
			}
		} catch (error) {
			console.error("DHCP lease request failed:", error);
		}

		// Fallback: return default lease if DHCP fails
		const lease: DHCPLease = {
			interfaceId,
			ipAddress: "0.0.0.0",
			netmask: "255.255.255.0",
			gateway: "192.168.1.1",
			dnsServers: this.networkConfig.dnsServers,
			leaseTime: 3600, // 1 hour
			obtainedAt: Date.now(),
		};

		// Store lease
		this.dhcpLeases.set(interfaceId, lease);

		// Configure interface with lease
		await this.configureInterface(interfaceId, {
			ipAddress: lease.ipAddress,
			netmask: lease.netmask,
			gateway: lease.gateway,
			status: "up",
		});

		return lease;
	}

	/**
	 * Resolve DNS hostname
	 */
	async resolveDNS(hostname: string): Promise<string> {
		// Check cache first
		const cached = this.dnsCache.get(hostname);
		if (cached && cached.expiresAt > Date.now()) {
			return cached.address;
		}

		// Send DNS query to DNS servers via kernel network stack
		const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";

		try {
			// Query DNS servers configured in network config
			for (const dnsServer of this.networkConfig.dnsServers) {
				const response = await fetch(`${kernelBridgeUrl}/api/kernel/network/dns/query`, {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({
						hostname,
						dnsServer,
					}),
				});

				if (response.ok) {
					const result = (await response.json()) as { address?: string };
					if (result.address) {
						const address = result.address;

						// Cache result
						this.dnsCache.set(hostname, {
							hostname,
							address,
							resolvedAt: Date.now(),
							expiresAt: Date.now() + 3600 * 1000, // 1 hour TTL
						});

						return address;
					}
				}
			}
		} catch (error) {
			console.error("DNS resolution failed:", error);
		}

		// Fallback: return default address if DNS fails
		const address = "0.0.0.0";

		return address;
	}

	/**
	 * Start DHCP client
	 */
	private startDHCPClient(): void {
		setInterval(async () => {
			// Renew DHCP leases before expiration
			for (const [interfaceId, lease] of this.dhcpLeases.entries()) {
				const timeUntilExpiry = lease.obtainedAt + lease.leaseTime * 1000 - Date.now();
				if (timeUntilExpiry < 300000) {
					// Renew if less than 5 minutes remaining
					await this.requestDHCPLease(interfaceId);
				}
			}
		}, 60000); // Check every minute
	}

	/**
	 * Start DNS service
	 */
	private startDNSService(): void {
		// DNS resolution is handled on-demand via IPC
		// Could add periodic cache cleanup here
		setInterval(() => {
			// Clean expired DNS cache entries
			for (const [hostname, record] of this.dnsCache.entries()) {
				if (record.expiresAt < Date.now()) {
					this.dnsCache.delete(hostname);
				}
			}
		}, 60000); // Clean every minute
	}

	/**
	 * Start network monitoring
	 */
	private startNetworkMonitoring(): void {
		setInterval(async () => {
			// Monitor network interfaces
			for (const [interfaceId, iface] of this.interfaces.entries()) {
				// Query kernel for actual network statistics via kernel-bridge service
				const kernelBridgeUrl = process.env["KERNEL_BRIDGE_URL"] || "http://127.0.0.1:9000";

				try {
					const response = await fetch(
						`${kernelBridgeUrl}/api/kernel/network/interface/${interfaceId}/stats`,
						{
							method: "GET",
						}
					);

					if (response.ok) {
						const stats = (await response.json()) as {
							bytesSent?: number;
							bytesReceived?: number;
							packetsSent?: number;
							packetsReceived?: number;
							errors?: number;
						};

						// Publish monitoring event with actual statistics
						const monitorMessage = SemanticMessageBuilder.create(
							"networkd",
							"*",
							{
								type: "network.monitor",
								action: "notify",
								constraints: {},
								context: {},
								priority: 1,
							},
							{
								interfaceId,
								stats,
							}
						);
						this.messageBus.publish(monitorMessage);
					}
				} catch (error) {
					console.error(`Failed to query network stats for ${interfaceId}:`, error);
				}
			}
		}, 10000); // Every 10 seconds
	}

	/**
	 * Get network interface
	 */
	getInterface(interfaceId: string): NetworkInterface | undefined {
		return this.interfaces.get(interfaceId);
	}

	/**
	 * Get DHCP lease
	 */
	getDHCPLease(interfaceId: string): DHCPLease | undefined {
		return this.dhcpLeases.get(interfaceId);
	}
}
