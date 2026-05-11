use archidoc_types::ir::{ArchitectureIR, DirNode};

/// Generate token-optimized AI context from IR.
///
/// Produces a compressed tree format: no Mermaid, no ASCII art, no tables.
/// Each module appears exactly once. ~75% fewer tokens than ARCHITECTURE.md.
pub fn generate(ir: &ArchitectureIR) -> String {
    let mut out = String::new();

    out.push_str("# Architecture (AI Context)\n\n");

    let narr = narrative(ir);
    if !narr.is_empty() {
        out.push_str(&narr);
        out.push('\n');
    }

    let tree = module_tree(ir);
    if !tree.is_empty() {
        out.push_str(&tree);
    }

    let rels = relationships(ir);
    if !rels.is_empty() {
        out.push('\n');
        out.push_str(&rels);
    }

    out
}

/// Extract prose from root content, skipping code blocks, tables, and markers.
fn narrative(ir: &ArchitectureIR) -> String {
    let content = match &ir.root.content {
        Some(c) if ir.root.is_annotated() => c,
        _ => return String::new(),
    };

    let mut lines: Vec<&str> = Vec::new();
    let mut in_code_block = false;
    let mut in_table = false;

    for line in content.lines() {
        let t = line.trim();

        // Skip fenced code blocks entirely (ASCII art, Mermaid, examples)
        if t.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if t.starts_with("@c4 ") {
            continue;
        }

        if t.starts_with("GoF:") {
            continue;
        }

        // Skip file table blocks
        if t.starts_with("| File") || t.starts_with("| file") {
            in_table = true;
            continue;
        }
        if in_table {
            if t.starts_with('|') {
                continue;
            }
            in_table = false;
        }

        lines.push(line);
    }

    // Remove orphaned headers (headers with no content before next header or end)
    let mut filtered: Vec<&str> = Vec::new();
    for (i, &line) in lines.iter().enumerate() {
        if line.trim().starts_with('#') {
            let has_content = lines[i + 1..]
                .iter()
                .take_while(|l| !l.trim().starts_with('#'))
                .any(|l| !l.trim().is_empty());
            if !has_content {
                continue;
            }
        }
        filtered.push(line);
    }

    // Collapse multiple blank lines
    let mut text = filtered.join("\n").trim().to_string();
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }

    if text.is_empty() {
        String::new()
    } else {
        format!("{}\n", text)
    }
}

/// Build indented module tree with pattern and description.
///
/// Walks the IR tree recursively — the tree IS the nesting.
/// No common_prefix logic needed.
fn module_tree(ir: &ArchitectureIR) -> String {
    let mut out = String::new();
    // Walk children of root (root itself is skipped, like _lib was)
    for child in &ir.root.dirs {
        emit_tree_node(child, 0, &mut out);
    }
    out
}

/// Recursively emit a DirNode into the tree output.
fn emit_tree_node(dir: &DirNode, depth: usize, out: &mut String) {
    if dir.is_annotated() {
        let indent = "  ".repeat(depth);
        let name = &dir.name;

        out.push_str(&indent);
        out.push_str(name);
        out.push('/');

        let pattern = dir.pattern.as_deref().unwrap_or("--");
        if pattern != "--" {
            out.push(' ');
            out.push_str(pattern);
        }

        let desc = dir.description.as_deref().unwrap_or("");
        if !desc.is_empty() {
            out.push_str(" — ");
            out.push_str(desc);
        }

        out.push('\n');

        // Children of an annotated node are indented one deeper
        for child in &dir.dirs {
            emit_tree_node(child, depth + 1, out);
        }
    } else {
        // Unannotated intermediate: pass through at same depth
        for child in &dir.dirs {
            emit_tree_node(child, depth, out);
        }
    }
}

