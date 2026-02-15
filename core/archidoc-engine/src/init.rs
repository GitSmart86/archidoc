use std::path::Path;

/// Supported comment styles for different languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// Rust doc comments: `//!`
    Rust,
    /// TypeScript/JavaScript JSDoc: `/** ... */`
    TypeScript,
}

impl CommentStyle {
    /// Auto-detect from project root by looking for Cargo.toml / package.json.
    pub fn detect(root: &Path) -> Option<Self> {
        if root.join("Cargo.toml").exists() {
            Some(Self::Rust)
        } else if root.join("package.json").exists() {
            Some(Self::TypeScript)
        } else {
            None
        }
    }

    pub fn from_lang(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "typescript" | "ts" | "javascript" | "js" => Some(Self::TypeScript),
            _ => None,
        }
    }
}

/// Generate a root-level annotation template for a project's entry file.
///
/// Outputs a doc comment block with recommended architectural sections,
/// each with TODO placeholders. Designed to be pasted into `lib.rs`, `index.ts`, etc.
pub fn generate_template(style: CommentStyle) -> String {
    let sections = vec![
        Section::heading("@c4 container"),
        Section::heading("[Project Name]").h1(),
        Section::blank(),
        Section::line("[TODO: One-line description — what this system does and why it exists.]"),
        Section::blank(),
        Section::heading("C4 Context").h2(),
        Section::blank(),
        Section::code_block(
            "mermaid",
            &[
                "C4Context",
                "    title System Context Diagram",
                "",
                "    Person(user, \"TODO: User\", \"TODO: Primary user/actor\")",
                "    System(system, \"TODO: System Name\", \"TODO: System purpose\")",
                "    System_Ext(ext1, \"TODO: External System\", \"TODO: External dependency\")",
                "",
                "    Rel(user, system, \"Uses\")",
                "    Rel(system, ext1, \"TODO: relationship\", \"TODO: protocol\")",
                "",
                "    UpdateLayoutConfig($c4ShapeInRow=\"3\", $c4BoundaryInRow=\"1\")",
            ],
        ),
        Section::blank(),
        Section::heading("Data Flow").h2(),
        Section::blank(),
        Section::line("1. TODO: Primary command/request flow (e.g., Frontend -> API -> Service -> DB)"),
        Section::line("2. TODO: Primary data/response flow (e.g., DB -> Service -> Frontend)"),
        Section::line("3. TODO: Secondary flows (settings, config, async jobs, etc.)"),
        Section::blank(),
        Section::heading("Concurrency & Data Patterns").h2(),
        Section::blank(),
        Section::line("- TODO: Key concurrency primitives (locks, channels, atomics, async, etc.)"),
        Section::line("- TODO: Data access patterns (caching, buffering, connection pooling, etc.)"),
        Section::blank(),
        Section::heading("Deployment").h2(),
        Section::blank(),
        Section::line("- TODO: Where does this run? (local, cloud, hybrid, embedded)"),
        Section::line("- TODO: Key infrastructure (Docker, K8s, serverless, etc.)"),
        Section::blank(),
        Section::heading("External Dependencies").h2(),
        Section::blank(),
        Section::line("- TODO: Third-party APIs and services"),
        Section::line("- TODO: Databases and storage systems"),
    ];

    render(style, &sections)
}

// -- Internal rendering helpers --

enum Section {
    Line(String),
    Blank,
    Heading { text: String, level: u8 },
    CodeBlock { lang: String, lines: Vec<String> },
}

impl Section {
    fn heading(text: &str) -> Self {
        Self::Heading {
            text: text.to_string(),
            level: 0,
        }
    }

    fn h1(self) -> Self {
        match self {
            Self::Heading { text, .. } => Self::Heading { text, level: 1 },
            other => other,
        }
    }

    fn h2(self) -> Self {
        match self {
            Self::Heading { text, .. } => Self::Heading { text, level: 2 },
            other => other,
        }
    }

    fn line(text: &str) -> Self {
        Self::Line(text.to_string())
    }

    fn blank() -> Self {
        Self::Blank
    }

