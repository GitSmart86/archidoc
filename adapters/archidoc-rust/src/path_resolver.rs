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
        return "_lib".to_string();
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
