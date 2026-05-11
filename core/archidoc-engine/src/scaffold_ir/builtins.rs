use archidoc_types::scaffold_ir::ScaffoldIR;

pub const BUILTIN_NAMES: &[&str] = &["custom-scaffolds", "custom-inits", "custom-trees"];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}

/// Load a built-in ScaffoldIR by name.
pub fn load_builtin(name: &str) -> Option<ScaffoldIR> {
    let json = match name {
        "custom-scaffolds" => CUSTOM_SCAFFOLDS,
        "custom-inits" => custom_inits_json(),
        "custom-trees" => CUSTOM_TREES,
        _ => return None,
    };
    serde_json::from_str(json).ok()
}

// ---------------------------------------------------------------------------
// custom-scaffolds
// ---------------------------------------------------------------------------

const CUSTOM_SCAFFOLDS: &str = r#"{
  "version": "1.0",
  "template": {
    "name": "custom-scaffolds",
    "description": "Create .archidoc/scaffolds/ for storing ScaffoldIR template files",
    "variables": [],
    "post_hooks": []
  },
  "nodes": [
    { "type": "dir", "path": ".archidoc" },
    { "type": "dir", "path": ".archidoc/scaffolds" }
  ]
}"#;

// ---------------------------------------------------------------------------
// custom-trees
// ---------------------------------------------------------------------------

const CUSTOM_TREES: &str = r#"{
  "version": "1.0",
  "template": {
    "name": "custom-trees",
    "description": "Add config.tree.json for customizing directory tree generation (exclusions, icons, thresholds)",
    "variables": [],
    "post_hooks": []
  },
  "nodes": [
    { "type": "dir",  "path": ".archidoc" },
    { "type": "file", "path": ".archidoc/config.tree.json", "content": "{\n  \"exclude_dirs\": [],\n  \"exclude_files\": [],\n  \"include_extensions\": [],\n  \"inline_threshold\": 6,\n  \"icons\": {\n    \"directory\": \"📁\",\n    \"file\": \"📄\",\n    \"by_ext\": {\n      \".md\": \"📖\",\n      \".rs\": \"🔷\",\n      \".ts\": \"🟦\",\n      \".js\": \"🟨\",\n      \".json\": \"⚙️\",\n      \".toml\": \"⚙️\",\n      \".yaml\": \"🗂️\",\n      \".yml\": \"🗂️\",\n      \".py\": \"🐍\",\n      \".sh\": \"📜\",\n      \".ps1\": \"📜\"\n    }\n  }\n}\n" }
  ]
}"#;

// ---------------------------------------------------------------------------
// custom-inits (references DEFAULT_ constants from crate::custom)
// ---------------------------------------------------------------------------

fn custom_inits_json() -> &'static str {
    // Content strings are substituted at call time from the canonical custom module.
    // We build this JSON once and leak it as a static str.
    use std::sync::OnceLock;
    static BUILT: OnceLock<String> = OnceLock::new();
    BUILT.get_or_init(|| {
        // Escape the content strings for JSON embedding.
        let mod_rs   = serde_json::to_string(crate::custom::DEFAULT_SUGGEST_RUST).unwrap();
        let index_ts  = serde_json::to_string(crate::custom::DEFAULT_SUGGEST_TS).unwrap();
        let index_md  = serde_json::to_string(crate::custom::DEFAULT_SUGGEST_MD).unwrap();
        let arch_table = serde_json::to_string(crate::custom::DEFAULT_ARCHITECTURE_TABLE).unwrap();

        format!(
            r#"{{
  "version": "1.0",
  "template": {{
    "name": "custom-inits",
    "description": "Add init-overrides/ with default override files for customizing init handler output",
    "variables": [],
    "post_hooks": []
  }},
  "nodes": [
    {{ "type": "dir",  "path": ".archidoc" }},
    {{ "type": "dir",  "path": ".archidoc/init-overrides" }},
    {{ "type": "file", "path": ".archidoc/init-overrides/mod.rs",                "content": {mod_rs} }},
    {{ "type": "file", "path": ".archidoc/init-overrides/index.ts",              "content": {index_ts} }},
    {{ "type": "file", "path": ".archidoc/init-overrides/_index.md",             "content": {index_md} }},
    {{ "type": "file", "path": ".archidoc/init-overrides/architecture-table.md", "content": {arch_table} }}
  ]
}}"#
        )
    })
}
