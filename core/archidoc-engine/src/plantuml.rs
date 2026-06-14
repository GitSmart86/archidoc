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
        let name = escape_label(&to_title_case(&dir.name));
        let pattern = escape_label(dir.pattern.as_deref().unwrap_or("--"));
        let desc = escape_label(dir.description.as_deref().unwrap_or(""));
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
                from_id,
                to_id,
                escape_label(&rel.label),
                escape_label(&rel.protocol)
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
        let parent_name = escape_label(&to_title_case(parent.split('/').last().unwrap_or(parent)));
        boundary_defs.push_str(&format!(
            "Container_Boundary({}_boundary, \"{}\") {{\n",
            parent_id, parent_name
        ));
        for dir in component_dirs {
            let id = to_puml_id(&dir.path);
            let name = escape_label(&dir.name);
            let pattern = escape_label(dir.pattern.as_deref().unwrap_or("--"));
            let desc = escape_label(dir.description.as_deref().unwrap_or(""));
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
                from_id,
                to_id,
                escape_label(&rel.label),
                escape_label(&rel.protocol)
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

/// Sanitize a string for embedding inside a PlantUML double-quoted argument.
///
/// C4/PlantUML macro arguments are double-quoted and have no escape sequence for
/// an embedded `"`, so a quote in a description or label (e.g. a doc comment
/// reading `e.g. "block"`) would prematurely close the string and corrupt the
/// diagram. Replace any `"` with `'` and flatten newlines to spaces so arbitrary
/// annotation text renders safely.
fn escape_label(s: &str) -> String {
    s.replace('"', "'").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{ArchitectureIR, DirNode};
    use archidoc_types::C4Level;

    #[test]
    fn escape_label_neutralizes_quotes_and_newlines() {
        assert_eq!(escape_label(r#"e.g. "block", "doc""#), "e.g. 'block', 'doc'");
        assert_eq!(escape_label("line1\nline2"), "line1 line2");
        assert_eq!(escape_label("plain"), "plain");
    }

    #[test]
    fn component_description_with_quotes_does_not_break_the_string() {
        let mut node = DirNode::empty("api", "api");
        node.c4_level = Some(C4Level::Component);
        node.description = Some(r#"Typed name (e.g. "block")"#.to_string());
        let mut ir = ArchitectureIR::new("/scan".to_string());
        ir.root.dirs = vec![node];

        let dir = std::env::temp_dir().join("archidoc_escape_test");
        std::fs::create_dir_all(&dir).unwrap();
        generate_component(&dir, &ir);
        let out = std::fs::read_to_string(dir.join("c4-component.puml")).unwrap();

        // The raw double-quote must not survive into the rendered argument.
        assert!(out.contains(r#"Component(api, "api", "--", "Typed name (e.g. 'block')")"#));
        assert!(!out.contains(r#"(e.g. "block")"#));
    }
}
