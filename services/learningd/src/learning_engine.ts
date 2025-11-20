/**
 * Adaptive Learning Engine
 */

import type {
	BehaviorPattern,
	LearningRequest,
	LearningResponse,
	Prediction,
	PredictionRequest,
	PredictionResponse,
	Recommendation,
	UserBehavior,
	UserProfile,
} from "./types.js";

export class LearningEngine {
	private profiles: Map<string, UserProfile> = new Map();
	private behaviorHistory: Map<string, UserBehavior[]> = new Map();

	/**
	 * Learn from user behavior
	 */
	async learn(request: LearningRequest): Promise<LearningResponse> {
		const { userId, behavior } = request;

		// Store behavior history
		if (!this.behaviorHistory.has(userId)) {
			this.behaviorHistory.set(userId, []);
		}
		this.behaviorHistory.get(userId)!.push(behavior);

		// Update user profile
		const profile = this.getOrCreateProfile(userId);
		this.updateProfile(profile, behavior);

		// Generate recommendations
		const recommendations = this.generateRecommendations(profile, behavior);

		return {
			success: true,
			recommendations,
		};
	}

	/**
	 * Predict user actions
	 */
	async predict(request: PredictionRequest): Promise<PredictionResponse> {
		const { userId, context } = request;
		const profile = this.profiles.get(userId);

		if (!profile) {
			return { predictions: [] };
		}

		const predictions: Prediction[] = [];

		// Analyze patterns to predict likely actions
		for (const pattern of profile.patterns) {
			if (this.matchesContext(pattern, context)) {
				predictions.push({
					action: pattern.type,
					probability: pattern.frequency / 100.0,
					confidence: Math.min(1.0, pattern.frequency / 10.0),
				});
			}
		}

		// Sort by probability
		predictions.sort((a, b) => b.probability - a.probability);

		return { predictions: predictions.slice(0, 5) };
	}

	/**
	 * Get user profile
	 */
	getProfile(userId: string): UserProfile | null {
		return this.profiles.get(userId) || null;
	}

	/**
	 * Get or create user profile
	 */
	private getOrCreateProfile(userId: string): UserProfile {
		if (!this.profiles.has(userId)) {
			this.profiles.set(userId, {
				userId,
				preferences: {},
				patterns: [],
				lastUpdated: Date.now(),
			});
		}
		return this.profiles.get(userId)!;
	}

	/**
	 * Update user profile based on behavior
	 */
	private updateProfile(profile: UserProfile, behavior: UserBehavior): void {
		// Extract time of day
		const date = new Date(behavior.timestamp);
		const hour = date.getHours();
		const dayOfWeek = date.getDay();

		// Find or create pattern
		let pattern = profile.patterns.find((p) => p.type === behavior.action);
		if (!pattern) {
			pattern = {
				type: behavior.action,
				frequency: 1,
				timeOfDay: [hour],
				dayOfWeek: [dayOfWeek],
				context: behavior.context,
			};
			profile.patterns.push(pattern);
		} else {
			pattern.frequency += 1;
			if (!pattern.timeOfDay) pattern.timeOfDay = [];
			if (!pattern.timeOfDay.includes(hour)) {
				pattern.timeOfDay.push(hour);
			}
			if (!pattern.dayOfWeek) pattern.dayOfWeek = [];
			if (!pattern.dayOfWeek.includes(dayOfWeek)) {
				pattern.dayOfWeek.push(dayOfWeek);
			}
		}

		profile.lastUpdated = Date.now();
	}

	/**
	 * Generate recommendations
	 */
	private generateRecommendations(profile: UserProfile, behavior: UserBehavior): Recommendation[] {
		const recommendations: Recommendation[] = [];

		// Recommend based on frequent patterns
		for (const pattern of profile.patterns) {
			if (pattern.frequency > 5) {
				recommendations.push({
					type: "suggestion",
					confidence: Math.min(1.0, pattern.frequency / 20.0),
					value: pattern.type,
					reason: `Frequently used action: ${pattern.type}`,
				});
			}
		}

		return recommendations;
	}

	/**
	 * Check if pattern matches context
	 */
	private matchesContext(pattern: BehaviorPattern, context: Record<string, unknown>): boolean {
		if (!pattern.context) return true;

		for (const [key, value] of Object.entries(pattern.context)) {
			if (context[key] !== value) {
				return false;
			}
		}

		return true;
	}
}
