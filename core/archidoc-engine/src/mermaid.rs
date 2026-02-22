use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use archidoc_types::{C4Level, ModuleDoc};

/// Build a lookup map from short names and full paths to canonical module paths.
///
/// Every module's full path maps to itself. Additionally, the last segment of
/// each path (the "short name") maps to the full path — unless two modules
/// share the same short name, in which case the ambiguous entry is removed
/// and callers must use a qualified path.
fn build_name_map(docs: &[ModuleDoc]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut ambiguous: HashSet<String> = HashSet::new();

    for doc in docs {
        // Full path always maps to itself.
        map.insert(doc.module_path.clone(), doc.module_path.clone());

        // Short name (last segment) maps to full path.
        let short_name = doc
            .module_path
            .split('.')
            .last()
            .unwrap_or(&doc.module_path)
            .to_string();
        if short_name != doc.module_path {
            if let Some(existing) = map.get(&short_name) {
                if *existing != doc.module_path {
                    ambiguous.insert(short_name.clone());
                }
            } else {
                map.insert(short_name, doc.module_path.clone());
            }
        }
    }

    // Remove ambiguous short-name entries.
    for name in &ambiguous {
        map.remove(name);
    }

    map
}

/// Resolve a relationship target to a Mermaid node ID.
///
/// 1. Looks up the target in the name map (handles both short names like
///    "session_store" and qualified paths like "app.backend.session_store").
/// 2. Validates the resolved path against declared nodes. If it matches, returns
///    the Mermaid node ID directly.
/// 3. If unresolved, finds the longest declared-node prefix (collapses to nearest
///    parent). This handles auto-discovered imports to sub-files that aren't
///    themselves declared components (e.g. "tests.drivers.stubs.stubFoo" collapses
///    to "tests.drivers").
/// 4. Returns `None` if no declared node matches at all — the Rel should be skipped.
fn resolve_rel_target(
    target: &str,
    name_map: &BTreeMap<String, String>,
    declared_paths: &HashSet<String>,
) -> Option<String> {
    let resolved = name_map
        .get(target)
        .cloned()
        .unwrap_or_else(|| target.to_string());

    // Direct match against a declared node.
    if declared_paths.contains(&resolved) {
        return Some(resolved.replace('.', "_"));
    }

    // Collapse to the longest declared-node prefix.
    declared_paths
        .iter()
        .filter(|p| resolved.starts_with(&format!("{}.", p)))
        .max_by_key(|p| p.len())
        .map(|p| p.replace('.', "_"))
}

/// Return the Mermaid C4 container diagram as a markdown code block string.
pub fn container_diagram(docs: &[ModuleDoc]) -> String {
    let containers: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.c4_level == C4Level::Container)
        .collect();

    let mut container_defs = String::new();
    for doc in &containers {
        let id = doc.module_path.replace('.', "_");
        let name = to_title_case(&doc.module_path);
        container_defs.push_str(&format!(
            "        Container({}, \"{}\", \"{}\", \"{}\")\n",
            id, name, doc.pattern, doc.description
        ));
    }

    let name_map = build_name_map(docs);
    // Only container-level modules are rendered as nodes in this diagram.
    let declared_paths: HashSet<String> = containers.iter().map(|d| d.module_path.clone()).collect();

    let mut rel_defs = String::new();
    let mut seen_rels: HashSet<String> = HashSet::new();
    for doc in &containers {
        let from_id = doc.module_path.replace('.', "_");
        for rel in &doc.relationships {
            if let Some(to_id) = resolve_rel_target(&rel.target, &name_map, &declared_paths) {
                let rel_key = format!("{}|{}|{}", from_id, to_id, rel.label);
                if seen_rels.insert(rel_key) {
                    rel_defs.push_str(&format!(
                        "    Rel({}, {}, \"{}\", \"{}\")\n",
                        from_id, to_id, rel.label, rel.protocol
                    ));
                }
            }
        }
    }

    format!(
        "```mermaid\nC4Container\n    title Container Diagram\n\n    System_Boundary(sys, \"System\") {{\n{}    }}\n\n{}\n    UpdateLayoutConfig($c4ShapeInRow=\"3\", $c4BoundaryInRow=\"1\")\n```",
        container_defs,
        rel_defs,
    )
}

/// Generate Mermaid C4 container diagram file from `@c4 container` modules.
pub fn generate_container(output_dir: &Path, docs: &[ModuleDoc]) {
    let filepath = output_dir.join("c4-container.md");

    let containers: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.c4_level == C4Level::Container)
        .collect();

    let table_rows: Vec<String> = containers
        .iter()
        .map(|d| format!("| {} | {} | {} |", d.module_path, d.pattern, d.description))
        .collect();

    let content = format!(
        "# C4 Container Diagram\n\n> Auto-generated by archidoc\n\n{}\n\n## Containers\n\n| Container | Pattern | Description |\n|-----------|---------|-------------|\n{}\n",
        container_diagram(docs),
        table_rows.join("\n")
    );

    fs::write(&filepath, content).expect("Failed to write c4-container.md");
}

