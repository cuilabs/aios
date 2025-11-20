/**
 * Adaptive Learning Service Types
 */

export interface UserBehavior {
	userId: string;
	timestamp: number;
	action: string;
	context: Record<string, unknown>;
	metadata?: Record<string, unknown>;
}

export interface UserProfile {
	userId: string;
	preferences: Record<string, unknown>;
	patterns: BehaviorPattern[];
	lastUpdated: number;
}

export interface BehaviorPattern {
	type: string;
	frequency: number;
	timeOfDay?: number[];
	dayOfWeek?: number[];
	context?: Record<string, unknown>;
}

export interface LearningRequest {
	userId: string;
	behavior: UserBehavior;
}

export interface LearningResponse {
	success: boolean;
	recommendations?: Recommendation[];
}

export interface Recommendation {
	type: string;
	confidence: number;
	value: unknown;
	reason: string;
}

export interface PredictionRequest {
	userId: string;
	context: Record<string, unknown>;
}

export interface PredictionResponse {
	predictions: Prediction[];
}

export interface Prediction {
	action: string;
	probability: number;
	confidence: number;
}
