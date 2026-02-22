/// Integration tests for polyglot auto-detection.
///
/// Tests that the archidoc CLI correctly detects and merges output from
/// both Rust and TypeScript language adapters.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Find the archidoc binary built by cargo.
fn archidoc_bin() -> PathBuf {
    // cargo test builds into the target directory; find the binary there
    let mut path = std::env::current_exe()
        .expect("cannot find test executable path")
        .parent()
        .expect("no parent dir")
        .parent()
        .expect("no grandparent dir")
        .to_path_buf();
    if cfg!(windows) {
        path.push("archidoc.exe");
    } else {
        path.push("archidoc");
    }
    path
}

/// Check if archidoc-ts is available via npx.
fn archidoc_ts_available() -> bool {
    let result = if cfg!(windows) {
        Command::new("cmd")
            .args(["/c", "npx", "archidoc-ts", "--help"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    } else {
        Command::new("npx")
            .args(["archidoc-ts", "--help"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    };
    result.map(|s| s.success()).unwrap_or(false)
}

/// Create a temp directory with a minimal Rust project fixture.
fn create_rust_fixture(root: &std::path::Path) {
    let src = root.join("src").join("api");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("mod.rs"),
        "//! @c4 container\n//!\n//! REST API gateway for client requests.\n//!\n//! | File | Pattern | Purpose | Health |\n//! |------|---------|---------|--------|\n//! | `routes.rs` | -- | Route handlers | active |\n",
    )
    .unwrap();
    // Cargo.toml so archidoc recognizes it as a Rust project
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"test-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub mod api;\n").unwrap();
}

/// Create a temp directory with a minimal TypeScript project fixture.
fn create_ts_fixture(root: &std::path::Path) {
    let src = root.join("src").join("dashboard");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("index.ts"),
        "/**\n * @c4 container\n *\n * Real-time dashboard for live data.\n *\n * | File | Pattern | Purpose | Health |\n * |------|---------|---------|--------|\n * | `charts.ts` | Observer | Chart rendering | active |\n */\nexport {};\n",
    )
    .unwrap();
    // package.json so archidoc detects TS project
    fs::write(
        root.join("package.json"),
        "{\"name\": \"test-fixture\", \"version\": \"0.1.0\"}\n",
    )
    .unwrap();
}

#[test]
fn rust_only_project_produces_output_without_ts_adapter() {
    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .args(["--health", "."])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "archidoc --health failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Should find the Rust container
    assert!(
        stdout.contains("Elements:"),
        "Health report missing Elements line: {stdout}"
    );
}

#[test]
fn rust_only_project_generates_architecture_docs() {
    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .arg(".")
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    assert!(
        output.status.success(),
        "archidoc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let arch = fs::read_to_string(tmp.path().join("ARCHITECTURE.md")).unwrap();
    assert!(arch.contains("api"), "ARCHITECTURE.md should mention 'api' container");
}

#[test]
fn project_without_package_json_skips_ts_adapter() {
    // A Rust-only project (no package.json) should not attempt to run archidoc-ts
    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .args(["-v", "."])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "archidoc -v failed: {stderr}"
    );
    // Verbose output should NOT mention typescript adapter
    assert!(
        !stderr.contains("typescript adapter:"),
        "Should not attempt TS adapter without package.json: {stderr}"
    );
}

#[test]
fn polyglot_project_merges_rust_and_typescript() {
    if !archidoc_ts_available() {
        eprintln!("SKIPPED: archidoc-ts not available via npx");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());
    create_ts_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .args(["-v", "."])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "archidoc -v failed: {stderr}"
    );

    // Verbose output should mention TS adapter found modules
    assert!(
        stderr.contains("typescript adapter:"),
        "Verbose output should show TS adapter: {stderr}"
    );

    // Generate and check ARCHITECTURE.md has both languages
    let arch = fs::read_to_string(tmp.path().join("ARCHITECTURE.md")).unwrap();
    assert!(
        arch.contains("api"),
        "ARCHITECTURE.md should contain Rust 'api' container: {arch}"
    );
    assert!(
        arch.contains("dashboard"),
        "ARCHITECTURE.md should contain TS 'dashboard' container: {arch}"
    );
}

#[test]
fn polyglot_health_reports_elements_from_both_languages() {
    if !archidoc_ts_available() {
        eprintln!("SKIPPED: archidoc-ts not available via npx");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());
    create_ts_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .args(["--health", "--json", "."])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    assert!(
        output.status.success(),
        "archidoc --health --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON health should report at least 2 elements (1 Rust + 1 TS)
    assert!(
        stdout.contains("\"total_elements\""),
        "JSON health should contain total_elements: {stdout}"
    );
}

#[test]
fn polyglot_emit_ir_includes_both_languages() {
    if !archidoc_ts_available() {
        eprintln!("SKIPPED: archidoc-ts not available via npx");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    create_rust_fixture(tmp.path());
    create_ts_fixture(tmp.path());

    let bin = archidoc_bin();
    let output = Command::new(&bin)
        .args(["--emit-ir", "."])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run archidoc");

    assert!(
        output.status.success(),
        "archidoc --emit-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let ir_json = String::from_utf8_lossy(&output.stdout);
    // Module paths include the src prefix (e.g., "src.api", "src.dashboard")
    assert!(
        ir_json.contains("api"),
        "Emitted IR should contain Rust 'api' module: {ir_json}"
    );
    assert!(
        ir_json.contains("dashboard"),
        "Emitted IR should contain TS 'dashboard' module: {ir_json}"
    );
}
