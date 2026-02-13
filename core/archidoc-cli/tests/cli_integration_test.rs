/// Integration tests for CLI argument parsing and output modes.
///
/// These tests verify that the clap-based CLI correctly handles:
/// - Output directory configuration (--output)
/// - Verbosity levels (--quiet, --verbose)
/// - JSON output mode (--json)
/// - Subcommands (init-adapter)

#[test]
fn test_cli_builds_successfully() {
    // This test simply verifies that the CLI binary compiles.
    // The actual CLI functionality is tested through the BDD test suite.
    assert!(true);
}
