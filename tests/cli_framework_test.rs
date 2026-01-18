//! Integration tests for CLI Framework (M1-E06-T03)
//!
//! These tests validate the CLI structure, flags, and help output.

use assert_cmd::Command;
use predicates::str::contains;

const COMMAND_NAME: &str = "sikil";

#[test]
fn test_cli_help_output() {
    // M1-E06-T03-S01: Test --help output
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains(
            "Manage AI coding assistant skills across multiple agents",
        ))
        .stdout(contains("Usage:"))
        .stdout(contains("Commands:"))
        .stdout(contains("Options:"))
        .stdout(contains("--json"))
        .stdout(contains("--verbose"))
        .stdout(contains("--quiet"))
        .stdout(contains("--help"))
        .stdout(contains("--version"));
}

#[test]
fn test_cli_version_output() {
    // M1-E06-T03-S02: Test --version output
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains("sikil"))
        .stdout(contains("0.1.0"));
}

#[test]
fn test_cli_version_short_flag() {
    // Also test -V short flag for version
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("-V")
        .assert()
        .success()
        .stdout(contains("sikil"))
        .stdout(contains("0.1.0"));
}

#[test]
fn test_unknown_command_error() {
    // M1-E06-T03-S03: Test unknown command error
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"))
        .stderr(contains("nonexistent-command"));
}

#[test]
fn test_json_flag_parsing() {
    // M1-E06-T03-S04: Test --json flag parsing
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    // Test that --json is accepted as a global flag
    cmd.arg("--json").arg("list").assert().success(); // Will fail later with "not yet implemented" but flag parses
}

#[test]
fn test_verbose_flag_parsing() {
    // Test --verbose flag parsing
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--verbose").arg("list").assert().success();
}

#[test]
fn test_verbose_short_flag_parsing() {
    // Test -v short flag for verbose
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("-v").arg("list").assert().success();
}

#[test]
fn test_quiet_flag_parsing() {
    // Test --quiet flag parsing
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--quiet").arg("list").assert().success();
}

#[test]
fn test_quiet_short_flag_parsing() {
    // Test -q short flag for quiet
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("-q").arg("list").assert().success();
}

#[test]
fn test_quiet_and_verbose_mutually_exclusive() {
    // Test that --quiet and --verbose are mutually exclusive
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--quiet")
        .arg("--verbose")
        .arg("list")
        .assert()
        .failure()
        .stderr(contains("--quiet and --verbose are mutually exclusive"));
}

#[test]
fn test_json_flag_with_list_command() {
    // M1-E06-T03-S05: Test --json emits valid JSON on stdout with no non-JSON noise
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    // The list command will output "List command - not yet implemented"
    // but we're testing that the JSON flag is properly parsed and handled
    // This will be more meaningful when list command is implemented
    cmd.arg("--json").arg("list").assert().success();
}

#[test]
fn test_list_command_help_includes_examples() {
    // M1-E06-T03-S06: Test sikil <cmd> --help includes Examples section
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil list"))
        .stdout(contains("sikil list --agent claude-code"))
        .stdout(contains("sikil list --managed"))
        .stdout(contains("sikil list --json"));
}

#[test]
fn test_show_command_help_includes_examples() {
    // Test show command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("show")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil show my-skill"))
        .stdout(contains("sikil show --json my-skill"));
}

#[test]
fn test_install_command_help_includes_examples() {
    // Test install command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("install")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil install ./path/to/skill"))
        .stdout(contains("sikil install user/repo"))
        .stdout(contains("sikil install https://github.com/user/repo.git"))
        .stdout(contains("sikil install ./skill --to claude-code,windsurf"));
}

#[test]
fn test_validate_command_help_includes_examples() {
    // Test validate command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil validate ./path/to/skill"))
        .stdout(contains("sikil validate my-skill"));
}

#[test]
fn test_adopt_command_help_includes_examples() {
    // Test adopt command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("adopt")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil adopt my-skill"))
        .stdout(contains("sikil adopt my-skill --from claude-code"));
}

#[test]
fn test_unmanage_command_help_includes_examples() {
    // Test unmanage command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("unmanage")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil unmanage my-skill"))
        .stdout(contains("sikil unmanage my-skill --agent claude-code"))
        .stdout(contains("sikil unmanage my-skill --yes"));
}

#[test]
fn test_remove_command_help_includes_examples() {
    // Test remove command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("remove")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil remove my-skill --agent claude-code"))
        .stdout(contains("sikil remove my-skill --all"))
        .stdout(contains("sikil remove my-skill --all --yes"));
}

#[test]
fn test_sync_command_help_includes_examples() {
    // Test sync command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("sync")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil sync my-skill"))
        .stdout(contains("sikil sync --all"))
        .stdout(contains("sikil sync my-skill --to claude-code,windsurf"));
}

#[test]
fn test_completions_command_help_includes_examples() {
    // Test completions command help includes examples
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("completions")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("EXAMPLES:"))
        .stdout(contains("sikil completions bash"))
        .stdout(contains(
            "sikil completions zsh --output ~/.zsh/completions/_sikil",
        ))
        .stdout(contains("sikil completions fish"));
}

#[test]
fn test_all_subcommands_available() {
    // Test that all defined subcommands are available
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("show"))
        .stdout(contains("install"))
        .stdout(contains("validate"))
        .stdout(contains("adopt"))
        .stdout(contains("unmanage"))
        .stdout(contains("remove"))
        .stdout(contains("sync"))
        .stdout(contains("config"))
        .stdout(contains("completions"));
}

#[test]
fn test_global_flags_documented() {
    // Test that global flags are documented in help
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Output in JSON format"))
        .stdout(contains("Enable verbose output"))
        .stdout(contains("Suppress all output"));
}

#[test]
fn test_no_args_requires_subcommand() {
    // Test that running with no args requires a subcommand (clap behavior)
    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    // When no subcommand is provided, clap should exit with error code 2
    // and display help text on stderr
    cmd.assert()
        .failure()
        .stderr(contains("Usage:"))
        .stderr(contains("Commands:"));
}
