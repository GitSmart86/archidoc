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
    #[arg(short, long, default_value = "_context/ARCHITECTURE.md")]
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

    /// Do not generate ARCHITECTURE.ai.md
    #[arg(long)]
    no_ai: bool,

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
    /// Generate a file from environment context
    ///
    /// Reads the target directory, applies the named handler, and writes output.
    /// Each handler reads project state and produces one or more files.
    ///
    /// Examples:
    ///   archidoc init _index.md ./src/              # generate _index.md for all dirs missing one
    ///   archidoc init _index.md ./src/ --dry-run    # list missing dirs (was: audit)
    ///   archidoc init c4-annotation ./src/api/      # generate @c4 stub (was: suggest)
    ///   archidoc init root-annotation .             # generate root template
    ///   archidoc init tree .                        # generate dir tree (was: tree)
    ///   archidoc init tree . --files --human        # tree variants
    ///   archidoc init --list                        # list available handlers
    Init(InitFileArgs),
    /// Create a project from a folder template
    ///
    /// Copies a named template tree from `.archidoc/templates/scaffold-folder-templates/<name>/`
    /// to the target directory, substituting `{{variable}}` tokens in paths and file contents.
    ///
    /// Templates are discovered by walking up the directory tree — firm-level templates
    /// at a workspace root are available from any subdirectory.
    ///
    /// Examples:
    ///   archidoc scaffold client --target ./clients/Acme --var client_name=Acme --var engagement_id=2026-001
    ///   archidoc scaffold --list
    ///   archidoc scaffold client --dry-run --var client_name=Test
    Scaffold(NewArgs),
}

#[derive(Args)]
struct NewArgs {
    /// Template name (folder name in scaffold-folder-templates/)
    name: Option<String>,

    /// Target directory (defaults to current directory)
    #[arg(long, short = 't')]
    target: Option<PathBuf>,

    /// Variable assignment (repeatable): --var key=value
    #[arg(long = "var", value_parser = parse_key_value)]
    vars: Vec<(String, String)>,

    /// List available folder templates
    #[arg(long, conflicts_with_all = ["name", "inspect", "dry_run"])]
    list: bool,

    /// Show template metadata and required variables
    #[arg(long)]
    inspect: bool,

    /// Show what would be created without writing anything
    #[arg(long)]
    dry_run: bool,

