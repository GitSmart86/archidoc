//! Phase D: Negative IR Validation (D9)
//!
//! These tests verify that the IR validator rejects malformed JSON —
//! missing required fields, invalid enum values, wrong types, and
//! structurally invalid input.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// Structurally invalid JSON
// =========================================================================

/// Completely invalid JSON is rejected.
#[test]
fn rejects_invalid_json() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects("this is not json at all");
}

/// An empty string is rejected.
#[test]
fn rejects_empty_string() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects("");
}

/// A JSON object (not an array) is rejected — IR must be ModuleDoc[].
#[test]
fn rejects_json_object_instead_of_array() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"{"module_path": "bus"}"#);
}

/// A JSON string is rejected — IR must be ModuleDoc[].
#[test]
fn rejects_json_string_instead_of_array() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#""just a string""#);
}

// =========================================================================
// Missing required fields
// =========================================================================

/// A ModuleDoc missing the required `module_path` field is rejected.
#[test]
fn rejects_module_missing_module_path() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": []
    }]"#);
}

/// A ModuleDoc missing the required `c4_level` field is rejected.
#[test]
fn rejects_module_missing_c4_level() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": []
    }]"#);
}

/// A ModuleDoc missing the required `relationships` field is rejected.
#[test]
fn rejects_module_missing_relationships() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "files": []
    }]"#);
}

/// A ModuleDoc missing the required `files` field is rejected.
#[test]
fn rejects_module_missing_files() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": []
    }]"#);
}

// =========================================================================
// Invalid enum values
// =========================================================================

/// An invalid c4_level value like "system" is rejected.
#[test]
fn rejects_invalid_c4_level() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "system",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": []
    }]"#);
}

/// An invalid pattern_status value like "confirmed" is rejected.
#[test]
fn rejects_invalid_pattern_status() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "confirmed",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": []
    }]"#);
}

/// An invalid health status value like "deprecated" in a file entry is rejected.
#[test]
fn rejects_invalid_health_status_in_file_entry() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": [{
            "name": "lanes.rs",
            "pattern": "Observer",
            "pattern_status": "planned",
            "purpose": "Event routing",
            "health": "deprecated"
        }]
    }]"#);
}

// =========================================================================
// Wrong types
// =========================================================================

/// c4_level as a number instead of string is rejected.
#[test]
fn rejects_c4_level_as_number() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": 1,
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": [],
        "files": []
    }]"#);
}

/// relationships as a string instead of array is rejected.
#[test]
fn rejects_relationships_as_string() {
    let arch = ArchitectureDsl::setup();
    arch.assert_ir_rejects(r#"[{
        "module_path": "bus",
        "content": "some content",
        "source_file": "bus/mod.rs",
        "c4_level": "container",
        "pattern": "Mediator",
        "pattern_status": "planned",
        "description": "Central messaging backbone",
        "parent_container": null,
        "relationships": "not an array",
        "files": []
    }]"#);
}
