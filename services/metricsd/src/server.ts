/**
 * Metrics Daemon HTTP Server
 * 
 * Production-grade HTTP REST API server for metrics service.
 * Exposes CPU, memory, IO, and swap metrics endpoints.
 */

import express, { type Request, type Response, type NextFunction } from "express";
import cors from "cors";
import { MetricsDaemon } from "./index.js";
import { readFileSync } from "fs";
import { execSync } from "child_process";

const PORT = 9004;

export class MetricsServer {
	private readonly app: express.Application;
	private readonly service: MetricsDaemon;
	private server: ReturnType<express.Application["listen"]> | null = null;

	constructor(service: MetricsDaemon) {
		this.service = service;
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		// CORS for localhost development
		this.app.use(cors({
			origin: ["http://localhost:9004", "http://127.0.0.1:9004"],
			credentials: true,
		}));

		// JSON body parser
		this.app.use(express.json({ limit: "10mb" }));

		// Request logging
		this.app.use((req: Request, _res: Response, next: NextFunction) => {
			console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
			next();
		});

		// Error handler
		this.app.use((err: Error, _req: Request, res: Response, _next: NextFunction) => {
			console.error("Error:", err);
			res.status(500).json({
				success: false,
				error: err.message || "Internal server error",
			});
		});
	}

	private setupRoutes(): void {
		// Metrics
		this.app.get("/api/metrics/cpu", this.handleGetCPU.bind(this));
		this.app.get("/api/metrics/memory", this.handleGetMemory.bind(this));
		this.app.get("/api/metrics/io", this.handleGetIO.bind(this));
		this.app.get("/api/metrics/swap", this.handleGetSwap.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "metricsd" });
		});
	}

	// Metrics Handlers

	private async handleGetCPU(req: Request, res: Response): Promise<void> {
		try {
			// Try to get from service first
			const systemMetrics = this.service.getSystemMetrics();
			if (systemMetrics) {
				res.json({
					cpu_percent: systemMetrics.cpuUsage,
				});
				return;
			}

			// Fallback: query system directly
			let cpuPercent = 0.0;

			if (process.platform === "linux") {
				try {
					const statContent = readFileSync("/proc/stat", "utf-8");
					const firstLine = statContent.split("\n")[0];
					const fields = firstLine.split(/\s+/);

					if (fields.length >= 8) {
						const user = parseInt(fields[1], 10);
						const nice = parseInt(fields[2], 10);
						const system = parseInt(fields[3], 10);
						const idle = parseInt(fields[4], 10);

						const total = user + nice + system + idle;
						const used = total - idle;

						if (total > 0) {
							cpuPercent = (used / total) * 100.0;
						}
					}
				} catch (error) {
					console.warn("Failed to read /proc/stat:", error);
				}
			} else if (process.platform === "darwin") {
				try {
					const output = execSync("sysctl -n vm.loadavg", { encoding: "utf-8" });
					const load = parseFloat(output.trim());
					cpuPercent = Math.min(load * 100, 100.0);
				} catch (error) {
					console.warn("Failed to get CPU load:", error);
				}
			}

			res.json({
				cpu_percent: cpuPercent,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get CPU metrics",
			});
		}
	}

	private async handleGetMemory(req: Request, res: Response): Promise<void> {
		try {
			// Try to get from service first
			const systemMetrics = this.service.getSystemMetrics();
			if (systemMetrics) {
				res.json({
					ram_mb: Math.floor(systemMetrics.memoryUsage / (1024 * 1024)),
				});
				return;
			}

			// Fallback: query system directly
			let ramMb = 0;

			if (process.platform === "linux") {
				try {
					const meminfo = readFileSync("/proc/meminfo", "utf-8");
					for (const line of meminfo.split("\n")) {
						if (line.startsWith("MemTotal:")) {
							const parts = line.split(/\s+/);
							if (parts.length >= 2) {
								const totalKb = parseInt(parts[1], 10);
								ramMb = Math.floor(totalKb / 1024);
								break;
							}
						}
					}
				} catch (error) {
					console.warn("Failed to read /proc/meminfo:", error);
				}
			} else if (process.platform === "darwin") {
				try {
					const output = execSync("sysctl -n hw.memsize", { encoding: "utf-8" });
					const memBytes = parseInt(output.trim(), 10);
					ramMb = Math.floor(memBytes / (1024 * 1024));
				} catch (error) {
					console.warn("Failed to get memory size:", error);
				}
			}

			res.json({
				ram_mb: ramMb,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get memory metrics",
			});
		}
	}

	private async handleGetIO(req: Request, res: Response): Promise<void> {
		try {
			// Try to get from service first
			const systemMetrics = this.service.getSystemMetrics();
			if (systemMetrics) {
				res.json({
					io_ops_per_sec: systemMetrics.ioThroughput,
				});
				return;
			}

			// Fallback: query system directly
			let ioOpsPerSec = 0.0;

			if (process.platform === "linux") {
				try {
					const diskstats = readFileSync("/proc/diskstats", "utf-8");
					let totalOps = 0;

					for (const line of diskstats.split("\n")) {
						const fields = line.split(/\s+/);
						if (fields.length >= 4) {
							const reads = parseInt(fields[3], 10) || 0;
							totalOps += reads;

							if (fields.length >= 8) {
								const writes = parseInt(fields[7], 10) || 0;
								totalOps += writes;
							}
						}
					}

					// Approximate ops per second (this is a snapshot, not a rate)
					ioOpsPerSec = totalOps / 60.0;
				} catch (error) {
					console.warn("Failed to read /proc/diskstats:", error);
				}
			}

			res.json({
				io_ops_per_sec: ioOpsPerSec,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get IO metrics",
			});
		}
	}

	private async handleGetSwap(req: Request, res: Response): Promise<void> {
		try {
			let swapIn = 0;
			let swapOut = 0;

			if (process.platform === "linux") {
				try {
					const vmstat = readFileSync("/proc/vmstat", "utf-8");
					for (const line of vmstat.split("\n")) {
						if (line.startsWith("pswpin ")) {
							const parts = line.split(/\s+/);
							if (parts.length >= 2) {
								swapIn = parseInt(parts[1], 10) || 0;
							}
						} else if (line.startsWith("pswpout ")) {
							const parts = line.split(/\s+/);
							if (parts.length >= 2) {
								swapOut = parseInt(parts[1], 10) || 0;
							}
						}
					}
				} catch (error) {
					console.warn("Failed to read /proc/vmstat:", error);
				}
			}

			res.json({
				swap_in_per_minute: swapIn,
				swap_out_per_minute: swapOut,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get swap metrics",
			});
		}
	}

	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				const server = this.app.listen(PORT, () => {
					console.log(`Metrics Daemon listening on port ${PORT}`);
					resolve();
				});

				server.on("error", (err: Error) => {
					console.error("Server error:", err);
					reject(err);
				});

				this.server = server;
			} catch (error) {
				reject(error);
			}
		});
	}

	async stop(): Promise<void> {
		return new Promise((resolve, reject) => {
			if (this.server) {
				this.server.close((err?: Error) => {
					if (err) {
						reject(err);
					} else {
						console.log("Metrics Daemon stopped");
						resolve();
					}
				});
			} else {
				resolve();
			}
		});
	}
}