    /// Overwrite existing files
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct InitFileArgs {
    /// Handler name (e.g., _index.md, c4-annotation, root-annotation, tree)
    name: Option<String>,

    /// Target directory (defaults to current directory)
    target: Option<PathBuf>,

    /// List available init handlers
    #[arg(long)]
    list: bool,

    /// Show what would be generated without writing
    #[arg(long)]
    dry_run: bool,

    /// Variable assignment (repeatable): --var key=value
    #[arg(long = "var", value_parser = parse_key_value)]
    vars: Vec<(String, String)>,

    // -- Handler-specific flags (passed through to handlers) --

    /// Include files in tree output (tree handler)
    #[arg(long)]
    files: bool,

    /// Generate human-readable tree with icons (tree handler)
    #[arg(long)]
    human: bool,

    /// Generate both AI dir and dir+files tree variants (tree handler)
    #[arg(long)]
    both: bool,

    /// Maximum traversal depth (tree handler)
    #[arg(long)]
    depth: Option<usize>,

    /// Language for comment syntax: rust, ts (root-annotation handler)
    #[arg(long)]
    lang: Option<String>,

    /// Output directory for generated files (tree handler, defaults to _context/)
    #[arg(long)]
    out: Option<PathBuf>,
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no '=' found in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::Init(args) => {
                run_init_file(args);
                return;
            }
            Commands::Scaffold(args) => {
                run_new(args);
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

    // Load custom templates once from CWD — used by suggest and generate
    let cwd = std::env::current_dir().expect("failed to get current directory");
    let custom = archidoc_engine::custom::CustomTemplates::load(&cwd);
    let table_columns = custom
        .architecture_table
        .as_deref()
        .map(archidoc_engine::custom::parse_table_columns)
        .unwrap_or_default();

    // Execute mode
    match mode {
        Mode::FromJsonStdin => {
            let docs = read_ir_from_stdin();
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
            run_generate(&root, &docs, &cli.global, verbosity, &table_columns);
        }
        Mode::FromJsonFile => {
            let path = &cli.global.from_json_file[0];
            let docs = read_ir_from_file(path);
            let root = cli
                .path
                .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
            run_generate(&root, &docs, &cli.global, verbosity, &table_columns);
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
            run_generate(&root, &docs, &cli.global, verbosity, &table_columns);
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

            let docs = collect_all_docs(&root, verbosity);

            match mode {
                Mode::Generate => run_generate(&root, &docs, &cli.global, verbosity, &table_columns),
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
    columns: &[String],
) {
    // Make root absolute without canonicalize — canonicalize produces UNC paths
    // on Windows (\\?\C:\...) which pathdiff cannot diff against regular paths
    // (C:\...) emitted by Node's path.resolve(). Just join with CWD instead.
    let root = &if root.is_absolute() {
        root.clone()
    } else {
        std::env::current_dir()
            .expect("failed to get current directory")
            .join(root)
    };
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
    let output_path = if opts.output.is_absolute() {
        opts.output.clone()
    } else {
        root.join(&opts.output)
    };
    let link_base = output_path.parent().unwrap_or(root.as_path());
    let content = archidoc_engine::architecture::generate(docs, link_base, columns);

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

    // AI context (default on, --no-ai to skip)
    if !opts.no_ai {
        let stem = output_path
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let ai_path = output_path.with_file_name(format!("{}.ai.md", stem));
        let ai_content = archidoc_engine::ai_context::generate(docs);
        fs::write(&ai_path, &ai_content).unwrap_or_else(|e| {
            eprintln!("error: failed to write {}: {}", ai_path.display(), e);
            std::process::exit(1);
        });
        if verbosity != Verbosity::Quiet {
            println!("wrote {}", ai_path.display());
        }
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
    let link_base = arch_file.parent().unwrap_or(root.as_path());
    let report = archidoc_engine::check::check_drift(docs, &arch_file, link_base);

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

fn run_init_file(args: InitFileArgs) {
    use archidoc_engine::init_cmd;

    if args.list {
        let handlers = init_cmd::all_handlers();
        println!("available init handlers:");
        println!();
        for handler in &handlers {
            let dry_run_note = if handler.supports_dry_run() {
                " (supports --dry-run)"
            } else {
                ""
            };
            println!("  {}{}", handler.name(), dry_run_note);
            println!("    {}", handler.description());
            println!();
        }
        return;
    }

    let Some(name) = &args.name else {
        eprintln!("error: provide a handler name or use --list");
        std::process::exit(1);
    };

    let handler = match init_cmd::find_handler(name) {
        Some(h) => h,
        None => {
            eprintln!("error: unknown init handler '{}'", name);
            eprintln!();
            eprintln!("run `archidoc init-file --list` to see available handlers.");
            std::process::exit(1);
        }
    };

    let cwd = std::env::current_dir().expect("failed to get current directory");
    let target = args.target.unwrap_or_else(|| cwd.clone());
    let target = if target.is_absolute() {
        target
    } else {
        cwd.join(target)
    };

    let vars: std::collections::BTreeMap<String, String> =
        args.vars.into_iter().collect();

    let extra_args = archidoc_engine::init_cmd::HandlerArgs {
        files: args.files,
        human: args.human,
        both: args.both,
        depth: args.depth,
        lang: args.lang,
        out: args.out,
    };

    // Dry-run mode
    if args.dry_run {
        if handler.supports_dry_run() {
            match handler.dry_run(&target, &vars, &extra_args) {
                Ok(items) => {
                    if items.is_empty() {
                        println!("nothing to do.");
                    } else {
                        println!("would generate ({} items):", items.len());
                        println!();
                        for item in &items {
                            println!("  {}", item);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("error: handler '{}' does not support --dry-run", name);
            std::process::exit(1);
        }
        return;
    }

    // Generate
    match handler.generate(&target, &vars, &extra_args) {
        Ok(outputs) => {
            if outputs.is_empty() {
                println!("nothing to generate.");
                return;
            }

            for output in &outputs {
                if output.path.to_string_lossy() == "-" {
                    // stdout sentinel — print directly
                    print!("{}", output.contents);
                } else {
                    // Write to file
                    if let Some(parent) = output.path.parent() {
                        if !parent.exists() {
                            if let Err(e) = fs::create_dir_all(parent) {
                                eprintln!("error: failed to create {}: {}", parent.display(), e);
                                std::process::exit(1);
                            }
                        }
                    }
                    match fs::write(&output.path, &output.contents) {
                        Ok(_) => {
                            let display = output
                                .path
                                .strip_prefix(&target)
                                .unwrap_or(&output.path);
                            println!("wrote {}", display.display());
                        }
                        Err(e) => {
                            eprintln!(
                                "error: failed to write {}: {}",
                                output.path.display(),
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_new(args: NewArgs) {
    let cwd = std::env::current_dir().expect("failed to get current directory");

    if args.list {
        let templates = archidoc_engine::folder_scaffold::list_templates(&cwd);
        if templates.is_empty() {
            println!("no folder templates found.");
            println!();
            println!("create templates in .archidoc/templates/scaffold-folder-templates/<name>/");
            return;
        }
        println!("available folder templates:");
        println!();
        for (name, path, manifest) in &templates {
            let rel = path
                .strip_prefix(&cwd)
                .unwrap_or(path);
            println!("  {}  (from {})", name, rel.display());
            println!("    {}", manifest.template.description);
            if !manifest.variables.is_empty() {
                let var_names: Vec<&str> = manifest.variables.iter().map(|v| v.name.as_str()).collect();
                println!("    vars: {}", var_names.join(", "));
            }
            println!();
        }
        return;
    }

    let Some(name) = &args.name else {
        eprintln!("error: provide a template name or use --list");
        std::process::exit(1);
    };

    // Discover template
    let template_dir = match archidoc_engine::folder_scaffold::discover_template(name, &cwd) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!();
            eprintln!("run `archidoc new --list` to see available templates.");
            std::process::exit(1);
        }
    };

    // Load manifest
    let manifest = match archidoc_engine::folder_scaffold::load_manifest(&template_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    // Inspect mode
    if args.inspect {
        println!("template: {}", manifest.template.name);
        println!("version:  {}", manifest.template.version);
        println!("description: {}", manifest.template.description);
        println!();
        if manifest.variables.is_empty() {
            println!("no variables required.");
        } else {
            println!("variables:");
            for var in &manifest.variables {
                let required = if var.required { "required" } else { "optional" };
                let default = var
                    .default
                    .as_deref()
                    .map(|d| format!(" (default: {})", d))
                    .unwrap_or_default();
                println!("  --var {}=<value>  [{}{}]", var.name, required, default);
                println!("    {}", var.description);
            }
        }
        if !manifest.post_hooks.is_empty() {
            println!();
            println!("post-hooks:");
            for hook in &manifest.post_hooks {
                println!("  {} — {}", hook.command, hook.description);
            }
        }
        return;
    }

    // Collect variables
    let variables = match archidoc_engine::folder_scaffold::collect_variables(&manifest, &args.vars)
    {
        Ok(v) => v,
        Err(archidoc_engine::folder_scaffold::ScaffoldError::MissingVariables(missing)) => {
            eprintln!("error: missing required variable(s): {}", missing.join(", "));
            eprintln!();
            eprintln!("provide them with --var flags:");
            for name in &missing {
                let desc = manifest
                    .variables
                    .iter()
                    .find(|v| &v.name == name)
                    .map(|v| v.description.as_str())
                    .unwrap_or("");
                eprintln!("  --var {}=<value>    {}", name, desc);
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    // Determine target
    let target = args.target.unwrap_or_else(|| cwd.clone());
    let target = if target.is_absolute() {
        target
    } else {
        cwd.join(target)
    };

    // Build plan
    let plan = match archidoc_engine::folder_scaffold::build_plan(
        name,
        &template_dir,
        &target,
        &variables,
        manifest.post_hooks.clone(),
        args.force,
    ) {
        Ok(p) => p,
        Err(archidoc_engine::folder_scaffold::ScaffoldError::WouldOverwrite(path)) => {
            eprintln!(
                "error: would overwrite existing file: {}",
                path.display()
            );
            eprintln!("use --force to overwrite.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    // Dry run
    if args.dry_run {
        println!("dry run — would create:");
        println!();
        for action in &plan.actions {
            match action {
                archidoc_engine::folder_scaffold::ScaffoldAction::CreateDir { path } => {
                    let rel = path.strip_prefix(&target).unwrap_or(path);
                    println!("  dir   {}/", rel.display());
                }
                archidoc_engine::folder_scaffold::ScaffoldAction::CreateFile { path, .. } => {
                    let rel = path.strip_prefix(&target).unwrap_or(path);
                    println!("  file  {}", rel.display());
                }
            }
        }
        if !plan.post_hooks.is_empty() {
            println!();
            println!("post-hooks:");
            for hook in &plan.post_hooks {
                println!("  {}", hook.command);
            }
        }
        return;
    }

    // Execute
    match archidoc_engine::folder_scaffold::execute_plan(&plan) {
        Ok(result) => {
            let mut created = 0;
            let mut skipped = 0;
            for outcome in &result.outcomes {
                match outcome {
                    archidoc_engine::folder_scaffold::ActionOutcome::Created(path) => {
                        let rel = path.strip_prefix(&target).unwrap_or(path);
                        println!("created  {}", rel.display());
                        created += 1;
                    }
                    archidoc_engine::folder_scaffold::ActionOutcome::Skipped { path, .. } => {
                        skipped += 1;
                        let rel = path.strip_prefix(&target).unwrap_or(path);
                        println!("skipped  {}", rel.display());
                    }
                    archidoc_engine::folder_scaffold::ActionOutcome::Failed { path, error } => {
                        let rel = path.strip_prefix(&target).unwrap_or(path);
                        eprintln!("failed   {}  ({})", rel.display(), error);
                    }
                }
            }
            println!();
            println!("created: {}  skipped: {}", created, skipped);

            // Report hook results
            for hr in &result.hook_results {
                if hr.success {
                    println!("hook ok: {}", hr.command);
                } else {
                    eprintln!("hook failed: {} — {}", hr.command, hr.output.trim());
                }
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Polyglot adapter detection — auto-detect and merge language adapters
// ---------------------------------------------------------------------------

/// Collect ModuleDoc from all detected language adapters.
///
/// 1. Always runs the built-in Rust adapter
/// 2. Auto-detects TypeScript (package.json + archidoc-ts available) and shells out
/// 3. Merges results if both produce output
fn collect_all_docs(root: &std::path::Path, verbosity: Verbosity) -> Vec<archidoc_types::ModuleDoc> {
    let rust_docs = archidoc_rust::walker::extract_all_docs(root);
    let ts_docs = detect_and_run_ts_adapter(root, verbosity);

    if !rust_docs.is_empty() && !ts_docs.is_empty() {
        match archidoc_engine::merge::merge_ir(vec![rust_docs, ts_docs]) {
            Ok(merged) => merged,
            Err(e) => {
                eprintln!("warning: IR merge failed ({}), using Rust-only docs", e);
                archidoc_rust::walker::extract_all_docs(root)
            }
        }
    } else if !ts_docs.is_empty() {
        ts_docs
    } else {
        rust_docs
    }
}

/// Detect and run the archidoc-ts adapter if the project has TypeScript sources.
///
/// Returns an empty Vec if:
/// - No package.json in root
/// - archidoc-ts is not installed (npx would fail)
/// - The subprocess fails or returns invalid JSON
fn detect_and_run_ts_adapter(root: &std::path::Path, verbosity: Verbosity) -> Vec<archidoc_types::ModuleDoc> {
    if !root.join("package.json").exists() {
        return vec![];
    }

    // On Windows, .cmd/.bat scripts aren't directly executable by Command::new.
    // Use cmd /c to resolve npx.cmd from PATH.
    let output = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/c", "npx", "archidoc-ts", &root.to_string_lossy()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
    } else {
        std::process::Command::new("npx")
            .args(["archidoc-ts", &root.to_string_lossy()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
    };

    match output {
        Ok(result) if result.status.success() => {
            let json = String::from_utf8_lossy(&result.stdout);
            match archidoc_engine::ir::deserialize(&json) {
                Ok(docs) if !docs.is_empty() => {
                    if verbosity == Verbosity::Verbose {
                        eprintln!("  typescript adapter: {} modules", docs.len());
                    }
                    docs
                }
                Ok(_) => vec![],
                Err(e) => {
                    eprintln!("warning: archidoc-ts output could not be parsed: {}", e);
                    vec![]
                }
            }
        }
        Ok(result) => {
            if verbosity == Verbosity::Verbose {
                let stderr = String::from_utf8_lossy(&result.stderr);
                eprintln!("  typescript adapter not available: {}", stderr.trim());
            }
            vec![]
        }
        Err(_) => {
            if verbosity == Verbosity::Verbose {
                eprintln!("  typescript adapter skipped (npx not found)");
            }
            vec![]
        }
    }
}