/// Flat relationship list.
fn relationships(ir: &ArchitectureIR) -> String {
    let annotated = ir.annotated_dirs();
    let modules: Vec<&&DirNode> = annotated
        .iter()
        .filter(|d| d.path != ".")
        .collect();

    let rels: Vec<_> = modules
        .iter()
        .flat_map(|dir| {
            dir.relationships.iter().map(move |r| {
                let src = &dir.path;
                let tgt = &r.target;
                (src.to_string(), tgt.to_string(), r.label.clone(), r.protocol.clone())
            })
        })
        .collect();

    if rels.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for (src, tgt, label, proto) in &rels {
        out.push_str(&format!(
            "{} -> {}: \"{}\" ({})\n",
            src, tgt, label, proto
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{ArchitectureIR, DirNode, Relationship};
    use archidoc_types::C4Level;

    fn make_ir_with_dirs(dirs: Vec<DirNode>, root_content: Option<String>) -> ArchitectureIR {
        let mut root = DirNode::empty(".", ".");
        if root_content.is_some() {
            root.c4_level = Some(C4Level::Container);
            root.content = root_content;
        }
        root.dirs = dirs;
        ArchitectureIR {
            version: "2.0".to_string(),
            scan_root: String::new(),
            root,
        }
    }

    fn make_annotated_dir(name: &str, path: &str, pattern: &str, desc: &str, level: C4Level) -> DirNode {
        let mut dir = DirNode::empty(name, path);
        dir.c4_level = Some(level);
        dir.pattern = if pattern == "--" { None } else { Some(pattern.to_string()) };
        dir.description = if desc.is_empty() { None } else { Some(desc.to_string()) };
        dir.source_file = Some(format!("{}/mod.rs", path));
        dir
    }

    #[test]
    fn empty_docs() {
        let ir = make_ir_with_dirs(vec![], None);
        let out = generate(&ir);
        assert!(out.contains("# Architecture (AI Context)"));
    }

    #[test]
    fn single_module() {
        let ir = make_ir_with_dirs(
            vec![make_annotated_dir("api", "api", "Facade", "REST gateway", C4Level::Container)],
            None,
        );
        let out = generate(&ir);
        assert!(out.contains("api/ Facade — REST gateway"));
    }

    #[test]
    fn nested_indentation() {
        let mut bus = make_annotated_dir("bus", "bus", "Mediator", "Bus", C4Level::Container);
        let mut calc = make_annotated_dir("calc", "bus/calc", "Strategy", "Calc", C4Level::Component);
        let ind = make_annotated_dir("ind", "bus/calc/ind", "--", "Indicators", C4Level::Component);
        calc.dirs.push(ind);
        bus.dirs.push(calc);

        let ir = make_ir_with_dirs(vec![bus], None);
        let out = generate(&ir);
        assert!(out.contains("bus/ Mediator — Bus\n"));
        assert!(out.contains("  calc/ Strategy — Calc\n"));
        assert!(out.contains("    ind/ — Indicators\n"));
    }

    #[test]
    fn narrative_skips_code_blocks() {
        let ir = make_ir_with_dirs(
            vec![],
            Some("# Title\n\nProse here.\n\n```text\n\u{250c}\u{2500}\u{2500}\u{2510}\n\u{2514}\u{2500}\u{2500}\u{2518}\n```\n\n## Flow\n\n1. Step one".to_string()),
        );
        let out = generate(&ir);
        assert!(out.contains("# Title"));
        assert!(out.contains("Prose here."));
        assert!(out.contains("## Flow"));
        assert!(out.contains("1. Step one"));
        assert!(!out.contains("\u{250c}"));
        assert!(!out.contains("```"));
    }

    #[test]
    fn narrative_skips_markers_and_tables() {
        let ir = make_ir_with_dirs(
            vec![],
            Some("@c4 container\n\n# Eng\n\nDesc.\n\n| File | Pattern |\n|------|---------|\n| `a` | Facade |\n\nGoF: Mediator".to_string()),
        );
        let out = generate(&ir);
        assert!(out.contains("# Eng"));
        assert!(out.contains("Desc."));
        assert!(!out.contains("@c4"));
        assert!(!out.contains("| File"));
        assert!(!out.contains("GoF:"));
    }

    #[test]
    fn dash_dash_pattern_hidden() {
        let ir = make_ir_with_dirs(
            vec![make_annotated_dir("types", "types", "--", "Core types", C4Level::Container)],
            None,
        );
        let out = generate(&ir);
        assert!(out.contains("types/ — Core types"));
        assert!(!out.contains("--"));
    }

    #[test]
    fn c4_context_mermaid_stripped_from_ai() {
        let ir = make_ir_with_dirs(
            vec![],
            Some("@c4 container\n\n# My App\n\nA cool app.\n\n## C4 Context\n\n```mermaid\nC4Context\n    Person(user, \"User\", \"Actor\")\n```\n\n## Data Flow\n\n1. A -> B -> C".to_string()),
        );
        let out = generate(&ir);
        assert!(out.contains("# My App"));
        assert!(out.contains("A cool app."));
        assert!(out.contains("## Data Flow"));
        assert!(out.contains("1. A -> B -> C"));
        assert!(!out.contains("C4 Context"));
        assert!(!out.contains("C4Context"));
        assert!(!out.contains("mermaid"));
    }

    #[test]
    fn relationships_included() {
        let mut api = make_annotated_dir("api", "x/api", "Facade", "API", C4Level::Container);
        api.relationships = vec![Relationship {
            target: "x/db".to_string(),
            label: "Persists".to_string(),
            protocol: "sqlx".to_string(),
        }];
        // Wrap in parent dir "x"
        let mut x = DirNode::empty("x", "x");
        x.dirs.push(api);
        x.dirs.push(make_annotated_dir("db", "x/db", "Repository", "DB", C4Level::Container));

        let ir = make_ir_with_dirs(vec![x], None);
        let out = generate(&ir);
        assert!(out.contains("x/api -> x/db: \"Persists\" (sqlx)"));
    }
}
