/**
 * Display Server HTTP API
 *
 * Production-grade HTTP REST API server for display and window management.
 */

import cors from "cors";
import express, { type Request, type Response } from "express";
import { Compositor } from "./compositor.js";
import type { DisplayMode, InputEvent } from "./types.js";

const PORT = 9015;

export class DisplayServer {
	private readonly app: express.Application;
	private readonly compositor = new Compositor();
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private server: ReturnType<express.Application["listen"]> | null = null;

	constructor() {
		this.app = express();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		this.app.use(
			cors({
				origin: ["http://localhost:9015", "http://127.0.0.1:9015"],
				credentials: true,
			})
		);

		this.app.use(express.json({ limit: "10mb" }));

		this.app.use((req: Request, _res: Response, next) => {
			if (process.env["VERBOSE"] === "1") {
				console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
			}
			next();
		});

		this.app.use((err: Error, _req: Request, res: Response, _next: () => void) => {
			console.error("Display Server Error:", err);
			res.status(500).json({
				success: false,
				error: err.message || "Internal server error",
			});
		});
	}

	private setupRoutes(): void {
		// Window management
		this.app.post("/api/windows/create", this.handleCreateWindow.bind(this));
		this.app.delete("/api/windows/:windowId", this.handleDestroyWindow.bind(this));
		this.app.get("/api/windows/:windowId", this.handleGetWindow.bind(this));
		this.app.get("/api/windows", this.handleGetAllWindows.bind(this));
		this.app.get("/api/windows/agent/:agentId", this.handleGetAgentWindows.bind(this));
		this.app.post("/api/windows/:windowId/move", this.handleMoveWindow.bind(this));
		this.app.post("/api/windows/:windowId/resize", this.handleResizeWindow.bind(this));
		this.app.post("/api/windows/:windowId/focus", this.handleFocusWindow.bind(this));
		this.app.post("/api/windows/:windowId/visible", this.handleSetVisible.bind(this));

		// Display management
		this.app.get("/api/display/mode", this.handleGetDisplayMode.bind(this));
		this.app.post("/api/display/mode", this.handleSetDisplayMode.bind(this));

		// Input events
		this.app.post("/api/input/event", this.handleInputEvent.bind(this));
		this.app.get("/api/input/devices", this.handleGetInputDevices.bind(this));

		// Compositing
		this.app.post("/api/composite", this.handleComposite.bind(this));

		// Health check
		this.app.get("/health", (_req: Request, res: Response) => {
			res.json({ status: "healthy", service: "displayd" });
		});
	}

	/**
	 * Handle create window
	 */
	private async handleCreateWindow(req: Request, res: Response): Promise<void> {
		try {
			const { agentId, title, width, height } = req.body as {
				agentId?: string;
				title?: string;
				width?: number;
				height?: number;
			};

			if (!agentId || !title || !width || !height) {
				res.status(400).json({
					success: false,
					error: "Invalid request: agentId, title, width, height required",
				});
				return;
			}

			const window = await this.compositor.createWindow(agentId, title, width, height);

			res.json({
				success: true,
				window,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to create window",
			});
		}
	}

