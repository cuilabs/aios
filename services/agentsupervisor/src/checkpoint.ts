/**
 * Agent Checkpoint System
 *
 * Handles saving and restoring agent state to/from disk
 */

import * as path from "path";
import { fileURLToPath } from "url";
import * as fs from "fs/promises";
import type { AgentStatus } from "./types.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export interface CheckpointMetadata {
	checkpointId: string;
	agentId: string;
	timestamp: number;
	state: "loaded" | "running" | "stopped" | "failed";
	resourceUsage: {
		cpu: number;
		memory: number;
		network: number;
		io: number;
	};
	startedAt?: number;
}

export class CheckpointManager {
	private readonly checkpointDir: string;

	constructor(checkpointDir?: string) {
		// Default to tests/artifacts/integration/checkpoints if not provided
		// But allow override via environment variable or parameter
		// Use absolute path from repo root
		const defaultPath = checkpointDir || process.env["CHECKPOINT_DIR"];
		if (defaultPath) {
			// If path is absolute, use as-is; otherwise resolve from repo root
			this.checkpointDir = path.isAbsolute(defaultPath)
				? defaultPath
				: path.resolve(process.cwd(), "..", "..", defaultPath);
		} else {
			// Default: resolve from current working directory (should be services/agentsupervisor)
			// Go up to repo root, then to tests/artifacts/integration/checkpoints
			const fromCwd = path.resolve(
				process.cwd(),
				"..",
				"..",
				"tests",
				"artifacts",
				"integration",
				"checkpoints"
			);
			// Use process.cwd() path (services are typically started from their directory)
			this.checkpointDir = fromCwd;
		}
	}

	/**
	 * Initialize checkpoint directory
	 */
	async initialize(): Promise<void> {
		try {
			await fs.mkdir(this.checkpointDir, { recursive: true });
		} catch (error) {
			// Directory might already exist, that's fine
			const err = error as NodeJS.ErrnoException;
			if (err.code !== "EEXIST") {
				console.error(`Failed to create checkpoint directory ${this.checkpointDir}:`, err);
				throw error;
			}
		}
		// Ensure directory is writable
		try {
			await fs.access(this.checkpointDir, fs.constants.W_OK);
		} catch {
			// Directory might not be writable, but we'll try anyway
		}
	}

	/**
	 * Create checkpoint for agent
	 */
	async createCheckpoint(agentId: string, status: AgentStatus): Promise<string> {
		await this.initialize();

		const checkpointId = `checkpoint_${agentId}_${Date.now()}`;
		const checkpointPath = path.join(this.checkpointDir, `${checkpointId}.json`);

		const metadata: CheckpointMetadata = {
			checkpointId,
			agentId: status.agentId,
			timestamp: Date.now(),
			state: status.status,
			resourceUsage: {
				cpu: status.resourceUsage.cpu,
				memory: status.resourceUsage.memory,
				network: status.resourceUsage.network,
				io: status.resourceUsage.io,
			},
			startedAt: status.startedAt,
		};

		await fs.writeFile(checkpointPath, JSON.stringify(metadata, null, 2), "utf-8");

		return checkpointId;
	}

	/**
	 * Restore agent from checkpoint
	 */
	async restoreCheckpoint(checkpointId: string): Promise<CheckpointMetadata> {
		// Try with .json extension first, then without (for backward compatibility)
		let checkpointPath = path.join(this.checkpointDir, `${checkpointId}.json`);
		try {
			const data = await fs.readFile(checkpointPath, "utf-8");
			const metadata = JSON.parse(data) as CheckpointMetadata;
			return metadata;
		} catch (error) {
			const err = error as NodeJS.ErrnoException;
			if (err.code === "ENOENT") {
				// Try without extension
				checkpointPath = path.join(this.checkpointDir, checkpointId);
				try {
					const data = await fs.readFile(checkpointPath, "utf-8");
					const metadata = JSON.parse(data) as CheckpointMetadata;
					return metadata;
				} catch (error2) {
					const err2 = error2 as NodeJS.ErrnoException;
					if (err2.code === "ENOENT") {
						throw new Error(`Checkpoint not found: ${checkpointId}`);
					}
					throw error2;
				}
			}
			throw error;
		}
	}

	/**
	 * List all checkpoints
	 */
	async listCheckpoints(): Promise<string[]> {
		await this.initialize();

		try {
			const files = await fs.readdir(this.checkpointDir);
			return files
				.filter((file) => file.startsWith("checkpoint_") && file.endsWith(".json"))
				.map((file) => file.replace(/\.json$/, "")); // Return checkpoint ID without extension
		} catch (error) {
			return [];
		}
	}

	/**
	 * Delete checkpoint
	 */
	async deleteCheckpoint(checkpointId: string): Promise<void> {
		// Try with .json extension first, then without
		let checkpointPath = path.join(this.checkpointDir, `${checkpointId}.json`);
		try {
			await fs.unlink(checkpointPath);
		} catch (error) {
			const err = error as NodeJS.ErrnoException;
			if (err.code === "ENOENT") {
				// Try without extension
				checkpointPath = path.join(this.checkpointDir, checkpointId);
				try {
					await fs.unlink(checkpointPath);
				} catch (error2) {
					const err2 = error2 as NodeJS.ErrnoException;
					if (err2.code !== "ENOENT") {
						throw error2;
					}
				}
			} else {
				throw error;
			}
		}
	}

	/**
	 * Check if checkpoint exists
	 */
	async checkpointExists(checkpointId: string): Promise<boolean> {
		// Try with .json extension first, then without
		let checkpointPath = path.join(this.checkpointDir, `${checkpointId}.json`);
		try {
			await fs.access(checkpointPath);
			return true;
		} catch {
			checkpointPath = path.join(this.checkpointDir, checkpointId);
			try {
				await fs.access(checkpointPath);
				return true;
			} catch {
				return false;
			}
		}
	}
}
