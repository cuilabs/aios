/**
 * Behavioral anomaly detection
 * Monitors agent behavior patterns and detects anomalies
 */

import type { BehavioralAnomaly, BehavioralProfile } from "../types.js";

export interface BehaviorMetrics {
	readonly operationCount: number;
	readonly averageLatency: number;
	readonly errorRate: number;
	readonly resourceUsage: number;
	readonly messageFrequency: number;
}

/**
 * Behavioral analyzer
 * Tracks and analyzes agent behavior patterns
 */
export class BehavioralAnalyzer {
	private readonly profiles = new Map<string, BehavioralProfile>();
	private readonly metricsHistory = new Map<string, BehaviorMetrics[]>();
	private readonly anomalyThresholds = {
		latency: 2.0, // 2x baseline
		errorRate: 0.1, // 10% error rate
		resourceUsage: 1.5, // 1.5x baseline
		messageFrequency: 3.0, // 3x baseline
	};

	/**
	 * Update behavioral profile
	 */
	updateProfile(agentId: string, metrics: BehaviorMetrics): BehavioralProfile {
		const history = this.metricsHistory.get(agentId) ?? [];
		history.push(metrics);

		// Keep last 100 metrics
		if (history.length > 100) {
			history.shift();
		}
		this.metricsHistory.set(agentId, history);

		// Calculate baseline
		const baseline = this.calculateBaseline(history);

		// Detect anomalies
		const anomalies = this.detectAnomalies(agentId, metrics, baseline);

		// Update patterns
		const patterns = this.extractPatterns(history);

		const profile: BehavioralProfile = {
			agentId,
			patterns,
			baseline,
			anomalies,
			lastUpdated: Date.now(),
		};

		this.profiles.set(agentId, profile);
		return profile;
	}

	/**
	 * Get behavioral profile
	 */
	getProfile(agentId: string): BehavioralProfile | null {
		return this.profiles.get(agentId) ?? null;
	}

	/**
	 * Check if agent behavior is anomalous
	 */
	isAnomalous(agentId: string, metrics: BehaviorMetrics): boolean {
		const profile = this.profiles.get(agentId);
		if (!profile) {
			return false;
		}

		const anomalies = this.detectAnomalies(agentId, metrics, profile.baseline);
		return anomalies.length > 0;
	}

	/**
	 * Calculate baseline from history
	 */
	private calculateBaseline(history: BehaviorMetrics[]): Readonly<Record<string, number>> {
		if (history.length === 0) {
			return {};
		}

		const sums = {
			operationCount: 0,
			averageLatency: 0,
			errorRate: 0,
			resourceUsage: 0,
			messageFrequency: 0,
		};

		for (const metrics of history) {
			sums.operationCount += metrics.operationCount;
			sums.averageLatency += metrics.averageLatency;
			sums.errorRate += metrics.errorRate;
			sums.resourceUsage += metrics.resourceUsage;
			sums.messageFrequency += metrics.messageFrequency;
		}

		const count = history.length;
		return {
			operationCount: sums.operationCount / count,
			averageLatency: sums.averageLatency / count,
			errorRate: sums.errorRate / count,
			resourceUsage: sums.resourceUsage / count,
			messageFrequency: sums.messageFrequency / count,
		};
	}

