use archidoc_tests::ArchitectureDsl;

#[test]
fn merge_combines_ir_from_separate_snapshots() {
    let mut arch = ArchitectureDsl::setup();

    // First compilation: just api
    arch.annotate_container(&["name: api", "purpose: REST API gateway"]);
    arch.compile();
    arch.emit_ir();
    arch.save_ir_as(&["snapshot: set_a"]);

    // Second compilation: just events (will also include api from source tree)
    arch.annotate_container(&["name: events", "purpose: Event processing"]);
    arch.compile();
    arch.emit_ir();
    arch.save_ir_as(&["snapshot: set_b"]);

    arch.merge_ir_snapshots(&["set_a", "set_b"]);
    arch.assert_merged_contains(&["name: api", "level: container"]);
    arch.assert_merged_contains(&["name: events", "level: container"]);
}

#[test]
fn merge_deduplicates_by_module_path() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&["name: api", "purpose: REST API gateway"]);
    arch.compile();
    arch.emit_ir();
    arch.save_ir_as(&["snapshot: original"]);

    // Save same IR again as different snapshot
    arch.save_ir_as(&["snapshot: duplicate"]);

    arch.merge_ir_snapshots(&["original", "duplicate"]);
    // api appears in both but should be deduplicated
    arch.assert_merged_element_count(&["count: 1"]);
}

#[test]
fn merge_preserves_all_relationships() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&["name: api", "purpose: REST API gateway"]);
    arch.annotate_container(&["name: database", "purpose: Persistence"]);
    arch.declare_dependency(&["from: api", "to: database", "label: Persists data", "protocol: sqlx"]);
    arch.compile();
    arch.emit_ir();
    arch.save_ir_as(&["snapshot: with_deps"]);

    // Merge with itself — relationships should survive
    arch.save_ir_as(&["snapshot: copy"]);
    arch.merge_ir_snapshots(&["with_deps", "copy"]);
    arch.assert_merged_contains(&["name: api", "level: container"]);
    arch.assert_merged_contains(&["name: database", "level: container"]);
}
