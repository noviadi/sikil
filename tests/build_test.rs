//! Build-related tests for Sikil
//!
//! These tests verify build constraints and binary properties as specified
//! in the build-and-platform.md spec.

/// Maximum allowed binary size in bytes (10MB as per spec)
const MAX_BINARY_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Gets the path to the sikil release binary.
///
/// This function returns the path to the release build binary. It assumes
/// the binary has already been built via `cargo build --release`.
fn release_binary_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/sikil")
}

/// Calculates the binary size in bytes.
///
/// # Panics
/// Panics if the release binary does not exist.
fn get_binary_size() -> u64 {
    let binary_path = release_binary_path();
    if !binary_path.exists() {
        panic!(
            "Release binary not found at {}. Run `cargo build --release` first.",
            binary_path.display()
        );
    }
    std::fs::metadata(binary_path)
        .expect("Failed to read binary metadata")
        .len()
}

/// Converts bytes to a human-readable size string (e.g., "3.2 MB").
fn format_size(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        const KB: u64 = 1024;
        format!("{:.1} KB", bytes as f64 / KB as f64)
    }
}

#[test]
#[ignore] // Requires release build, run with: cargo test --test build_test -- --ignored
fn test_release_binary_size_under_10mb() {
    let size_bytes = get_binary_size();
    let size_formatted = format_size(size_bytes);

    assert!(
        size_bytes <= MAX_BINARY_SIZE_BYTES,
        "Release binary size {} ({} bytes) exceeds the maximum allowed size of 10MB ({} bytes). \
        Consider reviewing dependencies or build configuration.",
        size_formatted,
        size_bytes,
        MAX_BINARY_SIZE_BYTES
    );

    println!("Binary size: {} ({} bytes)", size_formatted, size_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(10 * 1024 * 1024), "10.0 MB");
        assert_eq!(format_size(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(format_size(3_296_392), "3.1 MB");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(512), "0.5 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_max_binary_size_constant() {
        // Verify the constant is exactly 10MB
        assert_eq!(MAX_BINARY_SIZE_BYTES, 10 * 1024 * 1024);
    }
}
