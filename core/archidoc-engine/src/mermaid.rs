use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::C4Level;

/// Build a lookup map from short names and full paths to canonical dir paths.
///
/// Maps both slash-separated paths (IR format) and dot-separated paths
/// (parser/relationship format) to the canonical slash-separated path.
fn build_name_map(dirs: &[&DirNode]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut ambiguous: HashSet<String> = HashSet::new();

    for dir in dirs {
        // Full path always maps to itself (slash format).
        map.insert(dir.path.clone(), dir.path.clone());
        // Also map dot-separated form for backward compat with relationship targets
        let dot_path = dir.path.replace('/', ".");
        if dot_path != dir.path {
            map.insert(dot_path, dir.path.clone());
        }

        // Short name (last segment) maps to full path.
        let short_name = dir.name.clone();
        if short_name != dir.path {
            if let Some(existing) = map.get(&short_name) {
                if *existing != dir.path {
                    ambiguous.insert(short_name.clone());
                }
            } else {
                map.insert(short_name, dir.path.clone());
            }
        }
    }

    for name in &ambiguous {
        map.remove(name);
    }

    map
}

/// Resolve a relationship target to a Mermaid node ID.
///
/// Relationship targets come from the parser in dot-separated format (e.g.,
/// "app.backend.types"). Directory paths in the IR use slash separators
/// (e.g., "app/backend/types"). This function handles both formats.
fn resolve_rel_target(
    target: &str,
    name_map: &BTreeMap<String, String>,
    declared_paths: &HashSet<String>,
) -> Option<String> {
    let resolved = name_map
        .get(target)
        .cloned()
        .unwrap_or_else(|| target.to_string());

    // Direct match
    if declared_paths.contains(&resolved) {
        return Some(to_mermaid_id(&resolved));
    }

    // Try converting dots to slashes (parser targets use dots, IR paths use slashes)
    let slash_resolved = resolved.replace('.', "/");
    if declared_paths.contains(&slash_resolved) {
        return Some(to_mermaid_id(&slash_resolved));
    }

    // Collapse to the longest declared-node prefix
    // Check both dot and slash formats
    declared_paths
        .iter()
        .filter(|p| {
            resolved.starts_with(&format!("{}.", p))
                || resolved.starts_with(&format!("{}/", p))
                || slash_resolved.starts_with(&format!("{}/", p))
        })
        .max_by_key(|p| p.len())
        .map(|p| to_mermaid_id(p))
}

