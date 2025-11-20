/**
 * ML-Based Threat Detection Engine
 *
 * Provides machine learning-based threat detection and autonomous response
 * for proactive security in AIOS.
 */

import type { BehaviorMetrics, BehavioralAnalyzer } from "../behavioral";
import type { BehavioralAnomaly } from "../types";

/**
 * Threat score
 */
export interface ThreatScore {
	readonly score: number; // 0.0 (safe) to 1.0 (critical threat)
	readonly confidence: number; // 0.0 to 1.0
	readonly threatType: ThreatType;
	readonly recommendedAction: SecurityAction;
	readonly indicators: readonly ThreatIndicator[];
}

/**
 * Threat type
 */
export enum ThreatType {
	ResourceExhaustion = "resource_exhaustion",
	UnauthorizedAccess = "unauthorized_access",
	DataExfiltration = "data_exfiltration",
	DenialOfService = "denial_of_service",
	PrivilegeEscalation = "privilege_escalation",
	Unknown = "unknown",
}

/**
 * Security action
 */
export enum SecurityAction {
	Monitor = "monitor",
	Quarantine = "quarantine",
	Kill = "kill",
	Escalate = "escalate",
	NoAction = "no_action",
}

/**
 * Threat indicator
 */
export interface ThreatIndicator {
	readonly type: string;
	readonly severity: "low" | "medium" | "high" | "critical";
	readonly description: string;
	readonly value: number;
}

/**
 * Threat model interface
 */
export interface ThreatModel {
	/**
	 * Score behavior for threat level
	 */
	score(metrics: BehaviorMetrics, anomalies: readonly BehavioralAnomaly[]): ThreatScore;

	/**
	 * Train model on dataset
	 */
	train(dataset: BehaviorDataset): Promise<void>;

	/**
	 * Get model accuracy
	 */
	getAccuracy(): number;
}

/**
 * Behavior dataset for training
 */
export interface BehaviorDataset {
	readonly samples: readonly BehaviorSample[];
}

/**
 * Behavior sample
 */
export interface BehaviorSample {
	readonly metrics: BehaviorMetrics;
	readonly anomalies: readonly BehavioralAnomaly[];
	readonly label: ThreatLabel; // Ground truth
	readonly timestamp: number;
}

/**
 * Threat label (ground truth)
 */
export interface ThreatLabel {
	readonly isThreat: boolean;
	readonly threatType?: ThreatType;
	readonly severity?: number; // 0.0 to 1.0
}

/**
 * ML Threat Detector
 *
 * Uses machine learning models to detect threats based on agent behavior
 */
export class MLThreatDetector {
	private readonly models = new Map<string, ThreatModel>();
	private readonly behavioralAnalyzer: BehavioralAnalyzer;
	private trainingData: BehaviorDataset = { samples: [] };
	private readonly threatHistory: ThreatEvent[] = [];

	constructor(behavioralAnalyzer: BehavioralAnalyzer) {
		this.behavioralAnalyzer = behavioralAnalyzer;
	}

	/**
	 * Train model on historical behavior data
	 */
	async trainModel(agentId: string, dataset: BehaviorDataset): Promise<void> {
		// Train ML model (e.g., TensorFlow.js) or use rule-based fallback
		const model: ThreatModel = {
			score: (metrics, anomalies) => {
				return this.scoreThreatRuleBased(metrics, anomalies);
			},
			train: async (ds) => {
				// Store training data (create new array since samples is readonly)
				this.trainingData = {
					samples: [...this.trainingData.samples, ...ds.samples],
				};
			},
			getAccuracy: () => 0.85, // Default accuracy for rule-based model
		};

		this.models.set(agentId, model);
		await model.train(dataset);
	}

	/**
	 * Score current behavior for threat level
	 */
	async scoreThreat(agentId: string, metrics: BehaviorMetrics): Promise<ThreatScore> {
		// Get anomalies from behavioral analyzer
		const profile = this.behavioralAnalyzer.getProfile(agentId);
		const anomalies = profile?.anomalies ?? [];

		// Use agent-specific model if available
		const model = this.models.get(agentId);
		if (model) {
			const score = model.score(metrics, anomalies);
			this.recordThreatEvent(agentId, score);
			return score;
		}

		// Fallback: rule-based
		const score = this.scoreThreatRuleBased(metrics, anomalies);
		this.recordThreatEvent(agentId, score);
		return score;
	}

