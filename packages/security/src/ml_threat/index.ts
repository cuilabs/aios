/**
 * ML-Based Threat Detection Engine
 * 
 * Provides machine learning-based threat detection and autonomous response
 * for proactive security in AIOS.
 */

import { BehavioralAnalyzer, BehaviorMetrics, BehavioralAnomaly } from "../behavioral";
import { ThreatDetectorModel, ThreatFeatures, ThreatPrediction as MLThreatPrediction } from "@aios/ml";

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
	private readonly trainingData: BehaviorDataset = { samples: [] };
	private readonly threatHistory: ThreatEvent[] = [];
	private readonly mlModel: ThreatDetectorModel;
	private mlModelInitialized = false;

	constructor(behavioralAnalyzer: BehavioralAnalyzer) {
		this.behavioralAnalyzer = behavioralAnalyzer;
		this.mlModel = new ThreatDetectorModel();
	}

	/**
	 * Train model on historical behavior data
	 */
	async trainModel(agentId: string, dataset: BehaviorDataset): Promise<void> {
		// Train ML model (e.g., TensorFlow.js) or use rule-based fallback
		const model: ThreatModel = {
			score: (metrics, anomalies) => this.scoreThreat(metrics, anomalies),
			train: async (ds) => {
				// Store training data
				this.trainingData.samples.push(...ds.samples);
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
		// Initialize ML model if not already initialized
		if (!this.mlModelInitialized) {
			try {
				await this.mlModel.initialize();
				this.mlModelInitialized = true;
			} catch (error) {
				console.warn("Failed to initialize ML model, using rule-based fallback:", error);
			}
		}

		// Get anomalies from behavioral analyzer
		const profile = this.behavioralAnalyzer.getProfile(agentId);
		const anomalies = profile?.anomalies ?? [];

		// Try ML model first
		if (this.mlModelInitialized) {
			try {
				// Get historical threats for this agent
				const agentThreats = this.threatHistory
					.filter((e) => e.agentId === agentId)
					.slice(-10)
					.map((e) => e.score);

				const features: ThreatFeatures = {
					agentId,
					metrics,
					anomalies,
					historicalThreats: agentThreats.length >= 10 ? agentThreats : [...agentThreats, ...Array(10 - agentThreats.length).fill(0)],
					timeSinceLastThreat: agentThreats.length > 0
						? Date.now() - this.threatHistory
								.filter((e) => e.agentId === agentId)
								.slice(-1)[0]?.timestamp ?? Date.now()
						: Infinity,
				};

				const mlPrediction = await this.mlModel.predict(features);

				// Convert ML prediction to ThreatScore
				const threatTypes = [
					ThreatType.ResourceExhaustion,
					ThreatType.UnauthorizedAccess,
					ThreatType.DataExfiltration,
					ThreatType.DenialOfService,
					ThreatType.PrivilegeEscalation,
					ThreatType.Unknown,
				];

				const securityActions = [
					SecurityAction.Monitor,
					SecurityAction.Quarantine,
					SecurityAction.Kill,
					SecurityAction.Escalate,
					SecurityAction.NoAction,
				];

				const score: ThreatScore = {
					score: mlPrediction.threatScore,
					confidence: mlPrediction.confidence,
					threatType: threatTypes[mlPrediction.threatType] ?? ThreatType.Unknown,
					recommendedAction: securityActions[mlPrediction.recommendedAction] ?? SecurityAction.Monitor,
					indicators: this.getThreatIndicators(metrics, anomalies),
				};

				// Record threat event
				this.recordThreatEvent(agentId, score);

				return score;
			} catch (error) {
				console.warn("ML prediction failed, falling back to rule-based:", error);
			}
		}

		// Fallback to rule-based or agent-specific model
		const model = this.models.get(agentId);
		if (model) {
			const score = model.score(metrics, anomalies);
			this.recordThreatEvent(agentId, score);
			return score;
		}

		// Final fallback: rule-based
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
		const indicators: ThreatIndicator[] = [];

		// Check latency anomaly
		if (anomalies.some((a) => a.type === "high_latency")) {
			threatScore += 0.2;
			indicators.push({
				type: "high_latency",
				severity: "medium",
				description: "Unusually high latency detected",
				value: metrics.averageLatency,
			});
		}

		// Check error rate
		if (metrics.errorRate > 0.1) {
			threatScore += 0.3;
			indicators.push({
				type: "high_error_rate",
				severity: metrics.errorRate > 0.2 ? "critical" : "high",
				description: `Error rate ${(metrics.errorRate * 100).toFixed(2)}% exceeds threshold`,
				value: metrics.errorRate,
			});
		}

		// Check resource usage
		if (metrics.resourceUsage > 1.5) {
			threatScore += 0.2;
			indicators.push({
				type: "high_resource_usage",
				severity: "medium",
				description: "Resource usage exceeds baseline",
				value: metrics.resourceUsage,
			});
		}

		// Check message frequency
		if (metrics.messageFrequency > 1000) {
			threatScore += 0.3;
			indicators.push({
				type: "high_message_frequency",
				severity: "high",
				description: "Unusually high message frequency",
				value: metrics.messageFrequency,
			});
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
			score: threatScore.min(1.0),
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

