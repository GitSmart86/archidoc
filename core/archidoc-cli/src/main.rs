use std::fs;
use std::io::Read;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "archidoc")]
#[command(about = "Architecture documentation compiler", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to project root (defaults to current directory)
    path: Option<PathBuf>,

    #[command(flatten)]
    global: GlobalOpts,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args)]
struct GlobalOpts {
    /// Output path for generated ARCHITECTURE.md
    #[arg(short, long, default_value = "ARCHITECTURE.md")]
    output: PathBuf,

    /// Suppress informational output (only errors and requested output)
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Show verbose output with extra processing details
    #[arg(short, long)]
    verbose: bool,

    /// Output machine-readable JSON (for --health, --validate, --check)
    #[arg(long)]
    json: bool,

    /// Check for documentation drift (exit 1 if stale)
    #[arg(long)]
    check: bool,

    /// Print architecture health report
    #[arg(long)]
    health: bool,

    /// Validate file tables against filesystem
    #[arg(long)]
    validate: bool,

    /// Output JSON IR to stdout
    #[arg(long)]
    emit_ir: bool,

    /// Also generate PlantUML diagram files
    #[arg(long)]
    plantuml: bool,

    /// Also generate draw.io CSV files
    #[arg(long)]
    drawio: bool,

    /// Read JSON IR from stdin and generate docs
    #[arg(long, conflicts_with = "from_json_file")]
    from_json: bool,

    /// Read JSON IR from file(s) and generate docs
    #[arg(long, conflicts_with = "from_json")]
    from_json_file: Vec<PathBuf>,

    /// Validate JSON IR (from stdin or --from-json-file)
    #[arg(long)]
    validate_ir: bool,

    /// Merge multiple IR files (use with multiple --from-json-file; requires --merge-ir)
    #[arg(long)]
    merge_ir: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new language adapter scaffold
    InitAdapter {
        /// Language name for the adapter (e.g., python, go, java)
        #[arg(long)]
        lang: String,
    },
    /// Generate annotation template for a directory
    Suggest {
        /// Path to directory to generate annotation for
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::InitAdapter { lang } => {
                run_init_adapter(&lang);
                return;
            }
            Commands::Suggest { path } => {
                run_suggest(&path);
                return;
            }
        }
    }

    // Determine mode from flags
    let mode = if cli.global.validate_ir {
        Mode::ValidateIr
    } else if cli.global.from_json {
        Mode::FromJsonStdin
    } else if !cli.global.from_json_file.is_empty() {
        if cli.global.merge_ir {
            Mode::MergeIr
        } else {
            Mode::FromJsonFile
        }
    } else if cli.global.check {
        Mode::Check
    } else if cli.global.health {
        Mode::Health
    } else if cli.global.validate {
        Mode::Validate
    } else if cli.global.emit_ir {
        Mode::EmitIr
    } else {
        Mode::Generate
    };

    let verbosity = if cli.global.quiet {
        Verbosity::Quiet
    } else if cli.global.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    // Execute mode
    match mode {
        Mode::FromJsonStdin => {
            let docs = read_ir_from_stdin();
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
            run_generate(&root, &docs, &cli.global, verbosity);
        }
        Mode::FromJsonFile => {
            let path = &cli.global.from_json_file[0];
            let docs = read_ir_from_file(path);
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
            run_generate(&root, &docs, &cli.global, verbosity);
        }
        Mode::MergeIr => {
            if cli.global.from_json_file.len() < 2 {
                eprintln!("error: --merge-ir requires at least 2 --from-json-file arguments");
                std::process::exit(1);
            }
            let ir_sets: Vec<Vec<archidoc_types::ModuleDoc>> = cli
                .global
                .from_json_file
                .iter()
                .map(|p| read_ir_from_file(p))
                .collect();
            let docs = archidoc_engine::merge::merge_ir(ir_sets).unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1);
            });
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
            run_generate(&root, &docs, &cli.global, verbosity);
        }
        Mode::ValidateIr => {
            let json = if !cli.global.from_json_file.is_empty() {
                let path = &cli.global.from_json_file[0];
                fs::read_to_string(path).unwrap_or_else(|e| {
                    eprintln!("error: failed to read {}: {}", path.display(), e);
                    std::process::exit(1);
                })
            } else {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .expect("failed to read from stdin");
                buf
            };
            run_validate_ir(&json);
        }
        _ => {
            // Modes that parse from source need a root directory
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

            if !root.exists() {
                eprintln!("error: path does not exist: {}", root.display());
                std::process::exit(1);
            }

            let docs = archidoc_rust::walker::extract_all_docs(&root);

            match mode {
                Mode::Generate => run_generate(&root, &docs, &cli.global, verbosity),
                Mode::Check => run_check(&root, &docs, &cli.global.output, cli.global.json),
                Mode::Health => run_health(&docs, cli.global.json),
                Mode::Validate => run_validate(&docs, cli.global.json),
                Mode::EmitIr => run_emit_ir(&docs),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Generate,
    Check,
    Health,
    Validate,
    EmitIr,
    FromJsonStdin,
    FromJsonFile,
    MergeIr,
    ValidateIr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

fn run_generate(
    root: &PathBuf,
    docs: &[archidoc_types::ModuleDoc],
    opts: &GlobalOpts,
    verbosity: Verbosity,
) {
    if verbosity != Verbosity::Quiet {
        println!("archidoc: {} modules", docs.len());
    }

    if docs.is_empty() {
        if verbosity != Verbosity::Quiet {
            println!("  no annotated modules found");
            println!();
            println!("To get started:");
            println!("  1. Add @c4 annotations to your module entry files (mod.rs, index.ts)");
            println!("  2. Run `archidoc suggest <dir>` to generate a template for a directory");
            println!("  3. See https://github.com/archidoc/archidoc#getting-started");
        }
        return;
    }

    // Generate single ARCHITECTURE.md
    let content = archidoc_engine::architecture::generate(docs, root);
    let output_path = if opts.output.is_absolute() {
        opts.output.clone()
    } else {
        root.join(&opts.output)
    };

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("failed to create output directory");
        }
    }
    fs::write(&output_path, &content).unwrap_or_else(|e| {
        eprintln!("error: failed to write {}: {}", output_path.display(), e);
        std::process::exit(1);
    });

    if verbosity != Verbosity::Quiet {
        println!("wrote {}", output_path.display());
    }

    // Optional sidecar outputs
    if opts.plantuml || opts.drawio {
        let sidecar_dir = output_path.parent().unwrap_or(root);
        let c4_dir = sidecar_dir.join("c4");
        fs::create_dir_all(&c4_dir).expect("failed to create c4 dir");

        if opts.plantuml {
            archidoc_engine::plantuml::generate_container(&c4_dir, docs);
            archidoc_engine::plantuml::generate_component(&c4_dir, docs);
            if verbosity == Verbosity::Verbose {
                println!("wrote PlantUML files to {}", c4_dir.display());
            }
        }

        if opts.drawio {
            let drawio_dir = sidecar_dir.join("drawio");
            fs::create_dir_all(&drawio_dir).expect("failed to create drawio dir");
            archidoc_engine::drawio::generate_container_csv(&drawio_dir, docs);
            archidoc_engine::drawio::generate_component_csv(&drawio_dir, docs);
            if verbosity == Verbosity::Verbose {
                println!("wrote draw.io CSV files to {}", drawio_dir.display());
            }
        }
    }
}

