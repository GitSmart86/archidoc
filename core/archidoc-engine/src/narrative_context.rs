//! Discovery and parsing of `.archidoc/narrative-context.md`.
//!
//! The narrative context source file provides strategic project context
//! (problem statement, architecture rationale, design invariants, etc.)
//! that is merged with IR data at render time to produce `ai-strategy.md`.
//!
//! The file uses YAML frontmatter for structured metadata and markdown
//! body sections delimited by H2 headers.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Name of the source file inside `.archidoc/`.
const SOURCE_FILENAME: &str = "narrative-context.md";

/// Name of the config directory.
const CONFIG_DIR: &str = ".archidoc";

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// Parsed representation of `.archidoc/narrative-context.md`.
#[derive(Debug, Clone)]
pub struct NarrativeContext {
    /// Parsed YAML frontmatter metadata.
    pub frontmatter: Frontmatter,
    /// Body sections extracted from H2 headers, in document order.
    pub sections: Vec<Section>,
}

/// YAML frontmatter metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    /// Schema version for forward compatibility.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Project name (used in rendered header).
    #[serde(default)]
    pub project: String,
    /// Last edit date (rendered as freshness signal).
    #[serde(default)]
    pub updated: String,
    /// Optional component status summary.
    #[serde(default)]
    pub components: Vec<ComponentStatus>,
    /// Optional build phases.
    #[serde(default)]
    pub phases: Vec<BuildPhase>,
}

fn default_version() -> u32 {
    1
}

/// Component status entry from frontmatter.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentStatus {
    /// Component identifier (e.g. "01", "02").
    pub id: String,
    /// Component name (e.g. "MAC-Schema").
    pub name: String,
    /// Build status: not_started, partial, in_progress, complete.
    pub status: String,
}

/// Build phase entry from frontmatter.
#[derive(Debug, Clone, Deserialize)]
pub struct BuildPhase {
    /// Phase name (e.g. "Phase 0a").
    pub name: String,
    /// Human-readable label.
    #[serde(default)]
    pub label: String,
    /// Component IDs in this phase.
    #[serde(default)]
    pub components: Vec<String>,
    /// Phase status.
    #[serde(default)]
    pub status: String,
}

/// A body section extracted from an H2 header.
#[derive(Debug, Clone)]
pub struct Section {
    /// The H2 heading text (without the `## ` prefix).
    pub heading: String,
    /// Whether this is a recognized section name.
    pub recognized: bool,
    /// The body content (everything between this H2 and the next).
    pub body: String,
}

/// Known section names that the renderer can reorder or annotate.
const RECOGNIZED_SECTIONS: &[&str] = &[
    "Problem",
    "Architecture",
    "Build Order",
    "Design Invariants",
    "External Systems",
    "Concurrency Model",
    "Testing Conventions",
    "Decisions",
];

// ---------------------------------------------------------------------------
// Parse errors
// ---------------------------------------------------------------------------

/// Errors that can occur during narrative context parsing.
#[derive(Debug)]
pub enum ParseError {
    /// YAML frontmatter is missing or malformed.
    InvalidFrontmatter(String),
    /// File has no `---` frontmatter delimiters.
    NoFrontmatter,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFrontmatter(msg) => {
                write!(f, "invalid YAML frontmatter: {}", msg)
            }
            ParseError::NoFrontmatter => {
                write!(f, "no YAML frontmatter found (missing --- delimiters)")
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Walk up from `scan_root` looking for `.archidoc/narrative-context.md`.
///
/// Returns `Some(path)` if found, `None` if the walk reaches the filesystem
/// root without finding the file.
pub fn discover(scan_root: &Path) -> Option<PathBuf> {
    let mut current = scan_root.to_path_buf();

    // Canonicalize to resolve symlinks and relative paths
    if let Ok(canonical) = current.canonicalize() {
        current = canonical;
    }

    loop {
        let candidate = current.join(CONFIG_DIR).join(SOURCE_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }

        // Move up one directory
        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
            }
            _ => return None,
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse the content of a narrative context source file.
///
/// Expects YAML frontmatter delimited by `---` lines, followed by a
/// markdown body with H2 sections.
pub fn parse(content: &str) -> Result<NarrativeContext, ParseError> {
    let (frontmatter_str, body_str) = split_frontmatter(content)?;

    let frontmatter: Frontmatter = serde_yaml::from_str(&frontmatter_str)
        .map_err(|e| ParseError::InvalidFrontmatter(e.to_string()))?;

    let sections = parse_sections(&body_str);

    Ok(NarrativeContext {
        frontmatter,
        sections,
    })
}

/// Split content into frontmatter YAML and body markdown.
fn split_frontmatter(content: &str) -> Result<(String, String), ParseError> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(ParseError::NoFrontmatter);
    }

    // Find the closing ---
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.trim_start_matches(|c: char| c == '-'); // handle ---- etc.
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);

    if let Some(end_pos) = find_closing_delimiter(after_opening) {
        let frontmatter = after_opening[..end_pos].to_string();
        let body = after_opening[end_pos..]
            .trim_start_matches('-')
            .trim_start_matches('\n')
            .to_string();
        Ok((frontmatter, body))
    } else {
        Err(ParseError::NoFrontmatter)
    }
}

/// Find the position of the closing `---` delimiter.
fn find_closing_delimiter(content: &str) -> Option<usize> {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "---\r" {
            // Calculate byte offset
            let offset: usize = content.lines()
                .take(i)
                .map(|l| l.len() + 1) // +1 for newline
                .sum();
            return Some(offset);
        }
    }
    None
}

