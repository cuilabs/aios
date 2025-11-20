/**
 * NLP Integration Service HTTP Server
 */

import express, { type Request, type Response } from "express";
import cors from "cors";
import { NLPEngine } from "./nlp_engine.js";
import type {
	NLPDRequest,
	NLPDResponse,
	TranslationRequest,
	TranslationResponse,
	SpeechToTextRequest,
	SpeechToTextResponse,
	TextToSpeechRequest,
	TextToSpeechResponse,
} from "./types.js";

const PORT = 9007;

export class NLPServer {
	private app: express.Application;
	private nlpEngine: NLPEngine;

	constructor() {
		this.app = express();
		this.nlpEngine = new NLPEngine();
		this.setupMiddleware();
		this.setupRoutes();
	}

	private setupMiddleware(): void {
		this.app.use(cors());
		this.app.use(express.json());
	}

	private setupRoutes(): void {
		// Health check
		this.app.get("/health", (req: Request, res: Response) => {
			res.json({ status: "ok", service: "nlpd" });
		});

		// Process natural language request
		this.app.post("/api/nlp/process", async (req: Request, res: Response) => {
			try {
				const request: NLPDRequest = req.body;
				const response: NLPDResponse = await this.nlpEngine.processRequest(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Translate text
		this.app.post("/api/nlp/translate", async (req: Request, res: Response) => {
			try {
				const request: TranslationRequest = req.body;
				const response: TranslationResponse = await this.nlpEngine.translate(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Speech to text
		this.app.post("/api/nlp/speech-to-text", async (req: Request, res: Response) => {
			try {
				const request: SpeechToTextRequest = req.body;
				const response: SpeechToTextResponse = await this.nlpEngine.speechToText(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});

		// Text to speech
		this.app.post("/api/nlp/text-to-speech", async (req: Request, res: Response) => {
			try {
				const request: TextToSpeechRequest = req.body;
				const response: TextToSpeechResponse = await this.nlpEngine.textToSpeech(request);
				res.json(response);
			} catch (error) {
				res.status(500).json({ error: String(error) });
			}
		});
	}

	public start(): void {
		this.app.listen(PORT, () => {
			console.log(`NLP Integration Service (nlpd) listening on port ${PORT}`);
		});
	}
}

