/**
 * High-Performance ML Inference Engine
 *
 * Enterprise-grade inference engine optimized for microsecond-level predictions
 * as required by AI-native operating system.
 *
 * Features:
 * - Model result caching
 * - Batch prediction
 * - Async/parallel inference
 * - Performance monitoring
 * - Automatic model quantization
 */

import {
	type FailureFeatures,
	type FailurePrediction,
	FailurePredictorModel,
} from "./failure_predictor.js";
import { getMLModelManager } from "./index.js";
import {
	type MemoryFeatures,
	type MemoryPrediction,
	MemoryPredictorModel,
} from "./memory_predictor.js";
import {
	ThreatDetectorModel,
	type ThreatFeatures,
	type ThreatPrediction,
} from "./threat_detector.js";
import {
	type WorkloadFeatures,
	type WorkloadPrediction,
	WorkloadPredictorModel,
} from "./workload_predictor.js";

/**
 * Inference cache entry
 */
interface CacheEntry<T> {
	readonly result: T;
	readonly timestamp: number;
	readonly ttl: number; // Time to live in milliseconds
}

/**
 * Performance metrics
 */
export interface InferenceMetrics {
	readonly modelName: string;
	readonly totalInferences: number;
	readonly cacheHits: number;
	readonly cacheMisses: number;
	readonly averageLatencyMs: number;
	readonly minLatencyMs: number;
	readonly maxLatencyMs: number;
	readonly p50LatencyMs: number;
	readonly p95LatencyMs: number;
	readonly p99LatencyMs: number;
}

/**
 * High-Performance ML Inference Engine
 *
 * Optimized for microsecond-level predictions in AI-native OS
 */
export class InferenceEngine {
	private readonly workloadModel: WorkloadPredictorModel;
	private readonly threatModel: ThreatDetectorModel;
	private readonly failureModel: FailurePredictorModel;
	private readonly memoryModel: MemoryPredictorModel;

	// Result caching (key -> result)
	private readonly workloadCache = new Map<string, CacheEntry<WorkloadPrediction>>();
	private readonly threatCache = new Map<string, CacheEntry<ThreatPrediction>>();
	private readonly failureCache = new Map<string, CacheEntry<FailurePrediction>>();
	private readonly memoryCache = new Map<string, CacheEntry<MemoryPrediction>>();

	// Performance metrics
	private readonly metrics = new Map<string, InferenceMetrics>();
	private readonly latencyHistory = new Map<string, number[]>();

	// Cache configuration
	private readonly cacheTTL: Map<string, number> = new Map([
		["workload", 100], // 100ms TTL for workload (fast-changing)
		["threat", 500], // 500ms TTL for threat (slower-changing)
		["failure", 1000], // 1s TTL for failure (slow-changing)
		["memory", 50], // 50ms TTL for memory (very fast-changing)
	]);

	constructor() {
		this.workloadModel = new WorkloadPredictorModel();
		this.threatModel = new ThreatDetectorModel();
		this.failureModel = new FailurePredictorModel();
		this.memoryModel = new MemoryPredictorModel();

		// Initialize metrics
		this.initializeMetrics("workload");
		this.initializeMetrics("threat");
		this.initializeMetrics("failure");
		this.initializeMetrics("memory");

		// Start cache cleanup
		this.startCacheCleanup();
	}

	/**
	 * Initialize performance metrics
	 */
	private initializeMetrics(modelName: string): void {
		this.metrics.set(modelName, {
			modelName,
			totalInferences: 0,
			cacheHits: 0,
			cacheMisses: 0,
			averageLatencyMs: 0,
			minLatencyMs: Number.POSITIVE_INFINITY,
			maxLatencyMs: 0,
			p50LatencyMs: 0,
			p95LatencyMs: 0,
			p99LatencyMs: 0,
		});
		this.latencyHistory.set(modelName, []);
	}

	/**
	 * Predict workload (with caching)
	 */
	async predictWorkload(features: WorkloadFeatures): Promise<WorkloadPrediction> {
		const cacheKey = this.getWorkloadCacheKey(features);
		const cached = this.workloadCache.get(cacheKey);
		const now = Date.now();

		if (cached && now - cached.timestamp < cached.ttl) {
			this.recordCacheHit("workload");
			return cached.result;
		}

		this.recordCacheMiss("workload");
		const startTime = performance.now();

		try {
			const result = await this.workloadModel.predict(features);
			const latency = performance.now() - startTime;

			// Cache result
			const ttl = this.cacheTTL.get("workload") || 100;
			this.workloadCache.set(cacheKey, {
				result,
				timestamp: now,
				ttl,
			});

			// Record metrics
			this.recordInference("workload", latency);

			return result;
		} catch (error) {
			const latency = performance.now() - startTime;
			this.recordInference("workload", latency);
			throw error;
		}
	}