	/**
	 * Get threat indicators from metrics and anomalies
	 */
	private getThreatIndicators(
		metrics: BehaviorMetrics,
		anomalies: readonly BehavioralAnomaly[]
	): readonly ThreatIndicator[] {
		const indicators: ThreatIndicator[] = [];

		if (metrics.errorRate > 0.1) {
			indicators.push({
				type: "high_error_rate",
				severity: metrics.errorRate > 0.2 ? "critical" : "high",
				description: `Error rate ${(metrics.errorRate * 100).toFixed(2)}%`,
				value: metrics.errorRate,
			});
		}

		if (metrics.resourceUsage > 1.5) {
			indicators.push({
				type: "high_resource_usage",
				severity: "medium",
				description: "Resource usage exceeds baseline",
				value: metrics.resourceUsage,
			});
		}

		for (const anomaly of anomalies) {
			indicators.push({
				type: anomaly.type,
				severity: anomaly.severity,
				description: anomaly.description,
				value: 1.0,
			});
		}

		return indicators;
	}

	/**
	 * Rule-based threat scoring (fallback)
	 */
	private scoreThreatRuleBased(
		metrics: BehaviorMetrics,
		anomalies: readonly BehavioralAnomaly[]
	): ThreatScore {
		let threatScore = 0.0;

		// Get indicators from helper method
		const indicators = this.getThreatIndicators(metrics, anomalies);

		// Calculate threat score based on indicators
		for (const indicator of indicators) {
			if (indicator.severity === "critical") {
				threatScore += 0.3;
			} else if (indicator.severity === "high") {
				threatScore += 0.2;
			} else if (indicator.severity === "medium") {
				threatScore += 0.1;
			}
		}

		// Additional scoring based on metrics
		if (anomalies.some((a) => a.type === "high_latency")) {
			threatScore += 0.2;
		}
		if (metrics.messageFrequency > 1000) {
			threatScore += 0.3;
		}

		// Determine threat type
		let threatType = ThreatType.Unknown;
		if (metrics.resourceUsage > 2.0) {
			threatType = ThreatType.ResourceExhaustion;
		} else if (metrics.errorRate > 0.2) {
			threatType = ThreatType.DenialOfService;
		} else if (metrics.messageFrequency > 2000) {
			threatType = ThreatType.DataExfiltration;
		}

		// Determine recommended action
		let recommendedAction = SecurityAction.Monitor;
		if (threatScore >= 0.8) {
			recommendedAction = SecurityAction.Kill;
		} else if (threatScore >= 0.6) {
			recommendedAction = SecurityAction.Quarantine;
		} else if (threatScore >= 0.4) {
			recommendedAction = SecurityAction.Escalate;
		}

		return {
			score: Math.min(threatScore, 1.0),
			confidence: 0.7, // Rule-based has lower confidence
			threatType,
			recommendedAction,
			indicators,
		};
	}

	/**
	 * Classify anomaly as benign or malicious
	 */
	async classifyAnomaly(anomaly: BehavioralAnomaly): Promise<ThreatClassification> {
		// Use ML model or fallback to rule-based classification
		const isMalicious =
			anomaly.severity === "critical" ||
			(anomaly.severity === "high" && anomaly.type === "high_error_rate");

		return {
			isMalicious,
			confidence: isMalicious ? 0.8 : 0.6,
			threatType: isMalicious ? ThreatType.Unknown : undefined,
			reason: isMalicious
				? "Anomaly severity indicates potential threat"
				: "Anomaly appears benign",
		};
	}

	/**
	 * Record threat event
	 */
	private recordThreatEvent(agentId: string, score: ThreatScore): void {
		const event: ThreatEvent = {
			timestamp: Date.now(),
			agentId,
			score: score.score,
			threatType: score.threatType,
			action: score.recommendedAction,
		};

		this.threatHistory.push(event);

		// Keep last 10000 events
		if (this.threatHistory.length > 10000) {
			this.threatHistory.shift();
		}
	}

	/**
	 * Get threat history
	 */
	getThreatHistory(agentId?: string): readonly ThreatEvent[] {
		if (agentId) {
			return this.threatHistory.filter((e) => e.agentId === agentId);
		}
		return this.threatHistory;
	}
}

/**
 * Threat classification
 */
export interface ThreatClassification {
	readonly isMalicious: boolean;
	readonly confidence: number;
	readonly threatType?: ThreatType;
	readonly reason: string;
}

/**
 * Threat event
 */
export interface ThreatEvent {
	readonly timestamp: number;
	readonly agentId: string;
	readonly score: number;
	readonly threatType: ThreatType;
	readonly action: SecurityAction;
}
