# Kernel Tests

This directory contains unit and integration tests for the AIOS kernel.

## Structure

```
tests/kernel/
├── unit/              # Unit tests for individual subsystems
│   ├── test_framework.rs
│   ├── test_memory.rs
│   ├── test_scheduler.rs
│   └── test_policy.rs
├── integration/       # Integration tests
│   ├── test_boot.rs
│   └── test_subsystem_integration.rs
└── Cargo.toml
```

## Running Tests

```bash
cd tests/kernel
cargo test
```

## Test Framework

The test framework provides:
- `TestResult`: Pass, Fail, Skip
- `TestCase`: Individual test case
- `TestSuite`: Collection of tests
- Assert macros: `assert_eq!`, `assert_ne!`, `assert_true!`, `assert_false!`

## Adding Tests

1. Create a new test file in `unit/` or `integration/`
2. Define test functions returning `TestResult`
3. Create a `TestSuite` with your tests
4. Add the suite to the appropriate `mod.rs`

