# CLI Output Spec

## One-Sentence Description

CLI output controls terminal formatting and progress display.

## Overview

The `Output` struct in `src/cli/output.rs` manages consistent output formatting across all commands, handling colors, stream routing, and progress indicators.

## Output Struct

```rust
pub struct Output {
    pub json_mode: bool,
    pub no_color: bool,
}

impl Output {
    pub fn new(json_mode: bool) -> Self;
}
```

Constructor checks `NO_COLOR` environment variable automatically.

## Stream Routing

| Mode | stdout | stderr |
|------|--------|--------|
| Normal | Messages with colors/icons | Errors |
| JSON (`--json`) | JSON data only | Messages |

## Print Methods

| Method | Color | Icon (NO_COLOR) | Usage |
|--------|-------|-----------------|-------|
| `print_success(msg)` | Green | ✓ | Operation completed |
| `print_warning(msg)` | Yellow | ⚠ | Non-fatal issues |
| `print_error(msg)` | Red | ✗ | Error messages |
| `print_info(msg)` | None | None | Informational output |
| `print_json<T: Serialize>(value)` | - | - | JSON to stdout |

In JSON mode, all print methods write to stderr except `print_json`.

## NO_COLOR Support

- Respects `NO_COLOR` environment variable (any value disables colors)
- Falls back to Unicode icons: ✓ (success), ⚠ (warning), ✗ (error)
- Detected in `Output::new()` constructor

## TTY Detection

Uses `atty::is(atty::Stream::Stdout)` to detect if output is a terminal. Non-TTY disables:
- Progress indicators
- Spinner animations

## MessageWriter

Stream-aware writer returned by `Output::message_writer()`:

```rust
pub struct MessageWriter {
    json_mode: bool,
}

impl MessageWriter {
    pub fn write(&self, msg: &str);     // No newline
    pub fn writeln(&self, msg: &str);   // With newline
    pub fn flush(&self);                // Flush stream
}
```

Routes to stderr in JSON mode, stdout otherwise.

## Progress Struct

Progress indicator for long-running operations:

```rust
impl Progress {
    pub fn new(json_mode: bool, total: Option<u64>) -> Self;
    pub fn set_message(&self, msg: &str);
    pub fn inc(&self, delta: u64);
    pub fn finish_with_message(&self, msg: &str);
    pub fn abandon_with_message(&self, msg: &str);
    pub fn clear(&self);
}
```

### Behavior

- **Disabled when**: JSON mode, non-TTY, output redirected
- **Spinner style**: `⠁⠂⠄⡀⢀⠠⠐⠈` (green)
- **Template**: `{spinner:.green} {wide_msg}`
- `total: Some(n)` creates progress bar, `None` creates spinner

### Methods

| Method | Description |
|--------|-------------|
| `set_message` | Update displayed message |
| `inc` | Increment progress counter |
| `finish_with_message` | Complete with success message |
| `abandon_with_message` | Abort with message |
| `clear` | Remove progress indicator |

## Acceptance Criteria

- `print_success` outputs green text with ✓ icon when colors enabled
- `print_warning` outputs yellow text with ⚠ icon when colors enabled
- `print_error` outputs red text with ✗ icon when colors enabled
- `NO_COLOR` environment variable (any value) disables all colors but keeps Unicode icons
- In JSON mode, `print_json` writes to stdout
- In JSON mode, `print_success`, `print_warning`, `print_error`, and `print_info` write to stderr
- In normal mode, messages write to stdout and errors write to stderr
- Progress indicator is disabled when stdout is not a TTY
- Progress indicator is disabled in JSON mode
- `MessageWriter` routes to stderr in JSON mode, stdout otherwise
- `Progress::new` with `total: Some(n)` creates a progress bar
- `Progress::new` with `total: None` creates a spinner

## Dependencies

- `anstream` / `anstyle` - colored terminal output
- `serde` / `serde_json` - JSON serialization
- `indicatif` - progress bars
- `atty` - TTY detection

## Used By

- All command handlers for user-facing output
- `main.rs` creates `Output` from parsed `Cli` args