fn run_check(root: &PathBuf, docs: &[archidoc_types::ModuleDoc], output_path: &PathBuf, json: bool) {
    let arch_file = if output_path.is_absolute() {
        output_path.clone()
    } else {
        root.join(output_path)
    };
    let report = archidoc_engine::check::check_drift(docs, &arch_file, root);

    if json {
        let json_output = serde_json::to_string_pretty(&report).expect("failed to serialize report");
        println!("{}", json_output);
    } else {
        let text = archidoc_engine::check::format_drift_report(&report);
        print!("{}", text);
    }

    if report.has_drift() {
        std::process::exit(1);
    }
}

fn run_health(docs: &[archidoc_types::ModuleDoc], json: bool) {
    let report = archidoc_engine::health::aggregate_health(docs);

    if json {
        let json_output = serde_json::to_string_pretty(&report).expect("failed to serialize report");
        println!("{}", json_output);
    } else {
        let text = archidoc_engine::health::format_health_report(&report);
        print!("{}", text);
    }
}

fn run_validate(docs: &[archidoc_types::ModuleDoc], json: bool) {
    let report = archidoc_engine::validate::validate_file_tables(docs);

    if json {
        let json_output = serde_json::to_string_pretty(&report).expect("failed to serialize report");
        println!("{}", json_output);
    } else {
        let text = archidoc_engine::validate::format_validation_report(&report);
        print!("{}", text);
    }

    if !report.is_clean() {
        std::process::exit(1);
    }
}

fn run_emit_ir(docs: &[archidoc_types::ModuleDoc]) {
    let json = archidoc_engine::ir::serialize(docs);
    println!("{}", json);
}

fn read_ir_from_stdin() -> Vec<archidoc_types::ModuleDoc> {
    let mut json = String::new();
    std::io::stdin()
        .read_to_string(&mut json)
        .expect("failed to read JSON IR from stdin");
    archidoc_engine::ir::deserialize(&json).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    })
}