/// Return the Mermaid C4 component diagram as a markdown code block string.
///
/// Components are grouped by their nearest container (longest matching prefix),
/// then arranged as a tree within each container using nested `Container_Boundary`
/// blocks. Parent-child containment arrows are emitted automatically.
pub fn component_diagram(docs: &[ModuleDoc]) -> String {
    let components: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.c4_level == C4Level::Component)
        .collect();

    let containers: Vec<&ModuleDoc> = docs
        .iter()
        .filter(|d| d.c4_level == C4Level::Container)
        .collect();

    // Group components by their nearest container (longest prefix match).
    // Falls back to the parent_container field if no container prefix matches.
    let mut by_container: BTreeMap<String, Vec<&ModuleDoc>> = BTreeMap::new();
    for comp in &components {
        let container = containers
            .iter()
            .filter(|c| comp.module_path.starts_with(&format!("{}.", c.module_path)))
            .max_by_key(|c| c.module_path.len())
            .map(|c| c.module_path.clone())
            .unwrap_or_else(|| {
                comp.parent_container
                    .clone()
                    .unwrap_or_else(|| "other".to_string())
            });
        by_container.entry(container).or_default().push(comp);
    }

    let mut boundary_defs = String::new();
    let mut containment_rels: Vec<(String, String)> = Vec::new();

    for (container_path, comps) in &by_container {
        let container_id = container_path.replace('.', "_");
        let container_name = to_title_case(container_path);

        // Build immediate-parent map within this container group.
        // A component X is the immediate parent of Y if X.module_path is the
        // longest prefix of Y.module_path among all components in this group.
        let paths: Vec<&str> = comps.iter().map(|d| d.module_path.as_str()).collect();
        let mut parent_of: BTreeMap<&str, &str> = BTreeMap::new();
        let mut children_of: BTreeMap<&str, Vec<&ModuleDoc>> = BTreeMap::new();

        for comp in comps {
            let parent = paths
                .iter()
                .filter(|p| **p != comp.module_path)
                .filter(|p| comp.module_path.starts_with(&format!("{}.", p)))
                .max_by_key(|p| p.len());
            if let Some(p) = parent {
                parent_of.insert(&comp.module_path, p);
                children_of
                    .entry(p)
                    .or_default()
                    .push(comp);
                containment_rels.push((p.to_string(), comp.module_path.clone()));
            }
        }

        // Roots are components with no parent within this container group.
        let roots: Vec<&&ModuleDoc> = comps
            .iter()
            .filter(|d| !parent_of.contains_key(d.module_path.as_str()))
            .collect();

        boundary_defs.push_str(&format!(
            "    Container_Boundary({}_boundary, \"{}\") {{\n",
            container_id, container_name
        ));

        for root in &roots {
            emit_node(&mut boundary_defs, root, &children_of, 2);
        }

        boundary_defs.push_str("    }\n\n");
    }

    // Containment arrows (parent -> child)
    let mut rel_defs = String::new();
    for (from, to) in &containment_rels {
        let from_id = from.replace('.', "_");
        let to_id = to.replace('.', "_");
        rel_defs.push_str(&format!(
            "    Rel({}, {}, \"contains\")\n",
            from_id, to_id
        ));
    }

    // User-defined @c4 uses relationships
    let name_map = build_name_map(docs);
    // Components can reference both components (in this diagram) and containers
    // (declared in the container diagram). Include both levels as valid targets.
    let declared_paths: HashSet<String> = docs
        .iter()
        .filter(|d| d.c4_level == C4Level::Container || d.c4_level == C4Level::Component)
        .map(|d| d.module_path.clone())
        .collect();
    let mut seen_rels: HashSet<String> = HashSet::new();
    for doc in &components {
        let from_id = doc.module_path.replace('.', "_");
        for rel in &doc.relationships {
            if let Some(to_id) = resolve_rel_target(&rel.target, &name_map, &declared_paths) {
                let rel_key = format!("{}|{}|{}", from_id, to_id, rel.label);
                if seen_rels.insert(rel_key) {
                    rel_defs.push_str(&format!(
                        "    Rel({}, {}, \"{}\", \"{}\")\n",
                        from_id, to_id, rel.label, rel.protocol
                    ));
                }
            }
        }
    }

    format!(
        "```mermaid\nC4Component\n    title Component Diagram (GoF Patterns)\n\n{}{}```",
        boundary_defs, rel_defs
    )
}

/// Recursively emit a component node. If the node has children, wrap them
/// in a nested `Container_Boundary` with the parent component inside.
fn emit_node(
    out: &mut String,
    doc: &ModuleDoc,
    children_of: &BTreeMap<&str, Vec<&ModuleDoc>>,
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let id = doc.module_path.replace('.', "_");
    let name = doc
        .module_path
        .split('.')
        .last()
        .unwrap_or(&doc.module_path);

    if let Some(kids) = children_of.get(doc.module_path.as_str()) {
        // Parent node: emit a sub-boundary containing itself + children
        out.push_str(&format!(
            "{}Container_Boundary({}_boundary, \"{}\") {{\n",
            indent, id, name
        ));
        out.push_str(&format!(
            "{}    Component({}, \"{}\", \"{}\", \"{}\")\n",
            indent, id, name, doc.pattern, doc.description
        ));
        for kid in kids {
            emit_node(out, kid, children_of, depth + 1);
        }
        out.push_str(&format!("{}}}\n", indent));
    } else {
        // Leaf node
        out.push_str(&format!(
            "{}Component({}, \"{}\", \"{}\", \"{}\")\n",
            indent, id, name, doc.pattern, doc.description
        ));
    }
}

/// Generate Mermaid C4 component diagram file from `@c4 component` modules.
pub fn generate_component(output_dir: &Path, docs: &[ModuleDoc]) {
    let filepath = output_dir.join("c4-component.md");

    let content = format!(
        "# C4 Component Diagram\n\n> Auto-generated by archidoc\n\n{}\n",
        component_diagram(docs)
    );

    fs::write(&filepath, content).expect("Failed to write c4-component.md");
}

fn to_title_case(s: &str) -> String {
    s.split('.')
        .last()
        .unwrap_or(s)
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