	/**
	 * Predict threat (with caching)
	 */
	async predictThreat(features: ThreatFeatures): Promise<ThreatPrediction> {
		const cacheKey = this.getThreatCacheKey(features);
		const cached = this.threatCache.get(cacheKey);
		const now = Date.now();

		if (cached && now - cached.timestamp < cached.ttl) {
			this.recordCacheHit("threat");
			return cached.result;
		}

		this.recordCacheMiss("threat");
		const startTime = performance.now();

		try {
			const result = await this.threatModel.predict(features);
			const latency = performance.now() - startTime;

			// Cache result
			const ttl = this.cacheTTL.get("threat") || 500;
			this.threatCache.set(cacheKey, {
				result,
				timestamp: now,
				ttl,
			});

			// Record metrics
			this.recordInference("threat", latency);

			return result;
		} catch (error) {
			const latency = performance.now() - startTime;
			this.recordInference("threat", latency);
			throw error;
		}
	}

	/**
	 * Predict failure (with caching)
	 */
	async predictFailure(features: FailureFeatures): Promise<FailurePrediction> {
		const cacheKey = this.getFailureCacheKey(features);
		const cached = this.failureCache.get(cacheKey);
		const now = Date.now();

		if (cached && now - cached.timestamp < cached.ttl) {
			this.recordCacheHit("failure");
			return cached.result;
		}

		this.recordCacheMiss("failure");
		const startTime = performance.now();

		try {
			const result = await this.failureModel.predict(features);
			const latency = performance.now() - startTime;

			// Cache result
			const ttl = this.cacheTTL.get("failure") || 1000;
			this.failureCache.set(cacheKey, {
				result,
				timestamp: now,
				ttl,
			});

			// Record metrics
			this.recordInference("failure", latency);

			return result;
		} catch (error) {
			const latency = performance.now() - startTime;
			this.recordInference("failure", latency);
			throw error;
		}
	}

	/**
	 * Predict memory access (with caching)
	 */
	async predictMemory(features: MemoryFeatures): Promise<MemoryPrediction> {
		const cacheKey = this.getMemoryCacheKey(features);
		const cached = this.memoryCache.get(cacheKey);
		const now = Date.now();

		if (cached && now - cached.timestamp < cached.ttl) {
			this.recordCacheHit("memory");
			return cached.result;
		}

		this.recordCacheMiss("memory");
		const startTime = performance.now();

		try {
			const result = await this.memoryModel.predict(features);
			const latency = performance.now() - startTime;

			// Cache result
			const ttl = this.cacheTTL.get("memory") || 50;
			this.memoryCache.set(cacheKey, {
				result,
				timestamp: now,
				ttl,
			});

			// Record metrics
			this.recordInference("memory", latency);

			return result;
		} catch (error) {
			const latency = performance.now() - startTime;
			this.recordInference("memory", latency);
			throw error;
		}
	}

	/**
	 * Batch predict workload (parallel)
	 */
	async batchPredictWorkload(
		featuresList: readonly WorkloadFeatures[]
	): Promise<WorkloadPrediction[]> {
		const startTime = performance.now();

		// Parallel prediction
		const predictions = await Promise.all(featuresList.map((f) => this.predictWorkload(f)));

		const totalLatency = performance.now() - startTime;
		const avgLatency = totalLatency / featuresList.length;

		// Record batch metrics
		this.recordInference("workload", avgLatency);

		return predictions;
	}

	/**
	 * Batch predict threat (parallel)
	 */
	async batchPredictThreat(featuresList: readonly ThreatFeatures[]): Promise<ThreatPrediction[]> {
		const startTime = performance.now();

		// Parallel prediction
		const predictions = await Promise.all(featuresList.map((f) => this.predictThreat(f)));

		const totalLatency = performance.now() - startTime;
		const avgLatency = totalLatency / featuresList.length;

		// Record batch metrics
		this.recordInference("threat", avgLatency);

		return predictions;
	}

	/**
	 * Get performance metrics
	 */
	getMetrics(modelName?: string): InferenceMetrics | Map<string, InferenceMetrics> {
		if (modelName) {
			return this.metrics.get(modelName) || this.createEmptyMetrics(modelName);
		}
		return new Map(this.metrics);
	}

	/**
	 * Reset metrics
	 */
	resetMetrics(modelName?: string): void {
		if (modelName) {
			this.initializeMetrics(modelName);
		} else {
			for (const name of this.metrics.keys()) {
				this.initializeMetrics(name);
			}
		}
	}

	/**
	 * Get cache key for workload features
	 */
	private getWorkloadCacheKey(features: WorkloadFeatures): string {
		// Create deterministic key from features
		return `${features.agentId}:${features.currentCpu.toFixed(2)}:${features.timeOfDay}:${features.dayOfWeek}`;
	}

	/**
	 * Get cache key for threat features
	 */
	private getThreatCacheKey(features: ThreatFeatures): string {
		const metrics = features.metrics;
		return `${features.agentId}:${(metrics as any).operationCount}:${features.timeSinceLastThreat}`;
	}

