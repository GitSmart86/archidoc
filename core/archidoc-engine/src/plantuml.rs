use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::C4Level;

/// Generate PlantUML C4 container diagram.
pub fn generate_container(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-container.puml");

    let all_annotated = ir.annotated_dirs();
    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    let mut container_defs = String::new();
    for dir in &containers {
        let id = to_puml_id(&dir.path);
        let name = to_title_case(&dir.name);
        let pattern = dir.pattern.as_deref().unwrap_or("--");
        let desc = dir.description.as_deref().unwrap_or("");
        container_defs.push_str(&format!(
            "    Container({}, \"{}\", \"{}\", \"{}\")\n",
            id, name, pattern, desc
        ));
    }

    let mut rel_defs = String::new();
    for dir in &containers {
        let from_id = to_puml_id(&dir.path);
        for rel in &dir.relationships {
            let to_id = to_puml_id(&rel.target);
            rel_defs.push_str(&format!(
                "Rel({}, {}, \"{}\", \"{}\")\n",
                from_id, to_id, rel.label, rel.protocol
            ));
        }
    }

    let content = format!(
        r#"@startuml c4-container
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

title Container Diagram

System_Boundary(sys, "System") {{
{}}}

{}
@enduml
"#,
        container_defs, rel_defs
    );

    fs::write(&filepath, content).expect("Failed to write c4-container.puml");
}

/// Generate PlantUML C4 component diagram.
pub fn generate_component(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-component.puml");

    let all_annotated = ir.annotated_dirs();
    let components: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Component))
        .collect();

    let mut grouped: BTreeMap<String, Vec<&DirNode>> = BTreeMap::new();
    for dir in &components {
        let parent = dir
            .parent
            .clone()
            .unwrap_or_else(|| "other".to_string());
        grouped.entry(parent).or_default().push(dir);
    }

    let mut boundary_defs = String::new();
    for (parent, component_dirs) in &grouped {
        let parent_id = to_puml_id(parent);
        let parent_name = to_title_case(parent.split('/').last().unwrap_or(parent));
        boundary_defs.push_str(&format!(
            "Container_Boundary({}_boundary, \"{}\") {{\n",
            parent_id, parent_name
        ));
        for dir in component_dirs {
            let id = to_puml_id(&dir.path);
            let name = &dir.name;
            let pattern = dir.pattern.as_deref().unwrap_or("--");
            let desc = dir.description.as_deref().unwrap_or("");
            boundary_defs.push_str(&format!(
                "    Component({}, \"{}\", \"{}\", \"{}\")\n",
                id, name, pattern, desc
            ));
        }
        boundary_defs.push_str("}\n\n");
    }

    let mut rel_defs = String::new();
    for dir in &components {
        let from_id = to_puml_id(&dir.path);
        for rel in &dir.relationships {
            let to_id = to_puml_id(&rel.target);
            rel_defs.push_str(&format!(
                "Rel({}, {}, \"{}\", \"{}\")\n",
                from_id, to_id, rel.label, rel.protocol
            ));
        }
    }

    let content = format!(
        r#"@startuml c4-component
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

title Component Diagram (GoF Patterns)

{}{}
@enduml
"#,
        boundary_defs, rel_defs
    );

    fs::write(&filepath, content).expect("Failed to write c4-component.puml");
}

