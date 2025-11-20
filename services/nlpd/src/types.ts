/**
 * NLP Integration Service Types
 */

export interface NLPDRequest {
	text?: string;
	audio?: string; // Base64 encoded audio
	language?: string;
	context?: Record<string, unknown>;
}

export interface NLPDResponse {
	intent: string;
	confidence: number;
	entities: Record<string, string>;
	actions: Action[];
	translation?: string;
	tts?: string; // Base64 encoded audio
}

export interface Action {
	type: string;
	parameters: Record<string, unknown>;
	priority: number;
}

export interface VoiceCommand {
	command: string;
	audio: string; // Base64 encoded audio
	language?: string;
}

export interface TranslationRequest {
	text: string;
	sourceLanguage: string;
	targetLanguage: string;
}

export interface TranslationResponse {
	translatedText: string;
	confidence: number;
}

export interface SpeechToTextRequest {
	audio: string; // Base64 encoded audio
	language?: string;
}

export interface SpeechToTextResponse {
	text: string;
	confidence: number;
}

export interface TextToSpeechRequest {
	text: string;
	language?: string;
	voice?: string;
}

export interface TextToSpeechResponse {
	audio: string; // Base64 encoded audio
}
