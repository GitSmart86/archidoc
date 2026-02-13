use archidoc_tests::ArchitectureDsl;

#[test]
fn suggest_infers_container_for_top_level_module() {
    let mut arch = ArchitectureDsl::setup();
    arch.annotate_container(&["name: api", "purpose: REST API gateway"]);
    arch.catalog_file(&[
        "element: api",
        "file: routes.rs",
        "responsibility: HTTP handlers",
        "maturity: active",
    ]);
    arch.compile();
    arch.suggest_for(&["element: api"]);
    arch.assert_suggestion_level(&["level: container"]);
}

#[test]
fn suggest_infers_component_for_nested_module() {
    let mut arch = ArchitectureDsl::setup();
    arch.annotate_container(&["name: api", "purpose: REST API gateway"]);
    arch.annotate_component(&["name: api.auth", "purpose: Authentication"]);
    arch.compile();
    arch.suggest_for(&["element: api.auth"]);
    arch.assert_suggestion_level(&["level: component"]);
}

#[test]
fn suggest_lists_source_files_in_directory() {
    let mut arch = ArchitectureDsl::setup();
    arch.annotate_container(&["name: bus", "purpose: Central messaging"]);
    arch.compile();
    // Place source files to be discovered
    arch.place_code_file("bus", "lanes.rs", "// event routing");
    arch.place_code_file("bus", "store.rs", "// cache");
    arch.suggest_for(&["element: bus"]);
    arch.assert_suggestion_lists_file(&["file: lanes.rs"]);
    arch.assert_suggestion_lists_file(&["file: store.rs"]);
}

#[test]
fn suggest_excludes_entry_files() {
    let mut arch = ArchitectureDsl::setup();
    arch.annotate_container(&["name: bus", "purpose: Central messaging"]);
    arch.compile();
    arch.place_code_file("bus", "lanes.rs", "// event routing");
    arch.suggest_for(&["element: bus"]);
    // mod.rs should not appear in suggestion (it's an entry file)
    arch.assert_suggestion_lists_file(&["file: lanes.rs"]);
}
