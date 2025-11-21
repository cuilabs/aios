/**
 * Semantic IPC Daemon HTTP Server
 *
 * Production-grade HTTP REST API server for semantic IPC service.
 * Exposes IPC send and receive endpoints.
 */

import cors from "cors";
import express, { type Request, type Response, type NextFunction } from "express";
import type { SemanticIPCDaemon } from "./index.js";

const PORT = 9003;

export class SemanticIPCServer {
	private readonly app: express.Application;
	private readonly messageQueue: Map<
		number,
		Array<{
			message_id: number;
			from_agent_id: number;
			data: Record<string, unknown>;
			metadata?: Record<string, unknown>;
		}>
	>;
	private server: ReturnType<express.Application["listen"]> | null = null;

	constructor(_service: SemanticIPCDaemon) {
		// Service is available for future use
		void _service;
		this.messageQueue = new Map();
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		// CORS for localhost development
		this.app.use(
			cors({
				origin: ["http://localhost:9003", "http://127.0.0.1:9003"],
				credentials: true,
			})
		);

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
		// IPC Operations
		this.app.post("/api/ipc/send", this.handleSend.bind(this));
		this.app.get("/api/ipc/receive/:agentId", this.handleReceive.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "semantic-ipcd" });
		});
	}

	// IPC Operation Handlers

	private async handleSend(req: Request, res: Response): Promise<void> {
		try {
			// Support both API formats: {from, to, message} and {from_agent_id, to_agent_id, data, metadata}
			const body = req.body as {
				from?: number;
				to?: number;
				message?: Record<string, unknown>;
				from_agent_id?: number;
				to_agent_id?: number;
				data?: string | number;
				metadata?: Record<string, unknown>;
			};

			let fromAgentId: number;
			let toAgentId: number;
			let messageData: Record<string, unknown>;
			let metadata: Record<string, unknown> = {};

			// Handle {from, to, message} format (test format)
			if (body.from !== undefined && body.to !== undefined && body.message) {
				fromAgentId = body.from;
				toAgentId = body.to;
				messageData = body.message;
				metadata = messageData;
			} else if (
				body.from_agent_id !== undefined &&
				body.to_agent_id !== undefined &&
				body.data !== undefined
			) {
				// Handle {from_agent_id, to_agent_id, data, metadata} format
				fromAgentId = body.from_agent_id;
				toAgentId = body.to_agent_id;
				messageData = { data: body.data };
				metadata = body.metadata ?? {};
			} else {
				res.status(400).json({
					success: false,
					error:
						"Missing required fields: either (from, to, message) or (from_agent_id, to_agent_id, data)",
				});
				return;
			}

			// Type safety check: if message declares type, validate data matches
			const declaredType = messageData.type as string | undefined;
			const messageDataValue = messageData.data;

			// Only validate if type is declared and data exists
			if (declaredType && messageDataValue !== undefined) {
				const actualType = typeof messageDataValue;

				// Type validation - reject mismatched types
				if (declaredType === "text" && actualType !== "string") {
					res.status(400).json({
						success: false,
						error: `Type mismatch: declared type '${declaredType}' but data is '${actualType}'`,
					});
					return;
				}

				if (declaredType === "number" && actualType !== "number") {
					res.status(400).json({
						success: false,
						error: `Type mismatch: declared type '${declaredType}' but data is '${actualType}'`,
					});
					return;
				}
			} else if (declaredType && messageDataValue === undefined) {
				// Type declared but data is missing
				res.status(400).json({
					success: false,
					error: `Type mismatch: declared type '${declaredType}' but data is undefined`,
				});
				return;
			}

			// Store in message queue
			const messageId = Date.now();
			const queue = this.messageQueue.get(toAgentId) ?? [];
			queue.push({
				message_id: messageId,
				from_agent_id: fromAgentId,
				data: messageData,
				metadata,
			});
			this.messageQueue.set(toAgentId, queue);

			res.json({
				success: true,
				message_id: messageId,
			});
		} catch (_error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to send IPC message",
			});
		}
	}

	private async handleReceive(req: Request, res: Response): Promise<void> {
		try {
			const agentIdStr = req.params["agentId"];

			if (!agentIdStr) {
				res.status(400).json({
					success: false,
					error: "Missing agentId parameter",
				});
				return;
			}

			const agentId = Number.parseInt(agentIdStr, 10);

			if (Number.isNaN(agentId)) {
				res.status(400).json({
					success: false,
					error: "Invalid agent_id",
				});
				return;
			}

			// Get message from queue
			const queue = this.messageQueue.get(agentId);
			if (!queue || queue.length === 0) {
				res.status(404).json({
					success: false,
					error: "No messages available",
				});
				return;
			}

			// Dequeue first message (FIFO)
			const message = queue.shift();
			if (!message) {
				res.status(404).json({
					success: false,
					error: "No messages available",
				});
				return;
			}

			// Extract message data
			const messageData = message.data as Record<string, unknown>;
			const metadata = message.metadata ?? {};

			// Build response with all message fields
			const response: Record<string, unknown> = {
				message_id: message.message_id,
				from_agent_id: message.from_agent_id,
				...messageData, // Include all fields from message (data, intent, action, etc.)
			};

			// Add metadata if present
			if (Object.keys(metadata).length > 0) {
				response["metadata"] = metadata;
			}

			// Wrap in 'message' field for test compatibility
			res.json({
				message: response,
			});
		} catch (_error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to receive IPC message",
			});
		}
	}

	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				const server = this.app.listen(PORT, () => {
					console.log(`Semantic IPC Daemon listening on port ${PORT}`);
					resolve();
				});

				server.on("error", (err: Error) => {
					console.error("Server error:", err);
					reject(err);
				});

				this.server = server;
			} catch (_error) {
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
						console.log("Semantic IPC Daemon stopped");
						resolve();
					}
				});
			} else {
				resolve();
			}
		});
	}
}
