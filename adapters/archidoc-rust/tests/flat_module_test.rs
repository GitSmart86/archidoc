//! Integration test for flat module support and @c4 syntax
//!
//! Verifies that the Rust adapter can parse flat modules (src/foo.rs)
//! using the @c4 annotation syntax.

use archidoc_rust::walker;
use std::fs;
use tempfile::TempDir;

#[test]
fn flat_module_with_at_c4_syntax() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let root = temp.path();

    // Create a flat module using @c4 syntax
    fs::write(
        root.join("router.rs"),
        "//! @c4 container\n//!\n//! # Router\n//!\n//! HTTP request router\n",
    )
    .expect("failed to write router.rs");

    let docs = walker::extract_all_docs(root);

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].module_path, "router");
    assert_eq!(docs[0].c4_level, archidoc_types::C4Level::Container);
}

#[test]
fn nested_flat_module() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let root = temp.path();

    // Create bus directory with a flat module inside
    fs::create_dir_all(root.join("bus")).expect("failed to create bus dir");
    fs::write(
        root.join("bus/events.rs"),
        "//! @c4 component\n//!\n//! # Events\n//!\n//! Event handling for bus\n",
    )
    .expect("failed to write bus/events.rs");

    let docs = walker::extract_all_docs(root);

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].module_path, "bus.events");
    assert_eq!(docs[0].c4_level, archidoc_types::C4Level::Component);
    assert_eq!(docs[0].parent_container, Some("bus".to_string()));
}

#[test]
fn mod_rs_takes_priority_over_flat_module() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let root = temp.path();

    // Create both foo.rs and foo/mod.rs — mod.rs should win
    fs::write(
        root.join("foo.rs"),
        "//! @c4 container\n//!\n//! # Foo Flat\n//!\n//! This should be ignored\n",
    )
    .expect("failed to write foo.rs");

    fs::create_dir_all(root.join("foo")).expect("failed to create foo dir");
    fs::write(
        root.join("foo/mod.rs"),
        "//! @c4 container\n//!\n//! # Foo Mod\n//!\n//! This should be used\n",
    )
    .expect("failed to write foo/mod.rs");

    let docs = walker::extract_all_docs(root);

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].module_path, "foo");
    assert!(docs[0].content.contains("Foo Mod"), "mod.rs content should be used");
    assert!(!docs[0].content.contains("Foo Flat"), "foo.rs should be ignored");
}

#[test]
fn flat_module_without_c4_marker_is_skipped() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let root = temp.path();

    // Create a .rs file without C4 markers
    fs::write(
        root.join("utils.rs"),
        "//! # Utils\n//!\n//! Utility functions\n",
    )
    .expect("failed to write utils.rs");

    let docs = walker::extract_all_docs(root);

    // Should be skipped because it has no C4 marker
    assert_eq!(docs.len(), 0);
}

#[test]
fn mixed_traditional_and_flat_modules() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let root = temp.path();

    // lib.rs at root
    fs::write(
        root.join("lib.rs"),
        "//! @c4 container\n//!\n//! # Crate Root\n//!\n//! Top-level module\n",
    )
    .expect("failed to write lib.rs");

    // Traditional mod.rs structure for bus
    fs::create_dir_all(root.join("bus")).expect("failed to create bus dir");
    fs::write(
        root.join("bus/mod.rs"),
        "//! @c4 container\n//!\n//! # Bus\n//!\n//! Message bus\n",
    )
    .expect("failed to write bus/mod.rs");

    // Flat module for router
    fs::write(
        root.join("router.rs"),
        "//! @c4 component\n//!\n//! # Router\n//!\n//! Request router\n",
    )
    .expect("failed to write router.rs");

    // Nested flat module
    fs::write(
        root.join("bus/calc.rs"),
        "//! @c4 component\n//!\n//! # Calc\n//!\n//! Calculations\n",
    )
    .expect("failed to write bus/calc.rs");

    let mut docs = walker::extract_all_docs(root);
    docs.sort_by(|a, b| a.module_path.cmp(&b.module_path));

    assert_eq!(docs.len(), 4);
    assert_eq!(docs[0].module_path, "_lib");
    assert_eq!(docs[1].module_path, "bus");
    assert_eq!(docs[2].module_path, "bus.calc");
    assert_eq!(docs[3].module_path, "router");
}
