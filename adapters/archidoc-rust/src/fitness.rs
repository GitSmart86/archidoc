use std::path::Path;

use archidoc_types::ModuleDoc;

use crate::pattern_heuristic;

/// Result of running a fitness function across modules.
#[derive(Debug)]
pub struct FitnessResult {
    pub passed: bool,
    pub checked: usize,
    pub failures: Vec<FitnessFailure>,
}

/// A single module that failed a fitness check.
#[derive(Debug)]
pub struct FitnessFailure {
    pub module_path: String,
    pub source_file: String,
    pub reason: String,
}

/// H4: All modules with pattern "Strategy" must define at least one trait.
pub fn all_strategy_modules_define_a_trait(docs: &[ModuleDoc]) -> FitnessResult {
    check_modules_for_pattern(docs, "Strategy", "no trait definition found")
}

/// H5: All modules with pattern "Facade" must re-export submodules.
pub fn all_facade_modules_reexport_submodules(docs: &[ModuleDoc]) -> FitnessResult {
    check_modules_for_pattern(docs, "Facade", "no pub use re-exports or pub mod declarations found")
}

/// H6: All modules with pattern "Observer" must have channels or callbacks.
pub fn all_observer_modules_have_channels_or_callbacks(docs: &[ModuleDoc]) -> FitnessResult {
    check_modules_for_pattern(docs, "Observer", "no channel types or callback parameters found")
}

/// Run a named fitness function by name.
pub fn run_fitness(name: &str, docs: &[ModuleDoc]) -> Option<FitnessResult> {
    match name {
        "all_strategy_modules_define_a_trait" => {
            Some(all_strategy_modules_define_a_trait(docs))
        }
        "all_facade_modules_reexport_submodules" => {
            Some(all_facade_modules_reexport_submodules(docs))
        }
        "all_observer_modules_have_channels_or_callbacks" => {
            Some(all_observer_modules_have_channels_or_callbacks(docs))
        }
        _ => None,
    }
}

/// Generic: check all modules with the given pattern against the corresponding heuristic.
fn check_modules_for_pattern(
    docs: &[ModuleDoc],
    pattern: &str,
    failure_reason: &str,
) -> FitnessResult {
    let mut checked = 0;
    let mut failures = Vec::new();

    for doc in docs {
        if doc.pattern != pattern {
            continue;
        }

        checked += 1;

        let source_dir = match Path::new(&doc.source_file).parent() {
            Some(dir) => dir,
            None => {
                failures.push(FitnessFailure {
                    module_path: doc.module_path.clone(),
                    source_file: doc.source_file.clone(),
                    reason: "could not determine source directory".to_string(),
                });
                continue;
            }
        };

        if !pattern_heuristic::check_module_pattern(pattern, source_dir) {
            failures.push(FitnessFailure {
                module_path: doc.module_path.clone(),
                source_file: doc.source_file.clone(),
                reason: failure_reason.to_string(),
            });
        }
    }

    FitnessResult {
        passed: failures.is_empty(),
        checked,
        failures,
    }
}

/// Format a fitness result as human-readable text.
pub fn format_fitness_result(name: &str, result: &FitnessResult) -> String {
    let mut out = String::new();

    if result.passed {
        out.push_str(&format!(
            "PASS: {} — checked {} module(s)\n",
            name, result.checked
        ));
    } else {
        out.push_str(&format!(
            "FAIL: {} — {}/{} module(s) failed\n",
            name,
            result.failures.len(),
            result.checked
        ));
        for failure in &result.failures {
            out.push_str(&format!(
                "  {} ({}): {}\n",
                failure.module_path, failure.source_file, failure.reason
            ));
        }
    }

    out
}
