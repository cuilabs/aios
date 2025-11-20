/**
 * Boot Log System
 * 
 * Handles writing and reading boot logs for reproducibility checks
 */

import * as fs from "fs/promises";
import * as path from "path";

export class BootLogManager {
	private readonly bootLogDir: string;
	private readonly currentBootLogPath: string;

	constructor(bootLogDir: string = "tests/artifacts/integration/boot_logs") {
		this.bootLogDir = bootLogDir;
		this.currentBootLogPath = path.join(this.bootLogDir, `boot_${Date.now()}.log`);
	}

	/**
	 * Initialize boot log directory
	 */
	async initialize(): Promise<void> {
		try {
			await fs.mkdir(this.bootLogDir, { recursive: true });
		} catch (error) {
			const err = error as NodeJS.ErrnoException;
			if (err.code !== "EEXIST") {
				throw error;
			}
		}
	}

	/**
	 * Write boot log entry
	 */
	async writeEntry(entry: string): Promise<void> {
		await this.initialize();
		const timestamp = new Date().toISOString();
		const logLine = `[${timestamp}] ${entry}\n`;
		await fs.appendFile(this.currentBootLogPath, logLine, "utf-8");
	}

	/**
	 * Read boot log
	 */
	async readBootLog(bootId?: string): Promise<string> {
		await this.initialize();

		let logPath: string;
		if (bootId) {
			logPath = path.join(this.bootLogDir, `boot_${bootId}.log`);
		} else {
			// Get most recent boot log
			const files = await fs.readdir(this.bootLogDir);
			const bootLogs = files.filter((f) => f.startsWith("boot_") && f.endsWith(".log"));
			if (bootLogs.length === 0) {
				throw new Error("No boot logs found");
			}
			bootLogs.sort().reverse();
			logPath = path.join(this.bootLogDir, bootLogs[0]);
		}

		return await fs.readFile(logPath, "utf-8");
	}

	/**
	 * List all boot logs
	 */
	async listBootLogs(): Promise<string[]> {
		await this.initialize();
		try {
			const files = await fs.readdir(this.bootLogDir);
			return files.filter((f) => f.startsWith("boot_") && f.endsWith(".log"));
		} catch {
			return [];
		}
	}
}