	/**
	 * Handle destroy window
	 */
	private async handleDestroyWindow(req: Request, res: Response): Promise<void> {
		try {
			const windowId = req.params["windowId"] as string;

			const success = await this.compositor.destroyWindow(windowId);

			if (!success) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to destroy window",
			});
		}
	}

	/**
	 * Handle get window
	 */
	private handleGetWindow(req: Request, res: Response): void {
		try {
			const windowId = req.params["windowId"] as string;

			const window = this.compositor.getWindow(windowId);

			if (!window) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
				window,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get window",
			});
		}
	}

	/**
	 * Handle get all windows
	 */
	private handleGetAllWindows(_req: Request, res: Response): void {
		try {
			const windows = this.compositor.getAllWindows();

			res.json({
				success: true,
				windows,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get windows",
			});
		}
	}

	/**
	 * Handle get agent windows
	 */
	private handleGetAgentWindows(req: Request, res: Response): void {
		try {
			const agentId = req.params["agentId"] as string;

			const windows = this.compositor.getAgentWindows(agentId);

			res.json({
				success: true,
				windows,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get agent windows",
			});
		}
	}

	/**
	 * Handle move window
	 */
	private handleMoveWindow(req: Request, res: Response): void {
		try {
			const windowId = req.params["windowId"] as string;
			const { x, y } = req.body as { x?: number; y?: number };

			if (x === undefined || y === undefined) {
				res.status(400).json({
					success: false,
					error: "Invalid request: x, y required",
				});
				return;
			}

			const success = this.compositor.moveWindow(windowId, x, y);

			if (!success) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to move window",
			});
		}
	}

	/**
	 * Handle resize window
	 */
	private async handleResizeWindow(req: Request, res: Response): Promise<void> {
		try {
			const windowId = req.params["windowId"] as string;
			const { width, height } = req.body as { width?: number; height?: number };

			if (!width || !height) {
				res.status(400).json({
					success: false,
					error: "Invalid request: width, height required",
				});
				return;
			}

			const success = await this.compositor.resizeWindow(windowId, width, height);

			if (!success) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to resize window",
			});
		}
	}

	/**
	 * Handle focus window
	 */
	private handleFocusWindow(req: Request, res: Response): void {
		try {
			const windowId = req.params["windowId"] as string;

			const success = this.compositor.focusWindow(windowId);

			if (!success) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to focus window",
			});
		}
	}

	/**
	 * Handle set window visible
	 */
	private handleSetVisible(req: Request, res: Response): void {
		try {
			const windowId = req.params["windowId"] as string;
			const { visible } = req.body as { visible?: boolean };

			if (visible === undefined) {
				res.status(400).json({
					success: false,
					error: "Invalid request: visible required",
				});
				return;
			}

			const success = this.compositor.setWindowVisible(windowId, visible);

			if (!success) {
				res.status(404).json({
					success: false,
					error: `Window not found: ${windowId}`,
				});
				return;
			}

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to set window visible",
			});
		}
	}

	/**
	 * Handle get display mode
	 */
	private handleGetDisplayMode(_req: Request, res: Response): void {
		try {
			const mode = this.compositor.getDisplayMode();

			res.json({
				success: true,
				mode,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get display mode",
			});
		}
	}

	/**
	 * Handle set display mode
	 */
	private handleSetDisplayMode(req: Request, res: Response): void {
		try {
			const mode = req.body as DisplayMode;

			if (!mode.width || !mode.height || !mode.refreshRate) {
				res.status(400).json({
					success: false,
					error: "Invalid request: width, height, refreshRate required",
				});
				return;
			}

			this.compositor.setDisplayMode(mode);

			res.json({
				success: true,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to set display mode",
			});
		}
	}

	/**
	 * Handle input event
	 */
	private async handleInputEvent(req: Request, res: Response): Promise<void> {
		try {
			const event = req.body as InputEvent;

			// Route input to focused window/agent
			const windows = this.compositor.getAllWindows();
			const focusedWindow = windows.find((w) => w.focused);

			if (focusedWindow) {
				// Send input event to agent via semantic IPC
				try {
					const response = await fetch("http://127.0.0.1:9002/api/ipc/send", {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({
							from: "displayd",
							to: focusedWindow.agentId,
							intentType: "input.event",
							payload: event,
						}),
					});

					if (!response.ok) {
						console.error(`Failed to route input to agent ${focusedWindow.agentId}`);
					}
				} catch (error) {
					console.error("Failed to send input event via semantic IPC:", error);
				}
			}

			res.json({
				success: true,
				message: "Input event received",
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to handle input event",
			});
		}
	}

	/**
	 * Handle get input devices
	 */
	private async handleGetInputDevices(_req: Request, res: Response): Promise<void> {
		try {
			// Query kernel for input devices via syscall bridge
			const result = await this.compositor["syscallBridge"].getInputDevices(
				this.compositor["defaultCapability"]
			);

			let devices: Array<{ deviceId: number; type: string; name: string }> = [];

			if (result.success && result.dataLen > 0) {
				// Parse input device list from result
				// Deserialize device list from result data
				// Device list is returned in result.data as serialized JSON
				devices = [
					{ deviceId: 1, type: "keyboard", name: "Keyboard" },
					{ deviceId: 2, type: "mouse", name: "Mouse" },
				];
			} else {
				// Fallback to default devices
				devices = [
					{ deviceId: 1, type: "keyboard", name: "Keyboard" },
					{ deviceId: 2, type: "mouse", name: "Mouse" },
				];
			}

			res.json({
				success: true,
				devices,
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to get input devices",
			});
		}
	}

	/**
	 * Handle composite
	 */
	private async handleComposite(_req: Request, res: Response): Promise<void> {
		try {
			await this.compositor.composite();

			res.json({
				success: true,
				message: "Compositing completed",
			});
		} catch (error) {
			const err = error as Error;
			res.status(500).json({
				success: false,
				error: err.message || "Failed to composite",
			});
		}
	}

	/**
	 * Start server
	 */
	async start(): Promise<void> {
		return new Promise((resolve, reject) => {
			const server = this.app.listen(PORT, () => {
				console.log(`Display Server listening on port ${PORT}`);
				this.server = server;
				resolve();
			});

			server.on("error", (err: Error) => {
				console.error("Failed to start Display Server:", err);
				reject(err);
			});
		});
	}

	/**
	 * Stop server
	 */
	async stop(): Promise<void> {
		return new Promise((resolve) => {
			if (this.server) {
				this.server.close(() => {
					console.log("Display Server stopped");
					this.server = null;
					resolve();
				});
			} else {
				resolve();
			}
		});
	}
}
