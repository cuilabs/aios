# Test Scripts Gap Analysis

## Current Status: ⚠️ SCAFFOLDING ONLY

The test scripts in `/scripts` are currently **basic scaffolding** that create placeholder outputs. They do **NOT** yet implement the comprehensive test suites required.

---

## 1. Integration Suite ❌ INCOMPLETE

### Required Tests:
- ✅ **Agent lifecycle**: spawn, checkpoint, migrate, kill, restore
- ❌ **Semantic IPC**: messages delivered, intents preserved, type safety
- ❌ **Memory fabric**: write+read consistency, versioning checks, lease expiry
- ❌ **Capability model**: attempt unauthorized action should be rejected and audited
- ❌ **Boot reproducibility**: boot logs identical across runs (stable ordering)
- ❌ **GPU scheduler**: device claim/release and preemption correctness

### Current Implementation:
- Basic structure exists
- Calls `aios-test-runner` if it exists (doesn't exist yet)
- Creates placeholder `results.json` with only status indicators
- **Missing**: Per-test pass/fail, durations, actual test execution

### Required Output:
```json
{
  "timestamp": "...",
  "tests": [
    {
      "name": "agent_lifecycle_spawn",
      "status": "pass|fail",
      "duration_ms": 123,
      "details": "..."
    },
    // ... all tests with pass/fail and durations
  ]
}
```

---

## 2. Performance Suite ❌ INCOMPLETE

### Required Metrics:
- ❌ **Latency**: P50/P95/P99 per agent type
- ❌ **Throughput**: ops/sec per node
- ❌ **Resource utilization**: CPU, RAM, GPU, IO
- ❌ **Swap activity**: swap-in/out counts per minute
- ❌ **Garbage & healing**: heal ops / minute

### Current Implementation:
- Creates placeholder JSON files with all values set to 0
- No actual metric collection
- No baseline thresholds defined

### Required Output:
```json
{
  "timestamp": "...",
  "latency": {
    "agent_types": {
      "reasoning": { "p50_ms": 10, "p95_ms": 25, "p99_ms": 50 },
      "tool_use": { "p50_ms": 5, "p95_ms": 15, "p99_ms": 30 }
    }
  },
  "throughput": {
    "ops_per_second": 1000,
    "per_node": { "node1": 500, "node2": 500 }
  },
  "resource_utilization": {
    "cpu_percent": 45,
    "ram_mb": 8192,
    "gpu_percent": 30,
    "io_ops_per_sec": 100
  },
  "swap_activity": {
    "swap_in_per_minute": 10,
    "swap_out_per_minute": 5
  },
  "garbage_healing": {
    "heal_ops_per_minute": 2
  },
  "baseline_thresholds": {
    "latency_p99_max_ms": 100,
    "throughput_min_ops_per_sec": 500
  }
}
```

---

## 3. Chaos Suite ❌ INCOMPLETE

### Required Data:
- ❌ **Fault injection traces**: Detailed logs of what faults were injected
- ❌ **Healing events**: Timestamps and confidence scores
- ❌ **Snapshot pre/post verification**: Checksums and state comparison
- ❌ **State divergence details**: What changed, checksums

### Current Implementation:
- Basic fault injection scripts (placeholders)
- Checks for healer logs but doesn't parse them
- No snapshot verification
- No state divergence detection

### Required Output:
```json
{
  "timestamp": "...",
  "fault_injections": [
    {
      "type": "disk_latency",
      "timestamp": "...",
      "parameters": { "latency_ms": 200, "duration_s": 60 },
      "trace": "..."
    }
  ],
  "healing_events": [
    {
      "timestamp": "...",
      "event_type": "disk_recovery",
      "confidence_score": 0.95,
      "details": "..."
    }
  ],
  "snapshots": {
    "pre_fault": {
      "checksum": "abc123...",
      "timestamp": "..."
    },
    "post_fault": {
      "checksum": "def456...",
      "timestamp": "..."
    },
    "verification": {
      "match": true,
      "divergence_details": null
    }
  },
  "state_divergence": {
    "detected": false,
    "checksums": { "before": "...", "after": "..." }
  }
}
```

---

## 4. Model Validation ❌ NOT IMPLEMENTED

### Required Data:
- ❌ **Model card outputs**: last trained date, dataset size, eval metrics
- ❌ **Confusion matrix and ROC**: Full classification metrics
- ❌ **Drift scores**: vs historical baseline

### Current Implementation:
- Referenced in CI workflow but no script exists
- No model validation script in `/scripts`
- No collection of model metrics

### Required Output:
```json
{
  "timestamp": "...",
  "model_card": {
    "last_trained_date": "2025-11-19",
    "dataset_size": 1000000,
    "eval_metrics": {
      "accuracy": 0.95,
      "precision": 0.93,
      "recall": 0.94,
      "f1_score": 0.935
    }
  },
  "confusion_matrix": [[100, 5], [3, 92]],
  "roc": {
    "auc": 0.98,
    "curve": [[0, 0], [0.1, 0.2], ...]
  },
  "drift_scores": {
    "vs_baseline": 0.05,
    "threshold": 0.1,
    "status": "normal"
  }
}
```

---

## What Needs to Be Done

### Phase 1: Create Test Infrastructure
1. Build `aios-test-runner` binary that can execute integration tests
2. Create test harnesses for each test suite
3. Implement metric collection tools

### Phase 2: Implement Integration Tests
1. Agent lifecycle tests (spawn, checkpoint, migrate, kill, restore)
2. Semantic IPC tests (message delivery, intent preservation, type safety)
3. Memory fabric tests (consistency, versioning, lease expiry)
4. Capability model tests (unauthorized action rejection, audit)
5. Boot reproducibility tests
6. GPU scheduler tests

### Phase 3: Implement Performance Collection
1. Latency measurement per agent type (P50/P95/P99)
2. Throughput measurement (ops/sec per node)
3. Resource utilization monitoring (CPU, RAM, GPU, IO)
4. Swap activity tracking
5. Garbage & healing metrics

### Phase 4: Implement Chaos Testing
1. Fault injection with detailed traces
2. Healing event collection with confidence scores
3. Snapshot pre/post verification
4. State divergence detection

### Phase 5: Implement Model Validation
1. Model card generation
2. Confusion matrix and ROC calculation
3. Drift score calculation vs baseline

---

## Current Script Status

| Script | Status | Completeness |
|--------|--------|--------------|
| `build_all.sh` | ✅ Working | 90% - Builds components correctly |
| `run_integration.sh` | ⚠️ Scaffolding | 20% - Structure exists, tests not implemented |
| `run_performance.sh` | ⚠️ Scaffolding | 15% - Placeholder JSON only |
| `run_chaos.sh` | ⚠️ Scaffolding | 25% - Basic fault injection, no detailed collection |
| `build_perf.sh` | ✅ Working | 80% - Builds harnesses if they exist |
| `build_chaos.sh` | ✅ Working | 90% - Creates helper scripts |
| `collect_artifacts.sh` | ✅ Working | 100% - Collects and compresses correctly |

---

## Next Steps

1. **Create test infrastructure** - Build the actual test runners and harnesses
2. **Implement test suites** - Write the actual tests for each category
3. **Implement metric collection** - Add real metric gathering, not placeholders
4. **Add baseline thresholds** - Define performance baselines in repo
5. **Create model validation script** - Implement the missing model validation

The scripts are **ready to be enhanced** but currently only provide the structure and placeholder outputs.