/// Generate the PlantUML C4 code diagram from `@c4 code` elements.
///
/// Each annotated component that declares code elements becomes a
/// `Container_Boundary`, with one `Component(...)` per element (kind shown as
/// the technology tag) and any `@c4 uses` relationships as `Rel(...)` arrows.
/// Emits nothing when no code element exists.
pub fn generate_code(output_dir: &Path, ir: &ArchitectureIR) {
    let owners: Vec<&DirNode> = ir
        .annotated_dirs()
        .into_iter()
        .filter(|d| !d.code_elements.is_empty())
        .collect();
    if owners.is_empty() {
        return;
    }

    // Map a code element's bare name to its qualified puml id so that
    // intra-component `@c4 uses StorageEntity` arrows land on the defined node
    // instead of auto-creating a bare one.
    let mut by_name: BTreeMap<&str, String> = BTreeMap::new();
    for owner in &owners {
        for el in &owner.code_elements {
            by_name.insert(
                el.name.as_str(),
                to_puml_id(&format!("{}__{}", owner.path, el.name)),
            );
        }
    }

    let mut boundary_defs = String::new();
    let mut rel_defs = String::new();
    for owner in &owners {
        let boundary_id = to_puml_id(&owner.path);
        boundary_defs.push_str(&format!(
            "Container_Boundary({}_code, \"{}\") {{\n",
            boundary_id, owner.name
        ));
        for el in &owner.code_elements {
            let id = to_puml_id(&format!("{}__{}", owner.path, el.name));
            let desc = el.description.as_deref().unwrap_or("");
            boundary_defs.push_str(&format!(
                "    Component({}, \"{}\", \"{}\", \"{}\")\n",
                id, el.name, el.kind, desc
            ));
            for rel in &el.relationships {
                let to_id = by_name
                    .get(rel.target.as_str())
                    .cloned()
                    .unwrap_or_else(|| to_puml_id(&rel.target));
                rel_defs.push_str(&format!(
                    "Rel({}, {}, \"{}\", \"{}\")\n",
                    id, to_id, rel.label, rel.protocol
                ));
            }
        }
        boundary_defs.push_str("}\n\n");
    }

    // Realization arrows from `impl Trait for Type`. The implementing type is
    // always a `@c4 code` element (the walker filtered on that); draw the arrow
    // only when the trait is one too, so realizations stay within the curated
    // set rather than auto-creating bare nodes for std/derive traits.
    for owner in &owners {
        for ti in &owner.trait_impls {
            let (Some(type_id), Some(trait_id)) = (
                by_name.get(ti.type_name.as_str()),
                by_name.get(ti.trait_name.as_str()),
            ) else {
                continue;
            };
            rel_defs.push_str(&format!(
                "Rel({}, {}, \"implements\", \"trait\")\n",
                type_id, trait_id
            ));
        }
    }

    let content = format!(
        r#"@startuml c4-code
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

title Code Diagram (@c4 code elements)

{}{}
@enduml
"#,
        boundary_defs, rel_defs
    );

    fs::write(output_dir.join("c4-code.puml"), content)
        .expect("Failed to write c4-code.puml");
}

fn to_puml_id(s: &str) -> String {
    s.replace('.', "_").replace('/', "_").replace('-', "_")
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

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{ArchitectureIR, CodeElement, DirNode, TraitImpl};
    use archidoc_types::C4Level;

    fn code_owner(name: &str, elements: &[(&str, &str)], impls: &[(&str, &str)]) -> DirNode {
        let mut d = DirNode::empty(name, name);
        d.c4_level = Some(C4Level::Component);
        d.code_elements = elements
            .iter()
            .map(|(n, k)| CodeElement {
                name: n.to_string(),
                kind: k.to_string(),
                description: None,
                relationships: vec![],
            })
            .collect();
        d.trait_impls = impls
            .iter()
            .map(|(t, tr)| TraitImpl {
                type_name: t.to_string(),
                trait_name: tr.to_string(),
            })
            .collect();
        d
    }

    #[test]
    fn code_diagram_draws_realization_only_when_both_are_code() {
        let core = code_owner("core", &[("FileFormatAdapter", "trait")], &[]);
        // Md implements one @c4 code trait and one un-annotated trait (Clone).
        let md = code_owner(
            "markdown",
            &[("Md", "struct")],
            &[("Md", "FileFormatAdapter"), ("Md", "Clone")],
        );
        let mut ir = ArchitectureIR::new("/scan".to_string());
        ir.root.dirs = vec![core, md];

        let dir = std::env::temp_dir().join("archidoc_trait_impl_test");
        std::fs::create_dir_all(&dir).unwrap();
        generate_code(&dir, &ir);
        let out = std::fs::read_to_string(dir.join("c4-code.puml")).unwrap();

        assert!(out.contains("Rel(markdown__Md, core__FileFormatAdapter, \"implements\", \"trait\")"));
        // Clone is not a @c4 code element → no realization arrow, no bare node.
        assert!(!out.contains("Clone"));
    }
}
