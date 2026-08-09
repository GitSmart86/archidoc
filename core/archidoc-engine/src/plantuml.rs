use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::C4Level;

/// Generate PlantUML C4 container diagram.
pub fn generate_container(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-container.puml");
    fs::write(&filepath, container_diagram(ir)).expect("Failed to write c4-container.puml");
}

/// Build the PlantUML C4 container diagram body.
///
/// When any container declares an `@c4 layer`, the containers are grouped into
/// nested `Container_Boundary` blocks (one per layer, falling back to the
/// directory-derived parent, then "Other") inside the `System` boundary —
/// mirroring [`generate_component`]. With no layers declared the System holds a
/// flat container list, exactly as before.
fn container_diagram(ir: &ArchitectureIR) -> String {
    let all_annotated = ir.annotated_dirs();
    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    let container_line = |dir: &DirNode| {
        format!(
            "Container({}, \"{}\", \"{}\", \"{}\")",
            to_puml_id(&dir.path),
            to_title_case(&dir.name),
            dir.pattern.as_deref().unwrap_or("--"),
            dir.description.as_deref().unwrap_or("")
        )
    };

    let mut container_defs = String::new();
    if containers.iter().any(|d| d.layer.is_some()) {
        let mut grouped: BTreeMap<String, Vec<&DirNode>> = BTreeMap::new();
        for dir in &containers {
            let group = dir
                .layer
                .clone()
                .or_else(|| dir.parent.clone())
                .unwrap_or_else(|| "Other".to_string());
            grouped.entry(group).or_default().push(dir);
        }
        for (layer, layer_dirs) in &grouped {
            let layer_id = to_puml_id(layer);
            let layer_name = to_title_case(layer.split('/').last().unwrap_or(layer));
            container_defs.push_str(&format!(
                "    Container_Boundary({}_layer, \"{}\") {{\n",
                layer_id, layer_name
            ));
            for dir in layer_dirs {
                container_defs.push_str(&format!("        {}\n", container_line(dir)));
            }
            container_defs.push_str("    }\n");
        }
    } else {
        for dir in &containers {
            container_defs.push_str(&format!("    {}\n", container_line(dir)));
        }
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

    format!(
        r#"@startuml c4-container
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml

title Container Diagram

System_Boundary(sys, "System") {{
{}}}

{}
@enduml
"#,
        container_defs, rel_defs
    )
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

    // Group by explicit `@c4 layer` when set, else by directory-derived parent.
    let mut grouped: BTreeMap<String, Vec<&DirNode>> = BTreeMap::new();
    for dir in &components {
        let group = dir
            .layer
            .clone()
            .or_else(|| dir.parent.clone())
            .unwrap_or_else(|| "other".to_string());
        grouped.entry(group).or_default().push(dir);
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
    use archidoc_types::ir::{ArchitectureIR, DirNode};
    use archidoc_types::C4Level;

    fn container(name: &str, layer: Option<&str>) -> DirNode {
        let mut d = DirNode::empty(name, name);
        d.c4_level = Some(C4Level::Container);
        d.layer = layer.map(|s| s.to_string());
        d
    }

    fn ir_with(containers: Vec<DirNode>) -> ArchitectureIR {
        let mut ir = ArchitectureIR::new("/scan".to_string());
        ir.root.dirs = containers;
        ir
    }

    #[test]
    fn container_diagram_groups_by_layer() {
        let ir = ir_with(vec![
            container("gpui", Some("UI")),
            container("tui", Some("UI")),
            container("mcp", Some("Services")),
        ]);
        let out = container_diagram(&ir);

        // One nested boundary per declared layer, inside the System boundary.
        assert!(out.contains("System_Boundary(sys, \"System\")"));
        assert!(out.contains("Container_Boundary(UI_layer, \"UI\") {"));
        assert!(out.contains("Container_Boundary(Services_layer, \"Services\") {"));
        // Containers land inside their layer boundary.
        assert!(out.contains("Container(gpui, \"Gpui\""));
        assert!(out.contains("Container(tui, \"Tui\""));
        assert!(out.contains("Container(mcp, \"Mcp\""));
    }

    #[test]
    fn container_diagram_stays_flat_without_layers() {
        let ir = ir_with(vec![container("gpui", None), container("tui", None)]);
        let out = container_diagram(&ir);

        // No layer declared → no nested boundary, flat list as before.
        assert!(!out.contains("Container_Boundary"));
        assert!(out.contains("System_Boundary(sys, \"System\") {"));
        assert!(out.contains("    Container(gpui, \"Gpui\""));
    }
}
