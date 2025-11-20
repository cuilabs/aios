/**
 * Predictive Threat Intelligence
 * 
 * Predicts future threats based on historical patterns and trends
 */

import { ThreatEvent, ThreatType } from "./index";

/**
 * Predicted threat
 */
export interface PredictedThreat {
	readonly agentId: string;
	readonly threatType: ThreatType;
	readonly probability: number; // 0.0 to 1.0
	readonly predictedTime: number; // Timestamp when threat is predicted to occur
	readonly confidence: number; // 0.0 to 1.0
	readonly indicators: readonly string[];
}

/**
 * Threat trend
 */
export interface ThreatTrend {
	readonly threatType: ThreatType;
	readonly trend: "increasing" | "decreasing" | "stable";
	readonly rate: number; // Change rate per hour
	readonly prediction: number; // Predicted threat score in next hour
}

/**
 * Pattern learner for threat prediction
 */
class PatternLearner {
	private readonly patterns = new Map<string, ThreatPattern>();

	/**
	 * Learn pattern from threat history
	 */
	learnPattern(agentId: string, history: readonly ThreatEvent[]): ThreatPattern {
		// Analyze threat frequency over time
		const hourlyThreats = this.analyzeHourlyPattern(history);
		const dailyThreats = this.analyzeDailyPattern(history);

		const pattern: ThreatPattern = {
			agentId,
			hourlyPattern: hourlyThreats,
			dailyPattern: dailyThreats,
			threatFrequency: history.length / (this.getTimeSpan(history) / 3_600_000), // Threats per hour
			escalationRate: this.calculateEscalationRate(history),
		};

		this.patterns.set(agentId, pattern);
		return pattern;
	}

	/**
	 * Predict next threat time
	 */
	predictNextThreat(agentId: string): number | null {
		const pattern = this.patterns.get(agentId);
		if (!pattern) {
			return null;
		}

		// Predict based on frequency
		const avgInterval = 3_600_000 / pattern.threatFrequency; // Milliseconds
		return Date.now() + avgInterval;
	}

	private analyzeHourlyPattern(history: readonly ThreatEvent[]): number[] {
		const hourly = new Array(24).fill(0);
		for (const event of history) {
			const hour = new Date(event.timestamp).getHours();
			hourly[hour]++;
		}
		return hourly;
	}

	private analyzeDailyPattern(history: readonly ThreatEvent[]): number[] {
		const daily = new Array(7).fill(0); // 0 = Sunday, 6 = Saturday
		for (const event of history) {
			const day = new Date(event.timestamp).getDay();
			daily[day]++;
		}
		return daily;
	}

	private getTimeSpan(history: readonly ThreatEvent[]): number {
		if (history.length < 2) {
			return 3_600_000; // 1 hour default
		}
		const lastEvent = history[history.length - 1];
		const firstEvent = history[0];
		if (!lastEvent || !firstEvent) {
			return 3_600_000; // 1 hour default
		}
		return lastEvent.timestamp - firstEvent.timestamp;
	}

	private calculateEscalationRate(history: readonly ThreatEvent[]): number {
		if (history.length < 10) {
			return 0;
		}

		// Calculate average threat score over time
		const firstHalf = history.slice(0, history.length / 2);
		const secondHalf = history.slice(history.length / 2);

		const firstAvg =
			firstHalf.reduce((sum, e) => sum + e.score, 0) / firstHalf.length;
		const secondAvg =
			secondHalf.reduce((sum, e) => sum + e.score, 0) / secondHalf.length;

		return secondAvg - firstAvg; // Positive = escalating
	}
}

/**
 * Threat pattern
 */
interface ThreatPattern {
	readonly agentId: string;
	readonly hourlyPattern: readonly number[]; // Threats per hour (0-23)
	readonly dailyPattern: readonly number[]; // Threats per day (0-6)
	readonly threatFrequency: number; // Threats per hour
	readonly escalationRate: number; // Rate of escalation
}

/**
 * Predictive Threat Intelligence
 */
export class PredictiveThreatIntelligence {
	private readonly threatHistory: ThreatEvent[] = [];
	private readonly patternLearner = new PatternLearner();
	private readonly trends = new Map<ThreatType, ThreatTrend>();

	/**
	 * Add threat event to history
	 */
	addThreatEvent(event: ThreatEvent): void {
		this.threatHistory.push(event);

		// Keep last 100000 events
		if (this.threatHistory.length > 100000) {
			this.threatHistory.shift();
		}

		// Update patterns
		this.updatePatterns(event.agentId);
		this.updateTrends();
	}