    fn code_block(lang: &str, lines: &[&str]) -> Self {
        Self::CodeBlock {
            lang: lang.to_string(),
            lines: lines.iter().map(|s| s.to_string()).collect(),
        }
    }
}

fn render(style: CommentStyle, sections: &[Section]) -> String {
    let mut out = String::new();

    for section in sections {
        match section {
            Section::Blank => {
                out.push_str(&comment_line(style, ""));
                out.push('\n');
            }
            Section::Line(text) => {
                out.push_str(&comment_line(style, text));
                out.push('\n');
            }
            Section::Heading { text, level } => {
                let prefix = match level {
                    1 => "# ",
                    2 => "## ",
                    3 => "### ",
                    _ => "",
                };
                out.push_str(&comment_line(style, &format!("{}{}", prefix, text)));
                out.push('\n');
            }
            Section::CodeBlock { lang, lines } => {
                out.push_str(&comment_line(style, &format!("```{}", lang)));
                out.push('\n');
                for line in lines {
                    out.push_str(&comment_line(style, line));
                    out.push('\n');
                }
                out.push_str(&comment_line(style, "```"));
                out.push('\n');
            }
        }
    }

    out
}

fn comment_line(style: CommentStyle, text: &str) -> String {
    match style {
        CommentStyle::Rust => {
            if text.is_empty() {
                "//!".to_string()
            } else {
                format!("//! {}", text)
            }
        }
        CommentStyle::TypeScript => {
            if text.is_empty() {
                " *".to_string()
            } else {
                format!(" * {}", text)
            }
        }
    }
}

/// Wrap TypeScript output in JSDoc delimiters.
pub fn wrap_jsdoc(content: &str) -> String {
    format!("/**\n{} */\n", content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_template_has_c4_marker() {
        let out = generate_template(CommentStyle::Rust);
        assert!(out.contains("//! @c4 container"));
    }

    #[test]
    fn rust_template_has_all_sections() {
        let out = generate_template(CommentStyle::Rust);
        assert!(out.contains("# [Project Name]"));
        assert!(out.contains("## C4 Context"));
        assert!(out.contains("## Data Flow"));
        assert!(out.contains("## Concurrency & Data Patterns"));
        assert!(out.contains("## Deployment"));
        assert!(out.contains("## External Dependencies"));
    }

    #[test]
    fn rust_template_has_mermaid_block() {
        let out = generate_template(CommentStyle::Rust);
        assert!(out.contains("```mermaid"));
        assert!(out.contains("C4Context"));
        assert!(out.contains("```"));
    }

    #[test]
    fn rust_template_has_todo_placeholders() {
        let out = generate_template(CommentStyle::Rust);
        assert!(out.contains("TODO:"));
        assert!(out.contains("[TODO: One-line description"));
    }

    #[test]
    fn typescript_template_uses_jsdoc_style() {
        let out = generate_template(CommentStyle::TypeScript);
        assert!(out.contains(" * @c4 container"));
        assert!(out.contains(" * ## Data Flow"));
    }

    #[test]
    fn detect_rust_from_cargo_toml() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(CommentStyle::detect(tmp.path()), Some(CommentStyle::Rust));
    }

    #[test]
    fn detect_ts_from_package_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("package.json"), "").unwrap();
        assert_eq!(
            CommentStyle::detect(tmp.path()),
            Some(CommentStyle::TypeScript)
        );
    }

    #[test]
    fn from_lang_parsing() {
        assert_eq!(CommentStyle::from_lang("rust"), Some(CommentStyle::Rust));
        assert_eq!(CommentStyle::from_lang("rs"), Some(CommentStyle::Rust));
        assert_eq!(CommentStyle::from_lang("ts"), Some(CommentStyle::TypeScript));
        assert_eq!(CommentStyle::from_lang("unknown"), None);
    }

    #[test]
    fn blank_lines_are_comment_only() {
        let out = generate_template(CommentStyle::Rust);
        // Blank lines should be "//!" with no trailing space
        assert!(out.contains("\n//!\n"));
        assert!(!out.contains("//! \n"));
    }
}
