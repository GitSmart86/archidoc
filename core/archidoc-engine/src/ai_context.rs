use std::collections::HashSet;

use archidoc_types::ModuleDoc;

/// Generate token-optimized AI context from module documentation.
///
/// Produces a compressed tree format: no Mermaid, no ASCII art, no tables.
/// Each module appears exactly once. ~75% fewer tokens than ARCHITECTURE.md.
pub fn generate(docs: &[ModuleDoc]) -> String {
    let mut out = String::new();

    out.push_str("# Architecture (AI Context)\n\n");

    let narr = narrative(docs);
    if !narr.is_empty() {
        out.push_str(&narr);
        out.push('\n');
    }

    let tree = module_tree(docs);
    if !tree.is_empty() {
        out.push_str(&tree);
    }

    let rels = relationships(docs);
    if !rels.is_empty() {
        out.push('\n');
        out.push_str(&rels);
    }

    out
}

/// Extract prose from _lib content, skipping code blocks, tables, and markers.
fn narrative(docs: &[ModuleDoc]) -> String {
    let lib = match docs.iter().find(|d| d.module_path == "_lib") {
        Some(doc) => doc,
        None => return String::new(),
    };

    let mut lines: Vec<&str> = Vec::new();
    let mut in_code_block = false;
    let mut in_table = false;

    for line in lib.content.lines() {
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
fn module_tree(docs: &[ModuleDoc]) -> String {
    let mut modules: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.module_path != "_lib")
        .collect();
    modules.sort_by(|a, b| a.module_path.cmp(&b.module_path));

    if modules.is_empty() {
        return String::new();
    }

    let prefix = common_prefix(&modules);

    // Build set of short paths for depth computation
    let short_paths: HashSet<String> = modules
        .iter()
        .map(|d| {
            d.module_path
                .strip_prefix(&prefix)
                .unwrap_or(&d.module_path)
                .to_string()
        })
        .collect();

    let mut out = String::new();

    for doc in &modules {
        let short = doc
            .module_path
            .strip_prefix(&prefix)
            .unwrap_or(&doc.module_path);
        let name = short.rsplit('.').next().unwrap_or(short);

        // Depth = number of ancestor paths that are also modules in our set
        let parts: Vec<&str> = short.split('.').collect();
        let mut depth = 0;
        for i in 1..parts.len() {
            if short_paths.contains(&parts[..i].join(".")) {
                depth += 1;
            }
        }
        let indent = "  ".repeat(depth);

        out.push_str(&indent);
        out.push_str(name);
        out.push('/');

        if doc.pattern != "--" {
            out.push(' ');
            out.push_str(&doc.pattern);
        }

        if !doc.description.is_empty() {
            out.push_str(" — ");
            out.push_str(&doc.description);
        }

        out.push('\n');
    }

    out
}

/// Flat relationship list with short module names.
fn relationships(docs: &[ModuleDoc]) -> String {
    let modules: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.module_path != "_lib")
        .collect();
    let prefix = common_prefix(&modules);

    let rels: Vec<_> = modules
        .iter()
        .flat_map(|doc| {
            let prefix = &prefix;
            doc.relationships.iter().map(move |r| {
                let src = doc
                    .module_path
                    .strip_prefix(prefix)
                    .unwrap_or(&doc.module_path);
                let tgt = r.target.strip_prefix(prefix).unwrap_or(&r.target);
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

/// Find common dot-separated prefix across all module paths.
fn common_prefix(modules: &[&ModuleDoc]) -> String {
    if modules.len() < 2 {
        return String::new();
    }

    let first: Vec<&str> = modules[0].module_path.split('.').collect();
    let mut len = first.len();

    for m in &modules[1..] {
        let parts: Vec<&str> = m.module_path.split('.').collect();
        let mut n = 0;
        for (a, b) in first.iter().zip(parts.iter()) {
            if a == b {
                n += 1;
            } else {
                break;
            }
        }
        len = len.min(n);
    }

    // Don't strip beyond the shortest module path — every module must keep at least one segment
    let min_segments = modules
        .iter()
        .map(|m| m.module_path.matches('.').count() + 1)
        .min()
        .unwrap_or(1);
    len = len.min(min_segments - 1);

    if len == 0 {
        return String::new();
    }

    format!("{}.", first[..len].join("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::{C4Level, PatternStatus};

    fn doc(path: &str, pattern: &str, desc: &str, level: C4Level) -> ModuleDoc {
        ModuleDoc {
            module_path: path.to_string(),
            content: String::new(),
            source_file: format!("src/{}/mod.rs", path.replace('.', "/")),
            c4_level: level,
            pattern: pattern.to_string(),
            pattern_status: PatternStatus::Planned,
            description: desc.to_string(),
            parent_container: None,
            relationships: vec![],
            files: vec![],
        }
    }

    fn lib(content: &str) -> ModuleDoc {
        ModuleDoc {
            module_path: "_lib".to_string(),
            content: content.to_string(),
            source_file: "src/lib.rs".to_string(),
            c4_level: C4Level::Container,
            pattern: "--".to_string(),
            pattern_status: PatternStatus::Planned,
            description: String::new(),
            parent_container: None,
            relationships: vec![],
            files: vec![],
        }
    }

    #[test]
    fn empty_docs() {
        let out = generate(&[]);
        assert!(out.contains("# Architecture (AI Context)"));
    }

    #[test]
    fn single_module() {
        let docs = vec![doc("api", "Facade", "REST gateway", C4Level::Container)];
        let out = generate(&docs);
        assert!(out.contains("api/ Facade — REST gateway"));
    }

    #[test]
    fn strips_common_prefix() {
        let docs = vec![
            doc("a.b.foo", "Facade", "Foo", C4Level::Container),
            doc("a.b.bar", "--", "Bar", C4Level::Container),
        ];
        let out = generate(&docs);
        assert!(out.contains("bar/ — Bar"));
        assert!(out.contains("foo/ Facade — Foo"));
    }

    #[test]
    fn nested_indentation() {
        let docs = vec![
            doc("a.b.bus", "Mediator", "Bus", C4Level::Container),
            doc("a.b.bus.calc", "Strategy", "Calc", C4Level::Component),
            doc("a.b.bus.calc.ind", "--", "Indicators", C4Level::Component),
        ];
        let out = generate(&docs);
        assert!(out.contains("bus/ Mediator — Bus\n"));
        assert!(out.contains("  calc/ Strategy — Calc\n"));
        assert!(out.contains("    ind/ — Indicators\n"));
    }

    #[test]
    fn narrative_skips_code_blocks() {
        let docs = vec![lib(
            "# Title\n\nProse here.\n\n```text\n\u{250c}\u{2500}\u{2500}\u{2510}\n\u{2514}\u{2500}\u{2500}\u{2518}\n```\n\n## Flow\n\n1. Step one",
        )];
        let out = generate(&docs);
        assert!(out.contains("# Title"));
        assert!(out.contains("Prose here."));
        assert!(out.contains("## Flow"));
        assert!(out.contains("1. Step one"));
        assert!(!out.contains("\u{250c}"));
        assert!(!out.contains("```"));
    }

    #[test]
    fn narrative_skips_markers_and_tables() {
        let docs = vec![lib(
            "@c4 container\n\n# Eng\n\nDesc.\n\n| File | Pattern |\n|------|---------|\n| `a` | Facade |\n\nGoF: Mediator",
        )];
        let out = generate(&docs);
        assert!(out.contains("# Eng"));
        assert!(out.contains("Desc."));
        assert!(!out.contains("@c4"));
        assert!(!out.contains("| File"));
        assert!(!out.contains("GoF:"));
    }

    #[test]
    fn dash_dash_pattern_hidden() {
        let docs = vec![doc("types", "--", "Core types", C4Level::Container)];
        let out = generate(&docs);
        assert!(out.contains("types/ — Core types"));
        assert!(!out.contains("--"));
    }

    #[test]
    fn c4_context_mermaid_stripped_from_ai() {
        // Simulates the init template's C4 Context section — mermaid block should be
        // stripped entirely, and the orphaned ## C4 Context header removed.
        let docs = vec![lib(
            "@c4 container\n\n# My App\n\nA cool app.\n\n## C4 Context\n\n```mermaid\nC4Context\n    Person(user, \"User\", \"Actor\")\n```\n\n## Data Flow\n\n1. A -> B -> C",
        )];
        let out = generate(&docs);
        assert!(out.contains("# My App"));
        assert!(out.contains("A cool app."));
        assert!(out.contains("## Data Flow"));
        assert!(out.contains("1. A -> B -> C"));
        // C4 Context section should be gone
        assert!(!out.contains("C4 Context"));
        assert!(!out.contains("C4Context"));
        assert!(!out.contains("mermaid"));
    }

    #[test]
    fn relationships_included() {
        let mut api = doc("x.api", "Facade", "API", C4Level::Container);
        api.relationships = vec![archidoc_types::Relationship {
            target: "x.db".to_string(),
            label: "Persists".to_string(),
            protocol: "sqlx".to_string(),
        }];
        let docs = vec![api, doc("x.db", "Repository", "DB", C4Level::Container)];
        let out = generate(&docs);
        assert!(out.contains("api -> db: \"Persists\" (sqlx)"));
    }
}