/// Return the Mermaid C4 container diagram as a markdown code block string.
pub fn container_diagram(ir: &ArchitectureIR) -> String {
    let all_annotated = ir.annotated_dirs();
    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    let mut container_defs = String::new();
    for dir in &containers {
        let id = to_mermaid_id(&dir.path);
        let name = to_title_case(&dir.name);
        let pattern = dir.pattern.as_deref().unwrap_or("--");
        let desc = dir.description.as_deref().unwrap_or("");
        container_defs.push_str(&format!(
            "        Container({}, \"{}\", \"{}\", \"{}\")\n",
            id, name, pattern, desc
        ));
    }

    let name_map = build_name_map(&containers);
    let declared_paths: HashSet<String> = containers.iter().map(|d| d.path.clone()).collect();

    let mut rel_defs = String::new();
    let mut seen_rels: HashSet<String> = HashSet::new();
    for dir in &containers {
        let from_id = to_mermaid_id(&dir.path);
        for rel in &dir.relationships {
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

/// Generate Mermaid C4 container diagram file.
pub fn generate_container(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-container.md");

    let all_annotated = ir.annotated_dirs();
    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    let table_rows: Vec<String> = containers
        .iter()
        .map(|d| {
            let pattern = d.pattern.as_deref().unwrap_or("--");
            let desc = d.description.as_deref().unwrap_or("");
            format!("| {} | {} | {} |", d.path, pattern, desc)
        })
        .collect();

    let content = format!(
        "# C4 Container Diagram\n\n> Auto-generated by archidoc\n\n{}\n\n## Containers\n\n| Container | Pattern | Description |\n|-----------|---------|-------------|\n{}\n",
        container_diagram(ir),
        table_rows.join("\n")
    );

    fs::write(&filepath, content).expect("Failed to write c4-container.md");
}

/// Return the Mermaid C4 component diagram as a markdown code block string.
pub fn component_diagram(ir: &ArchitectureIR) -> String {
    let all_annotated = ir.annotated_dirs();
    let components: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Component))
        .collect();

    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    // Group components by their nearest container (longest prefix match).
    let mut by_container: BTreeMap<String, Vec<&DirNode>> = BTreeMap::new();
    for comp in &components {
        let container = containers
            .iter()
            .filter(|c| comp.path.starts_with(&format!("{}/", c.path)))
            .max_by_key(|c| c.path.len())
            .map(|c| c.path.clone())
            .unwrap_or_else(|| {
                comp.parent
                    .clone()
                    .unwrap_or_else(|| "other".to_string())
            });
        by_container.entry(container).or_default().push(comp);
    }

    let mut boundary_defs = String::new();

    for (container_path, comps) in &by_container {
        let container_id = to_mermaid_id(container_path);
        let container_name = to_title_case(container_path.split('/').last().unwrap_or(container_path));

        let paths: Vec<&str> = comps.iter().map(|d| d.path.as_str()).collect();
        let mut parent_of: BTreeMap<&str, &str> = BTreeMap::new();
        let mut children_of: BTreeMap<&str, Vec<&DirNode>> = BTreeMap::new();

        for comp in comps {
            let parent = paths
                .iter()
                .filter(|p| **p != comp.path)
                .filter(|p| comp.path.starts_with(&format!("{}/", p)))
                .max_by_key(|p| p.len());
            if let Some(p) = parent {
                parent_of.insert(&comp.path, p);
                children_of
                    .entry(p)
                    .or_default()
                    .push(comp);
            }
        }

        let roots: Vec<&&DirNode> = comps
            .iter()
            .filter(|d| !parent_of.contains_key(d.path.as_str()))
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

    // User-defined @c4 uses relationships
    let mut rel_defs = String::new();
    let all_dirs: Vec<&DirNode> = all_annotated.iter().copied().collect();
    let name_map = build_name_map(&all_dirs);
    let declared_paths: HashSet<String> = all_annotated
        .iter()
        .filter(|d| d.c4_level == Some(C4Level::Container) || d.c4_level == Some(C4Level::Component))
        .map(|d| d.path.clone())
        .collect();
    let mut seen_rels: HashSet<String> = HashSet::new();
    for comp in &components {
        let from_id = to_mermaid_id(&comp.path);
        for rel in &comp.relationships {
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

/// Emit a component node as a flat `Component()` entry.
fn emit_node(
    out: &mut String,
    dir: &DirNode,
    _children_of: &BTreeMap<&str, Vec<&DirNode>>,
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let id = to_mermaid_id(&dir.path);
    let name = &dir.name;
    let pattern = dir.pattern.as_deref().unwrap_or("--");
    let desc = dir.description.as_deref().unwrap_or("");

    out.push_str(&format!(
        "{}Component({}, \"{}\", \"{}\", \"{}\")\n",
        indent, id, name, pattern, desc
    ));
}

/// Generate Mermaid C4 component diagram file.
pub fn generate_component(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-component.md");

    let content = format!(
        "# C4 Component Diagram\n\n> Auto-generated by archidoc\n\n{}\n",
        component_diagram(ir)
    );

    fs::write(&filepath, content).expect("Failed to write c4-component.md");
}

/// Convert a directory path to a valid Mermaid node identifier.
fn to_mermaid_id(s: &str) -> String {
    s.replace('.', "_").replace('-', "_").replace('/', "_")
}

fn to_title_case(s: &str) -> String {
    s.split('_')
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
