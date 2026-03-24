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
///
/// Uses `@c4 container` / `@c4 component` syntax.
pub fn extract_c4_level(content: &str) -> C4Level {
    if content.contains("@c4 container") {
        C4Level::Container
    } else if content.contains("@c4 component") {
        C4Level::Component
    } else {
        C4Level::Unknown
    }
}

/// Extract the primary GoF pattern name from doc content.
///
/// Priority order:
/// 1. Explicit `Pattern:` line in the annotation (structured, reliable)
/// 2. Substring search for known pattern names in content (heuristic fallback)
/// 3. "--" if nothing found
pub fn extract_pattern(content: &str) -> String {
    // Priority 1: explicit "Pattern:" annotation line
    if let Some(pattern) = extract_explicit_pattern(content) {
        return pattern;
    }

    // Priority 2: heuristic substring search on description lines only
    // (excludes file table rows and @c4 markers to avoid false positives)
    // Order matters — longer/more specific patterns first to avoid
    // partial matches (e.g. "Chain of Responsibility" before "Command")
    let description_text: String = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with('|') && !t.starts_with("@c4 ") && !t.starts_with("Pattern:")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let patterns = [
        "Chain of Responsibility",
        "Active Object",
        "Template Method",
        "Abstract Factory",
        "Value Object",
        "Mediator",
        "Observer",
        "Strategy",
        "Facade",
        "Adapter",
        "Repository",
        "Singleton",
        "Factory",
        "Builder",
        "Decorator",
        "Memento",
        "Command",
        "Iterator",
        "Composite",
        "Interpreter",
        "Flyweight",
        "Publisher",
        "Prototype",
        "Bridge",
        "Proxy",
        "State",
        "Visitor",
        "Core",
    ];

    for name in patterns {
        if description_text.contains(name) {
            return name.to_string();
        }
    }

    "--".to_string()
}

/// Extract an explicit "Pattern: X" or "Pattern: X (verified)" line from doc content.
///
/// Looks for a line starting with "Pattern:" and extracts the pattern name.
/// Handles optional status suffix like "(verified)" or "(planned)".
fn extract_explicit_pattern(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Pattern:") {
            let rest = rest.trim();
            if rest.is_empty() || rest == "--" {
                continue;
            }
            // Strip optional status suffix: "Facade (verified)" -> "Facade"
            let pattern = if let Some(idx) = rest.find('(') {
                rest[..idx].trim()
            } else {
                rest
            };
            if !pattern.is_empty() {
                return Some(pattern.to_string());
            }
        }
    }
    None
}

/// Extract pattern status from doc content.
///
/// Checks for status on the explicit `Pattern:` line first (e.g. "Pattern: Facade (verified)"),
/// then falls back to searching for "(verified)" anywhere in content. Defaults to Planned.
pub fn extract_pattern_status(content: &str) -> PatternStatus {
    // Priority 1: status on the "Pattern:" line
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Pattern:") {
            let rest = rest.trim();
            if let Some(idx) = rest.find('(') {
                let status_str = rest[idx + 1..]
                    .trim_end_matches(')')
                    .trim();
                return PatternStatus::parse(status_str);
            }
        }
    }

    // Priority 2: "(verified)" anywhere in content (legacy fallback)
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
                && !trimmed.starts_with("@c4 ")
                && !trimmed.starts_with('|')
                && !trimmed.starts_with("GoF:")
                && !trimmed.starts_with("Pattern:")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pattern_single_word() {
        let content = "@c4 component\n\nSome description.\n\nPattern: Facade";
        assert_eq!(extract_pattern(content), "Facade");
    }

    #[test]
    fn explicit_pattern_multi_word() {
        let content = "@c4 component\n\nSome description.\n\nPattern: Value Object";
        assert_eq!(extract_pattern(content), "Value Object");
    }

    #[test]
    fn explicit_pattern_chain_of_responsibility() {
        let content = "@c4 component\n\nValidation pipeline.\n\nPattern: Chain of Responsibility";
        assert_eq!(extract_pattern(content), "Chain of Responsibility");
    }

    #[test]
    fn explicit_pattern_with_verified_status() {
        let content = "@c4 component\n\nEntry point.\n\nPattern: Strategy (verified)";
        assert_eq!(extract_pattern(content), "Strategy");
        assert_eq!(extract_pattern_status(content), PatternStatus::Verified);
    }

    #[test]
    fn explicit_pattern_overrides_heuristic() {
        // Description contains "Factory" but Pattern line says "Builder"
        let content = "@c4 component\n\nFactory for creating widgets.\n\nPattern: Builder";
        assert_eq!(extract_pattern(content), "Builder");
    }

    #[test]
    fn heuristic_fallback_when_no_pattern_line() {
        let content = "@c4 container\n\nCentral Facade for the module.";
        assert_eq!(extract_pattern(content), "Facade");
    }

    #[test]
    fn heuristic_ignores_file_table_patterns() {
        // File table contains "Value Object" but description doesn't mention a pattern
        let content = "@c4 container\n\nShared domain types.\n\n| File | Pattern | Purpose | Health |\n|------|---------|---------|--------|\n| `types.rs` | Value Object | Domain types | stable |";
        assert_eq!(extract_pattern(content), "--");
    }

    #[test]
    fn no_false_positive_from_registry_in_description() {
        // "RegistryEntry" in description should not match "Registry"
        // (Registry was removed from the heuristic pattern list)
        let content = "@c4 component\n\nPattern: Value Object\n\nDomain types: RegistryEntry, Primitive.";
        assert_eq!(extract_pattern(content), "Value Object");
    }

    #[test]
    fn description_skips_pattern_line() {
        let content = "@c4 component\n\nPattern: Facade\n\nActual description here.";
        assert_eq!(extract_description(content), "Actual description here.");
    }

    #[test]
    fn pattern_dash_dash_is_skipped() {
        let content = "@c4 component\n\nSome description.\n\nPattern: --";
        assert_eq!(extract_pattern(content), "--");
    }
}

/// Parse `@c4 uses target "label" "protocol"` markers from content.
pub fn extract_relationships(content: &str) -> Vec<Relationship> {
    let mut rels = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("@c4 uses ") {
            // Parse: target "label" "protocol"
            // Split on first quote to get target, then extract quoted strings
            if let Some(quote_start) = rest.find('"') {
                let target = rest[..quote_start].trim().to_string();
                let quoted_part = &rest[quote_start..];
                let quotes: Vec<&str> = quoted_part
                    .split('"')
                    .filter(|s| !s.trim().is_empty())
                    .collect();
                if quotes.len() >= 2 {
                    rels.push(Relationship {
                        target,
                        label: quotes[0].to_string(),
                        protocol: quotes[1].to_string(),
                    });
                }
            }
        }
    }

    rels
}
