use std::fmt;
use std::collections::HashMap;
use archidoc_types::ModuleDoc;

/// Error returned when merge encounters conflicting module definitions.
#[derive(Debug)]
pub struct MergeError {
    pub module_path: String,
    pub message: String,
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "merge conflict at '{}': {}", self.module_path, self.message)
    }
}

/// Merge multiple IR sets into a single unified ModuleDoc list.
///
/// Rules:
/// - Modules with unique paths are included as-is
/// - Duplicate module_paths with the SAME c4_level: last writer wins (later source overrides earlier)
/// - Duplicate module_paths with DIFFERENT c4_levels: returns MergeError
/// - Output is sorted by module_path
pub fn merge_ir(sources: Vec<Vec<ModuleDoc>>) -> Result<Vec<ModuleDoc>, MergeError> {
    let mut merged: HashMap<String, ModuleDoc> = HashMap::new();

    for source_set in sources {
        for doc in source_set {
            let module_path = doc.module_path.clone();

            if let Some(existing) = merged.get(&module_path) {
                if existing.c4_level != doc.c4_level {
                    return Err(MergeError {
                        module_path: module_path.clone(),
                        message: format!(
                            "conflicting C4 levels: existing '{}' vs new '{}'",
                            existing.c4_level,
                            doc.c4_level
                        ),
                    });
                }

                eprintln!(
                    "warning: duplicate module '{}' at C4 level '{}', overwriting with later source",
                    module_path,
                    doc.c4_level
                );
            }

            merged.insert(module_path, doc);
        }
    }

    let mut result: Vec<ModuleDoc> = merged.into_values().collect();
    result.sort_by(|a, b| a.module_path.cmp(&b.module_path));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::{C4Level, PatternStatus, Relationship};

    fn make_doc(path: &str, level: C4Level) -> ModuleDoc {
        ModuleDoc {
            module_path: path.to_string(),
            content: String::new(),
            source_file: format!("src/{}/mod.rs", path),
            c4_level: level,
            pattern: "--".to_string(),
            pattern_status: PatternStatus::Planned,
            description: format!("Module {}", path),
            parent_container: None,
            relationships: vec![],
            files: vec![],
        }
    }

    #[test]
    fn merge_combines_disjoint_sets() {
        let set1 = vec![
            make_doc("api", C4Level::Container),
            make_doc("core", C4Level::Container),
        ];
        let set2 = vec![
            make_doc("database", C4Level::Component),
            make_doc("ui", C4Level::Component),
        ];

        let result = merge_ir(vec![set1, set2]).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].module_path, "api");
        assert_eq!(result[1].module_path, "core");
        assert_eq!(result[2].module_path, "database");
        assert_eq!(result[3].module_path, "ui");
    }

    #[test]
    fn merge_deduplicates_same_level() {
        let set1 = vec![
            make_doc("api", C4Level::Container),
        ];
        let mut set2 = vec![
            make_doc("api", C4Level::Container),
        ];
        set2[0].description = "Updated API module".to_string();

        let result = merge_ir(vec![set1, set2]).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].module_path, "api");
        assert_eq!(result[0].description, "Updated API module");
    }

    #[test]
    fn merge_rejects_conflicting_c4_levels() {
        let set1 = vec![
            make_doc("api", C4Level::Container),
        ];
        let set2 = vec![
            make_doc("api", C4Level::Component),
        ];

        let result = merge_ir(vec![set1, set2]);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.module_path, "api");
        assert!(err.message.contains("conflicting C4 levels"));
        assert!(err.message.contains("container"));
        assert!(err.message.contains("component"));
    }

    #[test]
    fn merge_sorts_by_module_path() {
        let set1 = vec![
            make_doc("zebra", C4Level::Container),
            make_doc("alpha", C4Level::Container),
        ];
        let set2 = vec![
            make_doc("middle", C4Level::Component),
        ];

        let result = merge_ir(vec![set1, set2]).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].module_path, "alpha");
        assert_eq!(result[1].module_path, "middle");
        assert_eq!(result[2].module_path, "zebra");
    }

    #[test]
    fn merge_empty_inputs_returns_empty() {
        let result1 = merge_ir(vec![]).unwrap();
        assert_eq!(result1.len(), 0);

        let result2 = merge_ir(vec![vec![], vec![]]).unwrap();
        assert_eq!(result2.len(), 0);
    }

    #[test]
    fn merge_preserves_relationships() {
        let mut doc1 = make_doc("api", C4Level::Container);
        doc1.relationships = vec![
            Relationship {
                target: "database".to_string(),
                label: "Persists data".to_string(),
                protocol: "sqlx".to_string(),
            },
        ];

        let mut doc2 = make_doc("database", C4Level::Component);
        doc2.relationships = vec![
            Relationship {
                target: "storage".to_string(),
                label: "Writes files".to_string(),
                protocol: "fs".to_string(),
            },
        ];

        let result = merge_ir(vec![vec![doc1], vec![doc2]]).unwrap();

        assert_eq!(result.len(), 2);

        let api_doc = result.iter().find(|d| d.module_path == "api").unwrap();
        assert_eq!(api_doc.relationships.len(), 1);
        assert_eq!(api_doc.relationships[0].target, "database");

        let db_doc = result.iter().find(|d| d.module_path == "database").unwrap();
        assert_eq!(db_doc.relationships.len(), 1);
        assert_eq!(db_doc.relationships[0].target, "storage");
    }
}
