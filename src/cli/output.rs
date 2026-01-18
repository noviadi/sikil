//! Output formatting and utilities for CLI user interface
//!
//! This module provides consistent output formatting across all commands,
//! including colored output, JSON output, and progress indicators.

use anstream::println;
use anstyle::{AnsiColor, Color, Style};
use serde::Serialize;
use std::io::{self, Write};

/// Output manager for consistent CLI formatting
#[derive(Debug, Clone)]
pub struct Output {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Whether to suppress colors (NO_COLOR env var or explicit setting)
    pub no_color: bool,
}

impl Output {
    /// Create a new Output manager
    pub fn new(json_mode: bool) -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();

        Self {
            json_mode,
            no_color,
        }
    }

    /// Print a success message (green)
    pub fn print_success(&self, msg: &str) {
        if self.json_mode {
            // Messages go to stderr in JSON mode
            eprintln!("{}", msg);
        } else if !self.no_color {
            let style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
            println!("{} {}", style.render(), msg);
        } else {
            println!("✓ {}", msg);
        }
    }

    /// Print a warning message (yellow)
    pub fn print_warning(&self, msg: &str) {
        if self.json_mode {
            eprintln!("{}", msg);
        } else if !self.no_color {
            let style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
            println!("{} {}", style.render(), msg);
        } else {
            println!("⚠ {}", msg);
        }
    }

    /// Print an error message (red)
    pub fn print_error(&self, msg: &str) {
        if self.json_mode {
            eprintln!("{}", msg);
        } else if !self.no_color {
            let style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
            println!("{} {}", style.render(), msg);
        } else {
            println!("✗ {}", msg);
        }
    }

    /// Print JSON to stdout
    pub fn print_json<T: Serialize>(&self, value: &T) -> Result<(), serde_json::Error> {
        let json = serde_json::to_string_pretty(value)?;
        println!("{}", json);
        Ok(())
    }

    /// Print an info message (no color or icon, for informational output)
    pub fn print_info(&self, msg: &str) {
        if self.json_mode {
            eprintln!("{}", msg);
        } else {
            println!("{}", msg);
        }
    }

    /// Get a writer for user messages (stderr in JSON mode, stdout otherwise)
    pub fn message_writer(&self) -> MessageWriter {
        MessageWriter {
            json_mode: self.json_mode,
        }
    }
}

/// Writer that outputs to the correct stream based on JSON mode
pub struct MessageWriter {
    json_mode: bool,
}

impl MessageWriter {
    /// Write a message
    pub fn write(&self, msg: &str) {
        if self.json_mode {
            eprint!("{}", msg);
        } else {
            print!("{}", msg);
        }
    }

    /// Write a message with newline
    pub fn writeln(&self, msg: &str) {
        if self.json_mode {
            eprintln!("{}", msg);
        } else {
            println!("{}", msg);
        }
    }

    /// Flush the output
    pub fn flush(&self) {
        if self.json_mode {
            let _ = io::stderr().flush();
        } else {
            let _ = io::stdout().flush();
        }
    }
}

/// Progress helper for long-running operations
///
/// Automatically disabled when:
/// - JSON mode is enabled
/// - Output is not a TTY
/// - Output is redirected
pub struct Progress {
    inner: Option<indicatif::ProgressBar>,
}

impl Progress {
    /// Create a new progress indicator
    pub fn new(json_mode: bool, total: Option<u64>) -> Self {
        use indicatif::{ProgressBar, ProgressStyle};

        let inner = if json_mode || !atty::is(atty::Stream::Stdout) {
            None
        } else {
            let bar = match total {
                Some(count) => ProgressBar::new(count),
                None => ProgressBar::new_spinner(),
            };

            bar.set_style(
                ProgressStyle::with_template("{spinner:.green} {wide_msg}")
                    .expect("invalid progress template")
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );

            Some(bar)
        };

        Self { inner }
    }

    /// Set the current message
    pub fn set_message(&self, msg: &str) {
        if let Some(bar) = &self.inner {
            bar.set_message(msg.to_string());
        }
    }

    /// Increment the progress
    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.inner {
            bar.inc(delta);
        }
    }

    /// Finish the progress with a success message
    pub fn finish_with_message(&self, msg: &str) {
        if let Some(bar) = &self.inner {
            bar.finish_with_message(msg.to_string());
        }
    }

    /// Abandon the progress with a message
    pub fn abandon_with_message(&self, msg: &str) {
        if let Some(bar) = &self.inner {
            bar.abandon_with_message(msg.to_string());
        }
    }

    /// Clear the progress indicator (no-op in 0.17, just finish)
    pub fn clear(&self) {
        if let Some(bar) = &self.inner {
            bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_output_non_json() {
        let output = Output::new(false);
        // Just verify it doesn't panic
        output.print_success("test success");
        output.print_warning("test warning");
        output.print_error("test error");
        output.print_info("test info");
    }

    #[test]
    fn test_output_json() {
        let output = Output::new(true);
        output.print_success("test success");
        output.print_warning("test warning");
        output.print_error("test error");
    }

    #[test]
    fn test_print_json() {
        let output = Output::new(true);
        let data = json!({ "key": "value" });
        assert!(output.print_json(&data).is_ok());
    }

    #[test]
    fn test_print_json_string() {
        let output = Output::new(true);
        let result: Result<(), serde_json::Error> = output.print_json(&"test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_progress_non_json() {
        let progress = Progress::new(false, Some(10));
        progress.set_message("test");
        progress.inc(1);
        progress.clear();
    }

    #[test]
    fn test_progress_json_mode() {
        let progress = Progress::new(true, Some(10));
        progress.set_message("test");
        progress.inc(1);
        // Should not panic even though progress is disabled
        progress.finish_with_message("done");
    }
}