	/**
	 * Predict potential threats in next time window
	 */
	async predictThreats(timeWindowMs: number): Promise<readonly PredictedThreat[]> {
		const predictions: PredictedThreat[] = [];
		const now = Date.now();
		const endTime = now + timeWindowMs;

		// Group threats by agent
		const agentThreats = new Map<string, ThreatEvent[]>();
		for (const event of this.threatHistory) {
			const agentThreatsList = agentThreats.get(event.agentId) ?? [];
			agentThreatsList.push(event);
			agentThreats.set(event.agentId, agentThreatsList);
		}

		// Predict for each agent
		for (const [agentId, events] of agentThreats) {
			if (events.length < 5) {
				continue; // Need at least 5 events to predict
			}

			const pattern = this.patternLearner.learnPattern(agentId, events);
			const nextThreatTime = this.patternLearner.predictNextThreat(agentId);

			if (nextThreatTime && nextThreatTime <= endTime) {
				// Predict threat
				const recentThreats = events.slice(-10);
				const avgScore =
					recentThreats.reduce((sum, e) => sum + e.score, 0) /
					recentThreats.length;
				const mostCommonType = this.getMostCommonThreatType(events);

				predictions.push({
					agentId,
					threatType: mostCommonType,
					probability: Math.min(avgScore + pattern.escalationRate, 1.0),
					predictedTime: nextThreatTime,
					confidence: this.calculateConfidence(events.length),
					indicators: this.getThreatIndicators(pattern),
				});
			}
		}

		return predictions;
	}

	/**
	 * Analyze threat trends
	 */
	async analyzeTrends(): Promise<readonly ThreatTrend[]> {
		return Array.from(this.trends.values());
	}

	/**
	 * Update patterns for agent
	 */
	private updatePatterns(agentId: string): void {
		const agentThreats = this.threatHistory.filter(
			(e) => e.agentId === agentId
		);
		if (agentThreats.length >= 5) {
			this.patternLearner.learnPattern(agentId, agentThreats);
		}
	}

	/**
	 * Update threat trends
	 */
	private updateTrends(): void {
		// Group by threat type
		const typeGroups = new Map<ThreatType, ThreatEvent[]>();
		for (const event of this.threatHistory) {
			const group = typeGroups.get(event.threatType) ?? [];
			group.push(event);
			typeGroups.set(event.threatType, group);
		}

		// Calculate trends for each type
		for (const [threatType, events] of typeGroups) {
			if (events.length < 10) {
				continue;
			}

			// Split into two halves
			const firstHalf = events.slice(0, events.length / 2);
			const secondHalf = events.slice(events.length / 2);

			const firstRate = firstHalf.length / this.getTimeSpan(firstHalf);
			const secondRate = secondHalf.length / this.getTimeSpan(secondHalf);

			const trend: ThreatTrend = {
				threatType,
				trend:
					secondRate > firstRate * 1.1
						? "increasing"
						: secondRate < firstRate * 0.9
							? "decreasing"
							: "stable",
				rate: (secondRate - firstRate) * 3_600_000, // Per hour
				prediction: this.predictNextHourScore(threatType, events),
			};

			this.trends.set(threatType, trend);
		}
	}

	private getTimeSpan(events: readonly ThreatEvent[]): number {
		if (events.length < 2) {
			return 3_600_000; // 1 hour
		}
		const lastEvent = events[events.length - 1];
		const firstEvent = events[0];
		if (!lastEvent || !firstEvent) {
			return 3_600_000; // 1 hour default
		}
		return lastEvent.timestamp - firstEvent.timestamp;
	}

	private getMostCommonThreatType(events: readonly ThreatEvent[]): ThreatType {
		const counts = new Map<ThreatType, number>();
		for (const event of events) {
			counts.set(event.threatType, (counts.get(event.threatType) ?? 0) + 1);
		}

		let maxCount = 0;
		let mostCommon = ThreatType.Unknown;
		for (const [type, count] of counts) {
			if (count > maxCount) {
				maxCount = count;
				mostCommon = type;
			}
		}

		return mostCommon;
	}

	private calculateConfidence(eventCount: number): number {
		// More events = higher confidence
		if (eventCount < 5) {
			return 0.3;
		}
		if (eventCount < 20) {
			return 0.5 + (eventCount / 20) * 0.3;
		}
		return 0.8 + ((Math.min(eventCount, 100) - 20) / 80) * 0.2;
	}

	private getThreatIndicators(pattern: ThreatPattern): readonly string[] {
		const indicators: string[] = [];

		if (pattern.escalationRate > 0.1) {
			indicators.push("Escalating threat pattern");
		}

		if (pattern.threatFrequency > 10) {
			indicators.push("High threat frequency");
		}

		// Check for peak hours
		const maxHour = pattern.hourlyPattern.indexOf(
			Math.max(...pattern.hourlyPattern)
		);
		if (maxHour >= 0) {
			indicators.push(`Peak threat hour: ${maxHour}:00`);
		}

		return indicators;
	}

	private predictNextHourScore(
		_threatType: ThreatType,
		events: readonly ThreatEvent[]
	): number {
		// Prediction: use recent average
		const recent = events.slice(-20);
		if (recent.length === 0) {
			return 0;
		}

		return recent.reduce((sum, e) => sum + e.score, 0) / recent.length;
	}
}

