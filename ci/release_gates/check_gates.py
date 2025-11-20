#!/usr/bin/env python3
"""
Release Gate Checker
Evaluates release gates based on test results and metrics
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, Any, Optional, Tuple


def load_json_file(filepath: str) -> Optional[Dict[str, Any]]:
    """Load JSON file, return None if file doesn't exist"""
    path = Path(filepath)
    if not path.exists():
        return None
    try:
        with open(path, 'r') as f:
            return json.load(f)
    except (json.JSONDecodeError, IOError) as e:
        print(f"Warning: Failed to load {filepath}: {e}", file=sys.stderr)
        return None


def check_integration_gate(integration_data: Optional[Dict[str, Any]]) -> Tuple[bool, str]:
    """Check integration test gate"""
    if integration_data is None:
        return False, "Integration test results not found"
    
    # Check if tests passed
    summary = integration_data.get('summary', {})
    total = summary.get('total', 0)
    passed = summary.get('passed', 0)
    failed = summary.get('failed', 0)
    
    if total == 0:
        return False, "No integration tests run"
    
    # Require at least 80% pass rate
    pass_rate = passed / total if total > 0 else 0
    if pass_rate < 0.8:
        return False, f"Integration pass rate too low: {pass_rate:.1%} (required: 80%)"
    
    return True, f"Integration tests passed: {passed}/{total} ({pass_rate:.1%})"


def check_performance_gate(perf_data: Optional[Dict[str, Any]]) -> Tuple[bool, str]:
    """Check performance gate"""
    if perf_data is None:
        return True, "Performance data not available (optional)"
    
    # Check if performance metrics are within acceptable ranges
    # This is a placeholder - adjust thresholds as needed
    latency_p50 = perf_data.get('latency', {}).get('p50', 0)
    latency_p95 = perf_data.get('latency', {}).get('p95', 0)
    
    if latency_p95 > 1000:  # 1 second threshold
        return False, f"P95 latency too high: {latency_p95}ms (threshold: 1000ms)"
    
    return True, f"Performance metrics acceptable (P95: {latency_p95}ms)"


def check_chaos_gate(chaos_data: Optional[Dict[str, Any]]) -> Tuple[bool, str]:
    """Check chaos test gate"""
    if chaos_data is None:
        return True, "Chaos test data not available (optional)"
    
    # Check if system recovered from faults
    healing_events = chaos_data.get('healing_events', [])
    if len(healing_events) == 0:
        return True, "No healing events (system stable)"
    
    # Check recovery times
    avg_recovery = sum(e.get('recovery_time_ms', 0) for e in healing_events) / len(healing_events)
    if avg_recovery > 5000:  # 5 second threshold
        return False, f"Average recovery time too high: {avg_recovery:.0f}ms (threshold: 5000ms)"
    
    return True, f"Chaos tests passed: {len(healing_events)} healing events, avg recovery: {avg_recovery:.0f}ms"


def check_model_gate(model_data: Optional[Dict[str, Any]]) -> Tuple[bool, str]:
    """Check model validation gate"""
    if model_data is None:
        return True, "Model validation data not available (optional)"
    
    # Check model metrics
    accuracy = model_data.get('accuracy', 0)
    if accuracy < 0.9:  # 90% accuracy threshold
        return False, f"Model accuracy too low: {accuracy:.1%} (required: 90%)"
    
    return True, f"Model validation passed: accuracy {accuracy:.1%}"


def main():
    parser = argparse.ArgumentParser(description='Check release gates')
    parser.add_argument('--integration', type=str, help='Integration test results JSON')
    parser.add_argument('--perf', type=str, help='Performance test results JSON')
    parser.add_argument('--chaos', type=str, help='Chaos test results JSON')
    parser.add_argument('--models', type=str, help='Model validation results JSON')
    parser.add_argument('--output', type=str, required=True, help='Output JSON file')
    
    args = parser.parse_args()
    
    # Load all test results
    integration_data = load_json_file(args.integration) if args.integration else None
    perf_data = load_json_file(args.perf) if args.perf else None
    chaos_data = load_json_file(args.chaos) if args.chaos else None
    model_data = load_json_file(args.models) if args.models else None
    
    # Check each gate
    gates = {
        'integration': check_integration_gate(integration_data),
        'performance': check_performance_gate(perf_data),
        'chaos': check_chaos_gate(chaos_data),
        'models': check_model_gate(model_data),
    }
    
    # Determine overall status
    all_passed = all(passed for passed, _ in gates.values())
    
    # Build result
    result = {
        'passed': all_passed,
        'gates': {
            name: {
                'passed': passed,
                'message': message
            }
            for name, (passed, message) in gates.items()
        },
        'timestamp': json.dumps({})  # Will be replaced with actual timestamp
    }
    
    # Write result
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump(result, f, indent=2)
    
    # Print summary
    print("Release Gate Check Results:")
    print("=" * 50)
    for name, (passed, message) in gates.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status} {name}: {message}")
    print("=" * 50)
    print(f"Overall: {'✅ PASSED' if all_passed else '❌ FAILED'}")
    
    return 0 if all_passed else 1


if __name__ == '__main__':
    sys.exit(main())

