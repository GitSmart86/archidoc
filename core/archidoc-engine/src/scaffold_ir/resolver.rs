use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use archidoc_types::scaffold_ir::{ScaffoldIR, ScaffoldNode};

const MAX_DEPTH: usize = 10;

#[derive(Debug)]
pub enum ResolveError {
    NotFound { path: PathBuf },
    Cycle { path: PathBuf },
    DepthExceeded,
    InvalidJson { path: PathBuf, detail: String },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::NotFound { path } => {
                write!(f, "ref not found: {}", path.display())
            }
            ResolveError::Cycle { path } => {
                write!(f, "circular $ref detected: {}", path.display())
            }
            ResolveError::DepthExceeded => {
                write!(f, "$ref depth limit ({}) exceeded", MAX_DEPTH)
            }
            ResolveError::InvalidJson { path, detail } => {
                write!(f, "invalid JSON at {}: {}", path.display(), detail)
            }
        }
    }
}

/// Resolve all `$ref` nodes in `ir`, returning a new `ScaffoldIR` with a flat node list.
///
/// `source_dir` is the directory of the file that contains `ir` — used to
/// resolve relative `$ref` paths.
///
/// Resolution is a pre-pass that runs before variable substitution and
/// instantiation. All `$ref` nodes are replaced by the inline node list from
/// the referenced partial file. The process is recursive up to `MAX_DEPTH`.
pub fn resolve(ir: ScaffoldIR, source_dir: &Path) -> Result<ScaffoldIR, ResolveError> {
    let mut seen = HashSet::new();
    let nodes = resolve_nodes(ir.nodes, source_dir, &mut seen, 0)?;
    Ok(ScaffoldIR {
        version: ir.version,
        template: ir.template,
        nodes,
    })
}

fn resolve_nodes(
    nodes: Vec<ScaffoldNode>,
    source_dir: &Path,
    seen: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<Vec<ScaffoldNode>, ResolveError> {
    if depth > MAX_DEPTH {
        return Err(ResolveError::DepthExceeded);
    }

    let mut out: Vec<ScaffoldNode> = Vec::new();

    for node in nodes {
        if let Some(ref ref_path) = node.ref_path {
            let abs = source_dir.join(ref_path);
            let canonical = abs.canonicalize().map_err(|_| ResolveError::NotFound {
                path: abs.clone(),
            })?;

            if seen.contains(&canonical) {
                return Err(ResolveError::Cycle {
                    path: canonical,
                });
            }
            seen.insert(canonical.clone());

            let partial = ScaffoldIR::load(&canonical).map_err(|detail| {
                ResolveError::InvalidJson {
                    path: canonical.clone(),
                    detail,
                }
            })?;

            let partial_dir = canonical
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();

            let resolved_nodes =
                resolve_nodes(partial.nodes, &partial_dir, seen, depth + 1)?;
            out.extend(resolved_nodes);

            seen.remove(&canonical);
        } else {
            out.push(node);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::scaffold_ir::ScaffoldNode;
    use tempfile::TempDir;

    fn dir_node(path: &str) -> ScaffoldNode {
        ScaffoldNode {
            node_type: Some("dir".to_string()),
            path: Some(path.to_string()),
            content: None,
            ref_path: None,
        }
    }

    fn ref_node(path: &str) -> ScaffoldNode {
        ScaffoldNode {
            node_type: None,
            path: None,
            content: None,
            ref_path: Some(path.to_string()),
        }
    }

    fn write_partial(dir: &Path, name: &str, nodes: Vec<ScaffoldNode>) {
        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes,
        };
        let json = serde_json::to_string_pretty(&ir).unwrap();
        std::fs::write(dir.join(name), json).unwrap();
    }

    #[test]
    fn flat_template_unchanged() {
        let tmp = TempDir::new().unwrap();
        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes: vec![dir_node("src"), dir_node("tests")],
        };
        let result = resolve(ir, tmp.path()).unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].path.as_deref(), Some("src"));
    }

    #[test]
    fn single_ref_inlined() {
        let tmp = TempDir::new().unwrap();
        write_partial(tmp.path(), "partial.json", vec![dir_node("tests")]);

        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes: vec![dir_node("src"), ref_node("partial.json")],
        };
        let result = resolve(ir, tmp.path()).unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[1].path.as_deref(), Some("tests"));
    }

    #[test]
    fn nested_refs_inlined() {
        let tmp = TempDir::new().unwrap();
        write_partial(tmp.path(), "inner.json", vec![dir_node("inner")]);
        write_partial(
            tmp.path(),
            "outer.json",
            vec![dir_node("outer"), ref_node("inner.json")],
        );

        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes: vec![ref_node("outer.json")],
        };
        let result = resolve(ir, tmp.path()).unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].path.as_deref(), Some("outer"));
        assert_eq!(result.nodes[1].path.as_deref(), Some("inner"));
    }

    #[test]
    fn missing_ref_errors() {
        let tmp = TempDir::new().unwrap();
        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes: vec![ref_node("nonexistent.json")],
        };
        assert!(matches!(resolve(ir, tmp.path()), Err(ResolveError::NotFound { .. })));
    }

    #[test]
    fn cycle_detected() {
        let tmp = TempDir::new().unwrap();
        // a.json → b.json → a.json
        write_partial(tmp.path(), "b.json", vec![ref_node("a.json")]);
        write_partial(tmp.path(), "a.json", vec![ref_node("b.json")]);

        let ir = ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes: vec![ref_node("a.json")],
        };
        assert!(matches!(resolve(ir, tmp.path()), Err(ResolveError::Cycle { .. })));
    }
}
