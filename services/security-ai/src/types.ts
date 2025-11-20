/**
 * Security AI Service Types
 */

export interface ThreatDetectionRequest {
	agentId: string;
	metrics: Record<string, unknown>;
	events: SecurityEvent[];
}

export interface ThreatDetectionResponse {
	threatLevel: "low" | "medium" | "high" | "critical";
	confidence: number;
	threats: Threat[];
	recommendations: SecurityRecommendation[];
}

export interface Threat {
	type: string;
	severity: "low" | "medium" | "high" | "critical";
	description: string;
	indicators: string[];
}

export interface SecurityEvent {
	type: string;
	timestamp: number;
	source: string;
	data: Record<string, unknown>;
}

export interface SecurityRecommendation {
	action: string;
	priority: number;
	description: string;
	parameters?: Record<string, unknown>;
}

export interface VulnerabilityScanRequest {
	target: string;
	scanType: "full" | "quick" | "targeted";
}

export interface VulnerabilityScanResponse {
	vulnerabilities: Vulnerability[];
	scanTime: number;
}

export interface Vulnerability {
	id: string;
	severity: "low" | "medium" | "high" | "critical";
	description: string;
	affected: string[];
	remediation: string;
}

export interface ThreatIntelligenceRequest {
	threatType: string;
	context: Record<string, unknown>;
}

export interface ThreatIntelligenceResponse {
	intelligence: ThreatIntelligence[];
	confidence: number;
}

export interface ThreatIntelligence {
	source: string;
	threatType: string;
	description: string;
	indicators: string[];
	recommendations: string[];
}