	/**
	 * Detect anomalies in current metrics
	 */
	private detectAnomalies(
		_agentId: string,
		metrics: BehaviorMetrics,
		baseline: Readonly<Record<string, number>>
	): readonly BehavioralAnomaly[] {
		const anomalies: BehavioralAnomaly[] = [];

		// Check latency anomaly
		// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
		const baselineLatency = baseline["averageLatency"] ?? 0;
		if (
			baselineLatency > 0 &&
			metrics.averageLatency > baselineLatency * this.anomalyThresholds.latency
		) {
			anomalies.push({
				type: "high_latency",
				severity: metrics.averageLatency > baselineLatency * 3 ? "critical" : "high",
				description: `Latency ${metrics.averageLatency.toFixed(2)}ms exceeds baseline ${baselineLatency.toFixed(2)}ms`,
				timestamp: Date.now(),
				metrics: { latency: metrics.averageLatency, baseline: baselineLatency },
			});
		}

		// Check error rate anomaly
		if (metrics.errorRate > this.anomalyThresholds.errorRate) {
			anomalies.push({
				type: "high_error_rate",
				severity: metrics.errorRate > 0.2 ? "critical" : "high",
				description: `Error rate ${(metrics.errorRate * 100).toFixed(2)}% exceeds threshold`,
				timestamp: Date.now(),
				metrics: { errorRate: metrics.errorRate },
			});
		}

		// Check resource usage anomaly
		// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
		const baselineResourceUsage = baseline["resourceUsage"] ?? 0;
		if (
			baselineResourceUsage > 0 &&
			metrics.resourceUsage > baselineResourceUsage * this.anomalyThresholds.resourceUsage
		) {
			anomalies.push({
				type: "high_resource_usage",
				severity: "medium",
				description: "Resource usage exceeds baseline",
				timestamp: Date.now(),
				metrics: { resourceUsage: metrics.resourceUsage, baseline: baselineResourceUsage },
			});
		}

		// Check message frequency anomaly
		// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
		const baselineMessageFrequency = baseline["messageFrequency"] ?? 0;
		if (
			baselineMessageFrequency > 0 &&
			metrics.messageFrequency > baselineMessageFrequency * this.anomalyThresholds.messageFrequency
		) {
			anomalies.push({
				type: "high_message_frequency",
				severity: "medium",
				description: "Message frequency exceeds baseline",
				timestamp: Date.now(),
				metrics: { messageFrequency: metrics.messageFrequency, baseline: baselineMessageFrequency },
			});
		}

		return anomalies;
	}

	/**
	 * Extract patterns from history
	 * Performs statistical analysis on behavior metrics to identify patterns
	 */
	private extractPatterns(history: BehaviorMetrics[]): Readonly<Record<string, number>> {
		if (history.length === 0) {
			return {};
		}

		const sum = history.reduce(
			(acc, m) => ({
				operationCount: acc.operationCount + m.operationCount,
				averageLatency: acc.averageLatency + m.averageLatency,
				errorRate: acc.errorRate + m.errorRate,
				resourceUsage: acc.resourceUsage + m.resourceUsage,
				messageFrequency: acc.messageFrequency + m.messageFrequency,
			}),
			{
				operationCount: 0,
				averageLatency: 0,
				errorRate: 0,
				resourceUsage: 0,
				messageFrequency: 0,
			}
		);

		const count = history.length;
		const mean = {
			operationCount: sum.operationCount / count,
			averageLatency: sum.averageLatency / count,
			errorRate: sum.errorRate / count,
			resourceUsage: sum.resourceUsage / count,
			messageFrequency: sum.messageFrequency / count,
		};

		const variance = history.reduce(
			(acc, m) => ({
				operationCount: acc.operationCount + (m.operationCount - mean.operationCount) ** 2,
				averageLatency: acc.averageLatency + (m.averageLatency - mean.averageLatency) ** 2,
				errorRate: acc.errorRate + (m.errorRate - mean.errorRate) ** 2,
				resourceUsage: acc.resourceUsage + (m.resourceUsage - mean.resourceUsage) ** 2,
				messageFrequency: acc.messageFrequency + (m.messageFrequency - mean.messageFrequency) ** 2,
			}),
			{
				operationCount: 0,
				averageLatency: 0,
				errorRate: 0,
				resourceUsage: 0,
				messageFrequency: 0,
			}
		);

		const stdDev = {
			operationCount: Math.sqrt(variance.operationCount / count),
			averageLatency: Math.sqrt(variance.averageLatency / count),
			errorRate: Math.sqrt(variance.errorRate / count),
			resourceUsage: Math.sqrt(variance.resourceUsage / count),
			messageFrequency: Math.sqrt(variance.messageFrequency / count),
		};

		return {
			avgOperations: mean.operationCount,
			avgLatency: mean.averageLatency,
			stdDevOperations: stdDev.operationCount,
			stdDevLatency: stdDev.averageLatency,
			avgErrorRate: mean.errorRate,
			avgResourceUsage: mean.resourceUsage,
			avgMessageFrequency: mean.messageFrequency,
		};
	}
}
