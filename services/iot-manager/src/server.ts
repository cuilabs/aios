/**
 * IoT Device Management Service HTTP Server
 */

import cors from "cors";
import express, { type Request, type Response } from "express";
import { IoTManagerEngine } from "./iot_engine.js";
import type {
	DeviceControlRequest,
	DeviceControlResponse,
	DeviceDataRequest,
	DeviceDataResponse,
	DeviceDiscoveryRequest,
	DeviceDiscoveryResponse,
	IoTDevice,
} from "./types.js";

const PORT = 9012;

export class IoTManagerServer {
	private app: express.Application;
	private iotEngine: IoTManagerEngine;

	constructor() {
		this.app = express();
		this.iotEngine = new IoTManagerEngine();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		this.app.use(cors());
		this.app.use(express.json());
	}

	private setupRoutes(): void {
		// Health check
		this.app.get("/health", (req: Request, res: Response) => {
			res.json({ status: "ok", service: "iot-manager" });
		});

		// Discover devices
		this.app.post("/api/iot/discover", async (req: Request, res: Response) => {
			try {
				const request: DeviceDiscoveryRequest = req.body;
				const response: DeviceDiscoveryResponse = await this.iotEngine.discoverDevices(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Control device
		this.app.post("/api/iot/control", async (req: Request, res: Response) => {
			try {
				const request: DeviceControlRequest = req.body;
				const response: DeviceControlResponse = await this.iotEngine.controlDevice(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Get device data
		this.app.post("/api/iot/data", async (req: Request, res: Response) => {
			try {
				const request: DeviceDataRequest = req.body;
				const response: DeviceDataResponse = await this.iotEngine.getDeviceData(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// List devices
		this.app.get("/api/iot/devices", (req: Request, res: Response) => {
			try {
				const devices = this.iotEngine.listDevices();
				res.json({ devices });
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Get device
		this.app.get("/api/iot/devices/:deviceId", (req: Request, res: Response) => {
			try {
				const { deviceId } = req.params;
				const device = this.iotEngine.getDevice(deviceId);
				if (device) {
					res.json(device);
				} else {
					res.status(404).json({ error: "Device not found" });
				}
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Register device
		this.app.post("/api/iot/devices", (req: Request, res: Response) => {
			try {
				const device: IoTDevice = req.body;
				this.iotEngine.registerDevice(device);
				res.json({ success: true, deviceId: device.id });
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`IoT Device Management Service (iot-manager) listening on port ${PORT}`);
		});
	}
}