/// Parse the body into H2 sections.
fn parse_sections(body: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        if line.starts_with("## ") {
            // Flush the previous section
            if let Some(heading) = current_heading.take() {
                let body_text = collapse_body(&current_lines);
                let recognized = RECOGNIZED_SECTIONS
                    .iter()
                    .any(|&s| s.eq_ignore_ascii_case(&heading));
                sections.push(Section {
                    heading,
                    recognized,
                    body: body_text,
                });
                current_lines.clear();
            }
            // Start a new section
            let heading = line[3..].trim().to_string();
            current_heading = Some(heading);
        } else if current_heading.is_some() {
            current_lines.push(line);
        }
        // Lines before the first H2 are ignored (preamble)
    }

    // Flush final section
    if let Some(heading) = current_heading.take() {
        let body_text = collapse_body(&current_lines);
        let recognized = RECOGNIZED_SECTIONS
            .iter()
            .any(|&s| s.eq_ignore_ascii_case(&heading));
        sections.push(Section {
            heading,
            recognized,
            body: body_text,
        });
    }

    sections
}

/// Join lines and trim trailing whitespace, but preserve internal structure.
fn collapse_body(lines: &[&str]) -> String {
    let text = lines.join("\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_frontmatter() {
        let content = r#"---
version: 1
project: "Test Project"
updated: "2026-05-13"
---

## Problem

This is the problem statement.

## Architecture

The system uses a modular design.
"#;

        let ctx = parse(content).unwrap();
        assert_eq!(ctx.frontmatter.version, 1);
        assert_eq!(ctx.frontmatter.project, "Test Project");
        assert_eq!(ctx.frontmatter.updated, "2026-05-13");
        assert!(ctx.frontmatter.components.is_empty());
        assert!(ctx.frontmatter.phases.is_empty());
        assert_eq!(ctx.sections.len(), 2);
        assert_eq!(ctx.sections[0].heading, "Problem");
        assert!(ctx.sections[0].recognized);
        assert!(ctx.sections[0].body.contains("problem statement"));
        assert_eq!(ctx.sections[1].heading, "Architecture");
        assert!(ctx.sections[1].recognized);
    }

    #[test]
    fn parse_with_components_and_phases() {
        let content = r#"---
version: 1
project: "S3 Data Manager"
updated: "2026-05-13"
components:
  - id: "01"
    name: "MAC-Schema"
    status: "complete"
  - id: "02"
    name: "MAC-Board"
    status: "in_progress"
phases:
  - name: "Phase 0a"
    label: "Schema Discovery"
    components: ["01"]
    status: "complete"
---

## Problem

The problem.
"#;

        let ctx = parse(content).unwrap();
        assert_eq!(ctx.frontmatter.components.len(), 2);
        assert_eq!(ctx.frontmatter.components[0].id, "01");
        assert_eq!(ctx.frontmatter.components[0].status, "complete");
        assert_eq!(ctx.frontmatter.phases.len(), 1);
        assert_eq!(ctx.frontmatter.phases[0].name, "Phase 0a");
        assert_eq!(ctx.frontmatter.phases[0].components, vec!["01"]);
    }

    #[test]
    fn unrecognized_sections_pass_through() {
        let content = r#"---
version: 1
project: "Test"
updated: "2026-05-13"
---

## Problem

Known section.

## Custom Section

This is a custom section that should pass through.

## Architecture

Another known section.
"#;

        let ctx = parse(content).unwrap();
        assert_eq!(ctx.sections.len(), 3);
        assert!(ctx.sections[0].recognized); // Problem
        assert!(!ctx.sections[1].recognized); // Custom Section
        assert_eq!(ctx.sections[1].heading, "Custom Section");
        assert!(ctx.sections[2].recognized); // Architecture
    }

    #[test]
    fn no_frontmatter_returns_error() {
        let content = "## Problem\n\nNo frontmatter here.\n";
        assert!(parse(content).is_err());
    }

    #[test]
    fn empty_body_sections() {
        let content = r#"---
version: 1
project: "Test"
updated: "2026-05-13"
---
"#;

        let ctx = parse(content).unwrap();
        assert!(ctx.sections.is_empty());
    }

    #[test]
    fn defaults_applied_for_missing_fields() {
        let content = r#"---
project: "Minimal"
---

## Problem

Just a problem.
"#;

        let ctx = parse(content).unwrap();
        assert_eq!(ctx.frontmatter.version, 1); // default
        assert_eq!(ctx.frontmatter.updated, ""); // default empty
    }

    #[test]
    fn discover_returns_none_for_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(discover(tmp.path()).is_none());
    }

    #[test]
    fn discover_finds_in_current_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let archidoc_dir = tmp.path().join(CONFIG_DIR);
        std::fs::create_dir_all(&archidoc_dir).unwrap();
        let source_file = archidoc_dir.join(SOURCE_FILENAME);
        std::fs::write(&source_file, "---\nproject: test\n---\n").unwrap();

        let found = discover(tmp.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), SOURCE_FILENAME);
    }

    #[test]
    fn discover_walks_up() {
        let tmp = tempfile::tempdir().unwrap();
        // Put the file at the root
        let archidoc_dir = tmp.path().join(CONFIG_DIR);
        std::fs::create_dir_all(&archidoc_dir).unwrap();
        let source_file = archidoc_dir.join(SOURCE_FILENAME);
        std::fs::write(&source_file, "---\nproject: test\n---\n").unwrap();

        // Search from a subdirectory
        let sub = tmp.path().join("packages").join("api");
        std::fs::create_dir_all(&sub).unwrap();

        let found = discover(&sub);
        assert!(found.is_some());
    }
}
