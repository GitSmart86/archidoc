use std::path::Path;

/// Convert a file path to dot-notation module path.
///
/// Examples:
/// - `root/bus/mod.rs` relative to `root/` -> `bus`
/// - `root/bus/calc/indicators/mod.rs` relative to `root/` -> `bus.calc.indicators`
/// - `root/lib.rs` -> `_lib`
/// - `root/foo.rs` -> `foo` (flat module at root)
/// - `root/foo/bar.rs` -> `foo.bar` (flat module nested)
pub fn path_to_module_name(path: &Path, root: &Path, filename: &str) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let parent = relative.parent().unwrap_or(Path::new(""));

    if filename == "lib.rs" {
        // A crate-root `lib.rs`. In a single-crate scan it sits at the scan root
        // and maps to `_lib`. In a multi-crate workspace scan every crate has its
        // own `<crate>/src/lib.rs`; derive a unique module path from the crate
        // directory (stripping a trailing `src`) so the crates don't all collide
        // on `_lib` and overwrite each other in the doc map.
        let crate_dir = match parent.file_name().and_then(|n| n.to_str()) {
            Some("src") => parent.parent().unwrap_or(Path::new("")),
            _ => parent,
        };
        let parts: Vec<&str> = crate_dir
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        return if parts.is_empty() {
            "_lib".to_string()
        } else {
            parts.join(".")
        };
    }

    // Convert path components to dot notation
    let parts: Vec<&str> = parent
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if filename == "mod.rs" {
        // Traditional module structure: src/foo/mod.rs -> foo
        parts.join(".")
    } else {
        // Flat module structure: src/foo.rs -> foo, src/foo/bar.rs -> foo.bar
        let module_name = filename.strip_suffix(".rs").unwrap_or(filename);
        if parts.is_empty() {
            // Standalone file at root (e.g., router.rs)
            module_name.to_string()
        } else {
            // Nested flat module: src/foo/bar.rs -> foo.bar
            let mut full_parts = parts;
            full_parts.push(module_name);
            full_parts.join(".")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mod_rs_at_root() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/bus/mod.rs");
        assert_eq!(path_to_module_name(&path, &root, "mod.rs"), "bus");
    }

    #[test]
    fn test_nested_mod_rs() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/bus/calc/indicators/mod.rs");
        assert_eq!(
            path_to_module_name(&path, &root, "mod.rs"),
            "bus.calc.indicators"
        );
    }

    #[test]
    fn test_lib_rs() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/lib.rs");
        assert_eq!(path_to_module_name(&path, &root, "lib.rs"), "_lib");
    }

    #[test]
    fn test_workspace_crate_lib_rs() {
        // Each crate's `<crate>/src/lib.rs` gets a unique, crate-scoped module
        // name instead of all colliding on `_lib`.
        let root = PathBuf::from("/ws");
        assert_eq!(
            path_to_module_name(&PathBuf::from("/ws/holon-core/src/lib.rs"), &root, "lib.rs"),
            "holon-core"
        );
        assert_eq!(
            path_to_module_name(&PathBuf::from("/ws/crates/holon-turso/src/lib.rs"), &root, "lib.rs"),
            "crates.holon-turso"
        );
    }

    #[test]
    fn test_flat_module_at_root() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/router.rs");
        assert_eq!(path_to_module_name(&path, &root, "router.rs"), "router");
    }

    #[test]
    fn test_flat_module_nested() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/bus/events.rs");
        assert_eq!(
            path_to_module_name(&path, &root, "events.rs"),
            "bus.events"
        );
    }

    #[test]
    fn test_flat_module_deeply_nested() {
        let root = PathBuf::from("/src");
        let path = PathBuf::from("/src/bus/calc/indicators.rs");
        assert_eq!(
            path_to_module_name(&path, &root, "indicators.rs"),
            "bus.calc.indicators"
        );
    }
}
