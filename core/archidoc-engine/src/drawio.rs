use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::C4Level;

/// Generate draw.io container CSV.
pub fn generate_container_csv(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-container.csv");

    let all_annotated = ir.annotated_dirs();
    let containers: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Container))
        .collect();

    let mut rows = Vec::new();

    for dir in &containers {
        let refs: Vec<String> = dir.relationships.iter().map(|r| r.target.clone()).collect();
        let refs_str = refs.join(",");
        let pattern = dir.pattern.as_deref().unwrap_or("--");
        let desc = dir.description.as_deref().unwrap_or("");

        rows.push(format!(
            "{},{},container,{},{},{}",
            dir.path,
            to_title_case(&dir.name),
            pattern,
            desc,
            refs_str,
        ));
    }

    let content = format!(
        "{}\nid,name,type,pattern,description,refs\n{}",
        csv_header(),
        rows.join("\n")
    );

    fs::write(&filepath, content).expect("Failed to write container CSV");
}

/// Generate draw.io component CSV.
pub fn generate_component_csv(output_dir: &Path, ir: &ArchitectureIR) {
    let filepath = output_dir.join("c4-component.csv");

    let all_annotated = ir.annotated_dirs();
    let components: Vec<&DirNode> = all_annotated
        .iter()
        .copied()
        .filter(|d| d.c4_level == Some(C4Level::Component))
        .collect();

    // Group by parent
    let mut grouped: BTreeMap<String, Vec<&DirNode>> = BTreeMap::new();
    for dir in &components {
        let parent = dir
            .parent
            .clone()
            .unwrap_or_else(|| "other".to_string());
        grouped.entry(parent).or_default().push(dir);
    }

    let mut rows = Vec::new();

    // Add container stubs for grouping
    for parent in grouped.keys() {
        rows.push(format!(
            "{},{},container,,,",
            parent,
            to_title_case(parent.split('/').last().unwrap_or(parent)),
        ));
    }

    // Add components
    for dir in &components {
        let name = &dir.name;
        let parent = dir.parent.as_deref().unwrap_or("");
        let pattern = dir.pattern.as_deref().unwrap_or("--");
        let desc = dir.description.as_deref().unwrap_or("");
        let refs: Vec<String> = dir.relationships.iter().map(|r| r.target.clone()).collect();

        rows.push(format!(
            "{},{},component,{},{},{}",
            dir.path,
            name,
            pattern,
            desc,
            if refs.is_empty() {
                parent.to_string()
            } else {
                refs.join(",")
            },
        ));
    }

    let content = format!(
        "{}\nid,name,type,pattern,description,refs\n{}",
        csv_header(),
        rows.join("\n")
    );

    fs::write(&filepath, content).expect("Failed to write component CSV");
}

fn csv_header() -> &'static str {
    r#"## C4 Diagram
## Import: Arrange > Insert > Advanced > CSV
#
# label: <b>%name%</b><br><font style="font-size:11px;">%description%</font>
# stylename: type
# styles: {"container": "rounded=1;whiteSpace=wrap;fillColor=#438DD5;fontColor=#ffffff;", \
#          "component": "rounded=1;whiteSpace=wrap;fillColor=#85BBF0;fontColor=#000000;"}
# connect: {"from": "refs", "to": "id", "invert": false, "style": "curved=1;exitX=0.5;exitY=1;entryX=0.5;entryY=0;"}
# width: 200
# height: 100
# padding: 30
# ignore: id,refs,type,pattern
# identity: id
# namespace: c4"#
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
