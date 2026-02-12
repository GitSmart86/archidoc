use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut root = None;
    let mut mode = Mode::Generate;
    let mut from_json_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => mode = Mode::Check,
            "--health" => mode = Mode::Health,
            "--validate" => mode = Mode::Validate,
            "--emit-ir" => mode = Mode::EmitIr,
            "--from-json" => mode = Mode::FromJsonStdin,
            "--from-json-file" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --from-json-file requires a path argument");
                    std::process::exit(1);
                }
                from_json_file = Some(PathBuf::from(&args[i]));
                mode = Mode::FromJsonFile;
            }
            "--validate-ir" => mode = Mode::ValidateIr,
            "--help" | "-h" => {
                print_usage();
                return;
            }
            arg if !arg.starts_with('-') => {
                root = Some(PathBuf::from(arg));
            }
            other => {
                eprintln!("error: unknown flag '{}'", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Modes that consume JSON IR don't need a source root
    match mode {
        Mode::FromJsonStdin => {
            let docs = read_ir_from_stdin();
            let output_root = root.unwrap_or_else(|| env::current_dir().expect("failed to get current directory"));
            run_generate(&output_root, &docs);
        }
        Mode::FromJsonFile => {
            let path = from_json_file.expect("--from-json-file path missing");
            let docs = read_ir_from_file(&path);
            let output_root = root.unwrap_or_else(|| env::current_dir().expect("failed to get current directory"));
            run_generate(&output_root, &docs);
        }
        Mode::ValidateIr => {
            let json = if let Some(path) = from_json_file {
                fs::read_to_string(&path)
                    .unwrap_or_else(|e| {
                        eprintln!("error: failed to read {}: {}", path.display(), e);
                        std::process::exit(1);
                    })
            } else {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)
                    .expect("failed to read from stdin");
                buf
            };
            run_validate_ir(&json);
        }
        _ => {
            // Modes that parse from source need a root directory
            let root = root.unwrap_or_else(|| env::current_dir().expect("failed to get current directory"));

            if !root.exists() {
                eprintln!("error: path does not exist: {}", root.display());
                std::process::exit(1);
            }

            let docs = archidoc_rust::walker::extract_all_docs(&root);

            match mode {
                Mode::Generate => run_generate(&root, &docs),
                Mode::Check => run_check(&root, &docs),
                Mode::Health => run_health(&docs),
                Mode::Validate => run_validate(&docs),
                Mode::EmitIr => run_emit_ir(&docs),
                _ => unreachable!(),
            }
        }
    }
}

enum Mode {
    Generate,
    Check,
    Health,
    Validate,
    EmitIr,
    FromJsonStdin,
    FromJsonFile,
    ValidateIr,
}

fn run_generate(root: &PathBuf, docs: &[archidoc_types::ModuleDoc]) {
    println!("archidoc: {} modules", docs.len());

    if docs.is_empty() {
        println!("  no annotated modules found");
        return;
    }

    let output_base = root.join("docs").join("generated");
    let design_dir = output_base.join("design");
    let c4_dir = output_base.join("c4");
    let drawio_dir = output_base.join("drawio");

    fs::create_dir_all(&design_dir).expect("failed to create design dir");
    fs::create_dir_all(&c4_dir).expect("failed to create c4 dir");
    fs::create_dir_all(&drawio_dir).expect("failed to create drawio dir");

    archidoc_engine::markdown::generate_all(&design_dir, docs);
    archidoc_engine::mermaid::generate_container(&c4_dir, docs);
    archidoc_engine::mermaid::generate_component(&c4_dir, docs);
    archidoc_engine::drawio::generate_container_csv(&drawio_dir, docs);
    archidoc_engine::drawio::generate_component_csv(&drawio_dir, docs);

    println!("\ngenerated:");
    println!("  design/   markdown docs for each module");
    println!("  c4/       mermaid C4 diagrams");
    println!("  drawio/   draw.io CSV files");
    println!("\noutput: {}", output_base.display());
}

fn run_check(root: &PathBuf, docs: &[archidoc_types::ModuleDoc]) {
    let output_base = root.join("docs").join("generated");
    let report = archidoc_engine::check::check_drift(docs, &output_base);
    let text = archidoc_engine::check::format_drift_report(&report);
    print!("{}", text);

    if report.has_drift() {
        std::process::exit(1);
    }
}

fn run_health(docs: &[archidoc_types::ModuleDoc]) {
    let report = archidoc_engine::health::aggregate_health(docs);
    let text = archidoc_engine::health::format_health_report(&report);
    print!("{}", text);
}

fn run_validate(docs: &[archidoc_types::ModuleDoc]) {
    let report = archidoc_engine::validate::validate_file_tables(docs);
    let text = archidoc_engine::validate::format_validation_report(&report);
    print!("{}", text);

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

fn print_usage() {
    println!("archidoc — architecture documentation compiler");
    println!();
    println!("USAGE:");
    println!("  archidoc [PATH] [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --check              Check for documentation drift (exit 1 if stale)");
    println!("  --health             Print architecture health report");
    println!("  --validate           Validate file tables against filesystem");
    println!("  --emit-ir            Output JSON IR to stdout");
    println!("  --from-json          Read JSON IR from stdin and generate docs");
    println!("  --from-json-file F   Read JSON IR from file F and generate docs");
    println!("  --validate-ir        Validate JSON IR (from stdin or --from-json-file)");
    println!("  --help, -h           Print this help message");
    println!();
    println!("If no option is given, generates documentation to docs/generated/.");
    println!();
    println!("CROSS-LANGUAGE PIPELINE:");
    println!("  archidoc-rust ./src --emit-ir | archidoc --from-json");
    println!("  archidoc --from-json-file ir.json ./output-root");
}