fn read_ir_from_file(path: &PathBuf) -> Vec<archidoc_types::ModuleDoc> {
    let json = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: failed to read {}: {}", path.display(), e);
        std::process::exit(1);
    });
    archidoc_engine::ir::deserialize(&json).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    })
}

fn run_validate_ir(json: &str) {
    match archidoc_engine::ir::validate(json) {
        Ok(()) => println!("IR is valid."),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn run_suggest(path: &PathBuf) {
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        std::process::exit(1);
    }
    if !path.is_dir() {
        eprintln!("error: path is not a directory: {}", path.display());
        std::process::exit(1);
    }
    let annotation = archidoc_engine::suggest::suggest_annotation(path);
    print!("{}", annotation);
}

fn run_init_adapter(lang: &str) {
    println!("Creating adapter scaffold for '{}'...", lang);

    let adapter_dir = PathBuf::from("adapters").join(format!("archidoc-{}", lang));

    if adapter_dir.exists() {
        eprintln!("error: directory already exists: {}", adapter_dir.display());
        std::process::exit(1);
    }

    fs::create_dir_all(adapter_dir.join("src"))
        .expect("failed to create adapter directory structure");

    let cargo_toml = format!(
        r#"[package]
name = "archidoc-{lang}"
version = "0.1.0"
edition = "2021"
description = "Language adapter for {lang} — extracts archidoc annotations"
license = "MIT"
repository = "https://github.com/archidoc/archidoc"
keywords = ["c4-model", "architecture", "documentation", "{lang}"]
categories = ["development-tools"]

[dependencies]
archidoc-types = {{ version = "0.1.0", path = "../../core/archidoc-types" }}
"#
    );

    let lib_rs = format!(
        r#"//! # Archidoc {lang} Adapter
//!
//! Language adapter for extracting archidoc annotations from {lang} source code.
//!
//! ## TODO: Implementation Guide
//!
//! 1. **Parser** (parser.rs): Parse {lang} source files and extract annotation comments
//!    - Identify archidoc annotation format in {lang} (e.g., docstrings, decorators, comments)
//!    - Extract module_path, c4_level, purpose, pattern, etc.
//!    - Parse file tables, relationships, and other metadata
//!
//! 2. **Walker** (walker.rs): Traverse source tree and aggregate ModuleDoc
//!    - Recursively walk {lang} source directories
//!    - Call parser on each relevant file
//!    - Aggregate results into Vec<ModuleDoc>
//!
//! 3. **Integration**: Export `extract_all_docs` function for CLI usage
//!
//! See archidoc-rust adapter for a reference implementation.

pub mod parser;
pub mod walker;

pub use walker::extract_all_docs;
"#
    );

    let parser_rs = format!(
        r#"use archidoc_types::ModuleDoc;

/// Parse archidoc annotations from a {lang} source file.
///
/// TODO: Implement parser for {lang} annotation format
///
/// Returns Some(ModuleDoc) if annotations are found, None otherwise.
pub fn parse_file(path: &std::path::Path) -> Option<ModuleDoc> {{
    // TODO: Read file content
    // TODO: Parse annotations based on {lang} comment/docstring conventions
    // TODO: Extract module_path, c4_level, purpose, pattern, files, relationships
    // TODO: Return ModuleDoc

    None
}}
"#
    );

    let walker_rs = format!(
        r#"use std::path::Path;
use archidoc_types::ModuleDoc;

use crate::parser;

/// Extract all archidoc annotations from a {lang} source tree.
///
/// TODO: Implement directory traversal for {lang} projects
///
/// Recursively walks the source tree and parses each file.
pub fn extract_all_docs(root: &Path) -> Vec<ModuleDoc> {{
    let mut docs = Vec::new();

    // TODO: Implement recursive directory walk
    // TODO: Filter for {lang} source files
    // TODO: Call parser::parse_file on each file
    // TODO: Collect non-None results into docs vector

    docs.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    docs
}}
"#
    );

    fs::write(adapter_dir.join("Cargo.toml"), cargo_toml).expect("failed to write Cargo.toml");
    fs::write(adapter_dir.join("src").join("lib.rs"), lib_rs).expect("failed to write lib.rs");
    fs::write(adapter_dir.join("src").join("parser.rs"), parser_rs)
        .expect("failed to write parser.rs");
    fs::write(adapter_dir.join("src").join("walker.rs"), walker_rs)
        .expect("failed to write walker.rs");

    println!("Created adapter scaffold at: {}", adapter_dir.display());
    println!("\nNext steps:");
    println!("  1. cd {}", adapter_dir.display());
    println!("  2. Implement parser.rs to extract {} annotations", lang);
    println!("  3. Implement walker.rs to traverse {} source trees", lang);
    println!("  4. Test with: cargo test");
    println!("  5. Integrate with CLI by adding dependency in archidoc-cli/Cargo.toml");
}
