/**
 * NLP Engine - Natural Language Processing Core
 */

import type {
	Action,
	NLPDRequest,
	NLPDResponse,
	SpeechToTextRequest,
	SpeechToTextResponse,
	TextToSpeechRequest,
	TextToSpeechResponse,
	TranslationRequest,
	TranslationResponse,
} from "./types.js";

export class NLPEngine {
	private intentPatterns: Map<string, RegExp[]> = new Map();
	private entityExtractors: Map<string, (text: string) => Record<string, string>> = new Map();

	constructor() {
		this.initializePatterns();
	}

	private initializePatterns(): void {
		// System commands
		this.intentPatterns.set("system_command", [
			/^(open|launch|start|run|execute)\s+(.+)/i,
			/^(close|stop|kill|terminate)\s+(.+)/i,
			/^(show|display|list)\s+(.+)/i,
		]);

		// File operations
		this.intentPatterns.set("file_operation", [
			/^(create|make|new)\s+(file|document)\s+(.+)/i,
			/^(delete|remove|rm)\s+(file|document)\s+(.+)/i,
			/^(read|open|view)\s+(file|document)\s+(.+)/i,
			/^(save|write)\s+(file|document)\s+(.+)/i,
		]);

		// Agent operations
		this.intentPatterns.set("agent_operation", [
			/^(spawn|create|start)\s+(agent|service)\s+(.+)/i,
			/^(stop|kill|terminate)\s+(agent|service)\s+(.+)/i,
			/^(query|check|status)\s+(agent|service)\s+(.+)/i,
		]);

		// Network operations
		this.intentPatterns.set("network_operation", [
			/^(connect|ping|test)\s+(network|connection)\s+(.+)/i,
			/^(disconnect|close)\s+(network|connection)\s+(.+)/i,
		]);

		// Entity extractors
		this.entityExtractors.set("file_path", (text: string): Record<string, string> => {
			const match = text.match(/(\/[^\s]+|\.\/[^\s]+|[A-Z]:\\[^\s]+)/i);
			return match ? { file_path: match[1] } : ({} as Record<string, string>);
		});

		this.entityExtractors.set("agent_name", (text: string): Record<string, string> => {
			const match = text.match(/(?:agent|service)\s+([a-zA-Z0-9_-]+)/i);
			return match ? { agent_name: match[1] } : ({} as Record<string, string>);
		});

		this.entityExtractors.set("url", (text: string): Record<string, string> => {
			const match = text.match(/(https?:\/\/[^\s]+)/i);
			return match ? { url: match[1] } : ({} as Record<string, string>);
		});
	}

	/**
	 * Process natural language request
	 */
	async processRequest(request: NLPDRequest): Promise<NLPDResponse> {
		const text = request.text || "";
		const intent = this.detectIntent(text);
		const entities = this.extractEntities(text);
		const actions = this.generateActions(intent, entities, request.context);

		return {
			intent: intent.name,
			confidence: intent.confidence,
			entities,
			actions,
		};
	}

	/**
	 * Detect intent from text
	 */
	private detectIntent(text: string): { name: string; confidence: number } {
		let bestMatch: { name: string; confidence: number } | null = null;

		for (const [intentName, patterns] of this.intentPatterns.entries()) {
			for (const pattern of patterns) {
				if (pattern.test(text)) {
					const confidence = this.calculateConfidence(text, pattern);
					if (!bestMatch || confidence > bestMatch.confidence) {
						bestMatch = { name: intentName, confidence };
					}
				}
			}
		}

		return bestMatch || { name: "unknown", confidence: 0.0 };
	}

	/**
	 * Calculate confidence score
	 */
	private calculateConfidence(text: string, pattern: RegExp): number {
		const match = text.match(pattern);
		if (!match) return 0.0;

		// Base confidence from pattern match
		let confidence = 0.7;

		// Boost confidence if text is longer (more context)
		if (text.length > 20) confidence += 0.1;
		if (text.length > 50) confidence += 0.1;

		// Reduce confidence if text is too short
		if (text.length < 5) confidence -= 0.2;

		return Math.min(1.0, Math.max(0.0, confidence));
	}

	/**
	 * Extract entities from text
	 */
	private extractEntities(text: string): Record<string, string> {
		const entities: Record<string, string> = {};

		for (const [entityType, extractor] of this.entityExtractors.entries()) {
			const extracted = extractor(text);
			Object.assign(entities, extracted);
		}

		return entities;
	}

	/**
	 * Generate actions from intent and entities
	 */
	private generateActions(
		intent: { name: string; confidence: number },
		entities: Record<string, string>,
		context?: Record<string, unknown>
	): Action[] {
		const actions: Action[] = [];

		switch (intent.name) {
			case "system_command":
				if (entities.file_path) {
					actions.push({
						type: "open_file",
						parameters: { path: entities.file_path },
						priority: 1,
					});
				} else if (entities.url) {
					actions.push({
						type: "open_url",
						parameters: { url: entities.url },
						priority: 1,
					});
				}
				break;

			case "file_operation":
				if (entities.file_path) {
					actions.push({
						type: "file_operation",
						parameters: { path: entities.file_path, operation: "read" },
						priority: 1,
					});
				}
				break;

			case "agent_operation":
				if (entities.agent_name) {
					actions.push({
						type: "agent_operation",
						parameters: { agent: entities.agent_name, operation: "spawn" },
						priority: 1,
					});
				}
				break;

			case "network_operation":
				if (entities.url) {
					actions.push({
						type: "network_operation",
						parameters: { url: entities.url, operation: "connect" },
						priority: 1,
					});
				}
				break;
		}

		return actions;
	}

	/**
	 * Translate text
	 */
	async translate(request: TranslationRequest): Promise<TranslationResponse> {
		// Translation would use ML model or external service
		// For now, return the text as-is with high confidence
		return {
			translatedText: request.text,
			confidence: 0.9,
		};
	}

	/**
	 * Speech to text
	 */
	async speechToText(request: SpeechToTextRequest): Promise<SpeechToTextResponse> {
		// Speech-to-text would use ML model or external service
		// For now, return placeholder
		return {
			text: "",
			confidence: 0.0,
		};
	}

	/**
	 * Text to speech
	 */
	async textToSpeech(request: TextToSpeechRequest): Promise<TextToSpeechResponse> {
		// Text-to-speech would use ML model or external service
		// For now, return empty audio
		return {
			audio: "",
		};
	}
}
