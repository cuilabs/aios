/**
 * Autonomous Update Manager Types
 */

export interface Update {
	id: string;
	type: "security" | "feature" | "maintenance" | "critical";
	version: string;
	description: string;
	impact: "low" | "medium" | "high";
	requiresReboot: boolean;
}

export interface UpdateScheduleRequest {
	updates: Update[];
	preferences?: UpdatePreferences;
}

export interface UpdateScheduleResponse {
	schedule: ScheduledUpdate[];
	estimatedImpact: ImpactAssessment;
}

export interface ScheduledUpdate {
	update: Update;
	scheduledTime: number;
	reason: string;
	confidence: number;
}

export interface UpdatePreferences {
	maintenanceWindow?: { start: number; end: number };
	allowReboots?: boolean;
	maxImpact?: "low" | "medium" | "high";
}

export interface ImpactAssessment {
	estimatedDowntime: number;
	affectedServices: string[];
	riskLevel: "low" | "medium" | "high";
}

export interface UpdateImpactRequest {
	update: Update;
}

export interface UpdateImpactResponse {
	impact: ImpactAssessment;
	predictions: ImpactPrediction[];
}

export interface ImpactPrediction {
	metric: string;
	before: number;
	after: number;
	confidence: number;
}

export interface RollbackRequest {
	updateId: string;
	reason?: string;
}

export interface RollbackResponse {
	success: boolean;
	previousVersion: string;
	rollbackTime: number;
}