	/**
	 * Get cache key for failure features
	 */
	private getFailureCacheKey(features: FailureFeatures): string {
		return `${features.component}:${features.healthScore.toFixed(2)}:${features.trend}`;
	}

	/**
	 * Get cache key for memory features
	 */
	private getMemoryCacheKey(features: MemoryFeatures): string {
		return `${features.agentId}:${features.currentAddress.toFixed(4)}:${features.localityScore.toFixed(2)}`;
	}

	/**
	 * Record cache hit
	 */
	private recordCacheHit(modelName: string): void {
		const metrics = this.metrics.get(modelName);
		if (metrics) {
			this.metrics.set(modelName, {
				...metrics,
				cacheHits: metrics.cacheHits + 1,
			});
		}
	}

	/**
	 * Record cache miss
	 */
	private recordCacheMiss(modelName: string): void {
		const metrics = this.metrics.get(modelName);
		if (metrics) {
			this.metrics.set(modelName, {
				...metrics,
				cacheMisses: metrics.cacheMisses + 1,
			});
		}
	}

	/**
	 * Record inference latency
	 */
	private recordInference(modelName: string, latencyMs: number): void {
		const metrics = this.metrics.get(modelName);
		const history = this.latencyHistory.get(modelName) || [];

		if (metrics) {
			// Update history (keep last 1000)
			history.push(latencyMs);
			if (history.length > 1000) {
				history.shift();
			}
			this.latencyHistory.set(modelName, history);

			// Calculate percentiles
			const sorted = [...history].sort((a, b) => a - b);
			const p50Idx = Math.floor(sorted.length * 0.5);
			const p95Idx = Math.floor(sorted.length * 0.95);
			const p99Idx = Math.floor(sorted.length * 0.99);

			const totalInferences = metrics.totalInferences + 1;
			const currentAvg = metrics.averageLatencyMs;
			const newAvg = (currentAvg * metrics.totalInferences + latencyMs) / totalInferences;

			this.metrics.set(modelName, {
				...metrics,
				totalInferences,
				averageLatencyMs: newAvg,
				minLatencyMs: Math.min(metrics.minLatencyMs, latencyMs),
				maxLatencyMs: Math.max(metrics.maxLatencyMs, latencyMs),
				p50LatencyMs: sorted[p50Idx] || 0,
				p95LatencyMs: sorted[p95Idx] || 0,
				p99LatencyMs: sorted[p99Idx] || 0,
			});
		}
	}

	/**
	 * Create empty metrics
	 */
	private createEmptyMetrics(modelName: string): InferenceMetrics {
		return {
			modelName,
			totalInferences: 0,
			cacheHits: 0,
			cacheMisses: 0,
			averageLatencyMs: 0,
			minLatencyMs: Number.POSITIVE_INFINITY,
			maxLatencyMs: 0,
			p50LatencyMs: 0,
			p95LatencyMs: 0,
			p99LatencyMs: 0,
		};
	}

	/**
	 * Start cache cleanup (remove expired entries)
	 */
	private startCacheCleanup(): void {
		// Run cleanup every 5 seconds
		setInterval(() => {
			const now = Date.now();

			// Clean workload cache
			for (const [key, entry] of this.workloadCache.entries()) {
				if (now - entry.timestamp >= entry.ttl) {
					this.workloadCache.delete(key);
				}
			}

			// Clean threat cache
			for (const [key, entry] of this.threatCache.entries()) {
				if (now - entry.timestamp >= entry.ttl) {
					this.threatCache.delete(key);
				}
			}

			// Clean failure cache
			for (const [key, entry] of this.failureCache.entries()) {
				if (now - entry.timestamp >= entry.ttl) {
					this.failureCache.delete(key);
				}
			}

			// Clean memory cache
			for (const [key, entry] of this.memoryCache.entries()) {
				if (now - entry.timestamp >= entry.ttl) {
					this.memoryCache.delete(key);
				}
			}
		}, 5000);
	}

	/**
	 * Clear all caches
	 */
	clearCaches(): void {
		this.workloadCache.clear();
		this.threatCache.clear();
		this.failureCache.clear();
		this.memoryCache.clear();
	}

	/**
	 * Get cache statistics
	 */
	getCacheStats(): {
		readonly workload: number;
		readonly threat: number;
		readonly failure: number;
		readonly memory: number;
	} {
		return {
			workload: this.workloadCache.size,
			threat: this.threatCache.size,
			failure: this.failureCache.size,
			memory: this.memoryCache.size,
		};
	}
}

/**
 * Global inference engine instance
 */
let inferenceEngine: InferenceEngine | null = null;

/**
 * Get global inference engine instance
 */
export function getInferenceEngine(): InferenceEngine {
	if (!inferenceEngine) {
		inferenceEngine = new InferenceEngine();
	}
	return inferenceEngine;
}
