//! File Validation — Ghost and Orphan Detection (B2, B3, B4)
//!
//! Given an architecture with file catalogs, when the file tables are
//! validated against the filesystem, ghost entries (cataloged but deleted)
//! and orphan files (present but uncataloged) are detected.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// Clean validation — no ghosts or orphans
// =========================================================================

#[test]
fn clean_architecture_passes_validation() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    // No file catalog entries, so nothing to validate
    arch.compile();

    arch.assert_validation_clean();
}

// =========================================================================
// Ghost detection — cataloged file doesn't exist on disk
// =========================================================================

#[test]
fn ghost_detected_when_cataloged_file_is_missing() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "responsibility: Event routing",
        "maturity: active",
    ]);
    arch.compile();

    // The file table says lanes.rs exists, but we never created it on disk
    // (the fake source tree only creates mod.rs files, not the cataloged files)
    arch.assert_ghost_detected(&["element: bus", "file: lanes.rs"]);
}

// =========================================================================
// Ghost detection — file existed then was removed
// =========================================================================

#[test]
fn ghost_detected_when_file_removed_after_cataloging() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: temp.rs",
        "responsibility: Temporary work",
        "maturity: planned",
    ]);

    // Place the file on disk so it initially exists
    arch.place_file_on_disk(&["element: bus", "file: temp.rs"]);
    arch.compile();

    // Now remove it — simulating a developer deleting a file
    arch.remove_file_from_disk(&["element: bus", "file: temp.rs"]);

    arch.assert_ghost_detected(&["element: bus", "file: temp.rs"]);
}

// =========================================================================
// Orphan detection — file on disk not in catalog
// =========================================================================

#[test]
fn orphan_detected_when_uncataloged_file_exists() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "responsibility: Event routing",
        "maturity: active",
    ]);

    // Place both the cataloged file AND an extra uncataloged file
    arch.place_file_on_disk(&["element: bus", "file: lanes.rs"]);
    arch.place_file_on_disk(&["element: bus", "file: analytics.rs"]);
    arch.compile();

    // lanes.rs is in catalog + on disk = fine
    // analytics.rs is on disk but NOT in catalog = orphan
    arch.assert_orphan_detected(&["element: bus", "file: analytics.rs"]);
}

// =========================================================================
// Mixed ghosts and orphans in same element
// =========================================================================

#[test]
fn detects_both_ghosts_and_orphans_simultaneously() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "responsibility: Event routing",
        "maturity: active",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: deleted.rs",
        "responsibility: Was removed",
        "maturity: planned",
    ]);

    // lanes.rs exists on disk (not a ghost)
    arch.place_file_on_disk(&["element: bus", "file: lanes.rs"]);
    // deleted.rs does NOT exist on disk (ghost)
    // extra.rs exists on disk but NOT in catalog (orphan)
    arch.place_file_on_disk(&["element: bus", "file: extra.rs"]);
    arch.compile();

    arch.assert_ghost_detected(&["element: bus", "file: deleted.rs"]);
    arch.assert_orphan_detected(&["element: bus", "file: extra.rs"]);
}
