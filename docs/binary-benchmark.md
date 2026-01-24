# Binary Size Benchmark

Tracking binary size changes for the `sikil` CLI.

## Baseline

| Date | Version | Binary Size | Notes |
|------|---------|-------------|-------|
| 2026-01-23 | 0.1.0 | 5.0 MB | SQLite cache (rusqlite bundled) |
| 2026-01-23 | 0.1.0 | 3.2 MB | JSON file cache (after replacing SQLite) |

## Build Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## Planned Changes

| Change | Expected Savings | Status | Actual Savings |
|--------|------------------|--------|---|
| Replace SQLite with JSON file cache | ~1-2 MB | ✅ Completed | 1.8 MB (36% reduction) |
