use std::path::Path;

use archidoc_types::{ModuleDoc, PatternStatus};

use crate::pattern_heuristic;

/// Recognized patterns that have structural heuristics.
const VERIFIABLE_PATTERNS: &[&str] = &[
    "Observer", "Strategy", "Facade", "Builder", "Factory",
    "Adapter", "Decorator", "Singleton", "Command",
];

/// H7: Auto-promote pattern labels from `planned` to `verified`
/// when structural heuristics pass.
///
/// For each module:
/// - Skip if pattern_status is already Verified
/// - Skip if pattern has no heuristic (not in VERIFIABLE_PATTERNS)
/// - Scan the module's source directory for structural evidence
/// - Promote to Verified if the heuristic passes
///
/// Returns the number of modules promoted.
pub fn auto_promote(docs: &mut [ModuleDoc]) -> usize {
    let mut promoted = 0;

    for doc in docs.iter_mut() {
        if doc.pattern_status != PatternStatus::Planned {
            continue;
        }

        if !VERIFIABLE_PATTERNS.contains(&doc.pattern.as_str()) {
            continue;
        }

        let source_dir = match Path::new(&doc.source_file).parent() {
            Some(dir) => dir,
            None => continue,
        };

        if pattern_heuristic::check_module_pattern(&doc.pattern, source_dir) {
            doc.pattern_status = PatternStatus::Verified;
            promoted += 1;
        }
    }

    promoted
}
