use std::fs;
use std::path::Path;

use archidoc_types::{
    C4Level, FileEntry, HealthStatus, PatternStatus, Relationship,
};

/// Extract `//!` doc comments from a Rust source file.
///
/// Returns the joined content of all leading `//!` lines, with prefixes stripped.
pub fn archidoc_from_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;

    let doc_lines: Vec<&str> = content
        .lines()
        .take_while(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//!") || trimmed.is_empty()
        })
        .filter(|line| line.trim().starts_with("//!"))
        .map(|line| {
            let trimmed = line.trim();
            if trimmed == "//!" {
                ""
            } else if let Some(rest) = trimmed.strip_prefix("//! ") {
                rest
            } else {
                trimmed.strip_prefix("//!").unwrap_or("")
            }
        })
        .collect();

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

/// Extract the C4 level marker from doc content.
pub fn extract_c4_level(content: &str) -> C4Level {
    if content.contains("<<container>>") {
        C4Level::Container
    } else if content.contains("<<component>>") {
        C4Level::Component
    } else {
        C4Level::Unknown
    }
}

/// Extract the primary GoF pattern name from doc content.
///
/// Looks for known pattern names in the content. Returns the first match
/// or "--" if none found.
pub fn extract_pattern(content: &str) -> String {
    let patterns = [
        "Mediator",
        "Observer",
        "Strategy",
        "Facade",
        "Adapter",
        "Repository",
        "Singleton",
        "Factory",
        "Active Object",
        "Memento",
        "Command",
        "Chain of Responsibility",
        "Registry",
        "Composite",
        "Interpreter",
        "Flyweight",
        "Publisher",
    ];

    for name in patterns {
        if content.contains(name) {
            return name.to_string();
        }
    }

    "--".to_string()
}

/// Extract pattern status from doc content.
///
/// Looks for "(verified)" near a pattern name. Defaults to Planned.
pub fn extract_pattern_status(content: &str) -> PatternStatus {
    if content.contains("(verified)") {
        PatternStatus::Verified
    } else {
        PatternStatus::Planned
    }
}

/// Extract the first non-header, non-marker line as description.
pub fn extract_description(content: &str) -> String {
    content
        .lines()
        .find(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.contains("<<")
                && !trimmed.starts_with('|')
                && !trimmed.starts_with("GoF:")
        })
        .unwrap_or("*No description*")
        .trim()
        .to_string()
}

/// Extract the parent container from a dot-notation module path.
///
/// "bus.calc.indicators" -> Some("bus")
/// "bus" -> None
pub fn extract_parent_container(module_path: &str) -> Option<String> {
    if module_path.contains('.') {
        Some(
            module_path
                .split('.')
                .next()
                .unwrap_or(module_path)
                .to_string(),
        )
    } else {
        None
    }
}

/// Parse the markdown file table into FileEntry structs.
///
/// Expects format:
/// ```text
/// | File | Pattern | Purpose | Health |
/// |------|---------|---------|--------|
/// | `core.rs` | Facade | Entry point | stable |
/// ```
pub fn extract_file_table(content: &str) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let mut in_table = false;
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_table {
            // Look for table header
            if trimmed.starts_with('|')
                && (trimmed.contains("File") || trimmed.contains("file"))
                && (trimmed.contains("Pattern") || trimmed.contains("pattern"))
            {
                in_table = true;
                continue;
            }
        } else if !header_seen {
            // Skip the separator row (|------|...)
            if trimmed.starts_with('|') && trimmed.contains("---") {
                header_seen = true;
                continue;
            }
        } else {
            // Parse data rows
            if !trimmed.starts_with('|') {
                break; // End of table
            }

            let cells: Vec<&str> = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim())
                .collect();

            if cells.len() >= 4 {
                let filename = cells[0]
                    .trim_matches('`')
                    .trim()
                    .to_string();

                let (pattern, pattern_status) = parse_pattern_field(cells[1]);
                let purpose = cells[2].trim().to_string();
                let health = HealthStatus::parse(cells[3]);

                entries.push(FileEntry {
                    name: filename,
                    pattern,
                    pattern_status,
                    purpose,
                    health,
                });
            }
        }
    }

    entries
}

/// Parse a pattern field like "Strategy (verified)" into (pattern, status).
fn parse_pattern_field(field: &str) -> (String, PatternStatus) {
    let trimmed = field.trim();

    if let Some(idx) = trimmed.find('(') {
        let pattern = trimmed[..idx].trim().to_string();
        let status_str = trimmed[idx + 1..]
            .trim_end_matches(')')
            .trim();
        (pattern, PatternStatus::parse(status_str))
    } else {
        (trimmed.to_string(), PatternStatus::Planned)
    }
}

/// Parse `<<uses: target, "label", "protocol">>` markers from content.
pub fn extract_relationships(content: &str) -> Vec<Relationship> {
    let mut rels = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(inner) = trimmed
            .strip_prefix("<<uses:")
            .and_then(|s| s.strip_suffix(">>"))
        {
            // Parse: target, "label", "protocol"
            let parts: Vec<&str> = inner.splitn(3, ',').collect();
            if parts.len() >= 3 {
                let target = parts[0].trim().to_string();
                let label = parts[1].trim().trim_matches('"').to_string();
                let protocol = parts[2].trim().trim_matches('"').to_string();
                rels.push(Relationship {
                    target,
                    label,
                    protocol,
                });
            }
        }
    }

    rels
}
