# Snapshot Testing Guide

## Overview
This project uses [insta](https://insta.rs/) for snapshot testing. Snapshots capture test outputs and make it easy to review changes.

## Running Snapshot Tests

### Normal Test Run
```bash
cargo test
```

If outputs have changed, tests will fail and show diffs.

### Reviewing Snapshots
```bash
cargo insta review
```

This opens an interactive session to accept/reject changes.

### Accepting All Changes
```bash
cargo insta accept
```

### Test Specific Snapshots
```bash
cargo insta test --test snapshot_example_test
```

## Writing Snapshot Tests

### Basic Example
```rust
use common::snapshot_utils::assert_transpiler_snapshot;

#[test]
fn test_my_feature() {
    let input = "val x = 42";
    let output = transpile(input, &ctx).unwrap();
    assert_transpiler_snapshot("my_feature", input, &output);
}
```

### Error Snapshots
```rust
use common::snapshot_utils::assert_error_snapshot;

#[test]
fn test_error_case() {
    let error = parse("invalid code").unwrap_err();
    assert_error_snapshot("my_error_case", &error);
}
```

## Snapshot Files
- Stored in `tests/snapshots/`
- Named: `{test_module}__{snapshot_name}.snap`
- Committed to git
- Human-readable format

## CI Integration
The CI pipeline will fail if snapshots are out of date. Before pushing:
1. Run all tests
2. Review any snapshot changes
3. Commit accepted snapshots

## Best Practices
1. Use descriptive snapshot names
2. Keep snapshots focused and small
3. Review snapshot changes carefully
4. Don't snapshot timestamps or paths
5. Use redactions for non-deterministic values

## Troubleshooting

### Snapshots not found
Ensure you're running tests from the project root.

### Pending snapshots
Run `cargo insta review` to handle pending snapshots.

### Different output locally vs CI
Check for platform-specific differences (paths, line endings).
