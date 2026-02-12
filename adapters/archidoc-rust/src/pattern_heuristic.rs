//! Structural heuristics for GoF pattern detection.
//!
//! Each heuristic checks for **structural evidence** of a pattern — not proof.
//! They are intentionally permissive to avoid false negatives: it's better to
//! verify a module that loosely matches than to miss one that clearly does.
//!
//! A heuristic returning `true` means "there is structural evidence consistent
//! with this pattern." It does NOT mean "this code correctly implements the
//! pattern." The promotion from `planned` to `verified` reflects structural
//! alignment, not behavioral correctness.

use std::path::Path;

use syn::{Item, Visibility};

use crate::walker;

/// Check if Rust source code structurally matches the Observer pattern (H1).
///
/// Looks for channel types (mpsc, crossbeam, tokio broadcast/watch),
/// callback type parameters (Fn/FnMut/FnOnce), or event-related identifiers.
pub fn check_observer(source: &str) -> bool {
    // String-based heuristics for channel/callback patterns
    let indicators = [
        "mpsc::Sender",
        "mpsc::Receiver",
        "mpsc::channel",
        "crossbeam_channel",
        "broadcast::Sender",
        "watch::Sender",
        "watch::Receiver",
        "Box<dyn Fn",
        "Box<dyn FnMut",
        "Box<dyn FnOnce",
        "Arc<dyn Fn",
        "impl Fn(",
        "impl FnMut(",
        "impl FnOnce(",
        "-> Receiver",
        "-> Sender",
    ];

    for indicator in &indicators {
        if source.contains(indicator) {
            return true;
        }
    }

    // Parse with syn to check for method names suggesting observer pattern
    if let Ok(file) = syn::parse_file(source) {
        for item in &file.items {
            if let Item::Trait(trait_item) = item {
                for method in &trait_item.items {
                    if let syn::TraitItem::Fn(m) = method {
                        let name = m.sig.ident.to_string();
                        if matches!(
                            name.as_str(),
                            "subscribe"
                                | "unsubscribe"
                                | "notify"
                                | "on_event"
                                | "on_update"
                                | "on_change"
                                | "emit"
                                | "publish"
                                | "add_listener"
                                | "remove_listener"
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check if Rust source code structurally matches the Strategy pattern (H2).
///
/// Looks for trait definitions — a Strategy module defines an interchangeable
/// behavior contract via a trait.
pub fn check_strategy(source: &str) -> bool {
    if let Ok(file) = syn::parse_file(source) {
        for item in &file.items {
            if let Item::Trait(_) = item {
                return true;
            }
        }
    }
    false
}

/// Check if Rust source code structurally matches the Facade pattern (H3).
///
/// Looks for `pub use` re-exports or `pub mod` declarations — a Facade
/// provides a simplified entry point by re-exporting from submodules.
pub fn check_facade(source: &str) -> bool {
    if let Ok(file) = syn::parse_file(source) {
        let mut pub_use_count = 0;
        let mut pub_mod_count = 0;

        for item in &file.items {
            match item {
                Item::Use(use_item) => {
                    if matches!(use_item.vis, Visibility::Public(_)) {
                        pub_use_count += 1;
                    }
                }
                Item::Mod(mod_item) => {
                    if matches!(mod_item.vis, Visibility::Public(_)) {
                        pub_mod_count += 1;
                    }
                }
                _ => {}
            }
        }

        // A Facade must have at least one pub use or two pub mod declarations
        pub_use_count >= 1 || pub_mod_count >= 2
    } else {
        false
    }
}

/// Run the appropriate heuristic for a named GoF pattern.
pub fn check_pattern(pattern: &str, source: &str) -> bool {
    match pattern {
        "Observer" => check_observer(source),
        "Strategy" => check_strategy(source),
        "Facade" => check_facade(source),
        _ => false,
    }
}

/// Scan all `.rs` files in a module's source directory for structural evidence.
///
/// Returns true if ANY file in the directory passes the pattern heuristic.
/// File discovery is delegated to `walker::read_rs_sources` to keep this
/// module focused on AST analysis.
pub fn check_module_pattern(pattern: &str, source_dir: &Path) -> bool {
    walker::read_rs_sources(source_dir)
        .iter()
        .any(|(_, source)| check_pattern(pattern, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_detects_trait() {
        let source = r#"
            pub trait Calculator {
                fn calculate(&self, prices: &[f64]) -> f64;
            }
        "#;
        assert!(check_strategy(source));
    }

    #[test]
    fn strategy_rejects_no_trait() {
        let source = r#"
            pub struct SimpleCalc;
            impl SimpleCalc {
                pub fn calculate(&self, prices: &[f64]) -> f64 {
                    prices.iter().sum()
                }
            }
        "#;
        assert!(!check_strategy(source));
    }

    #[test]
    fn facade_detects_pub_use() {
        let source = r#"
            pub use crate::calc::Calculator;
            pub use crate::store::DataStore;
        "#;
        assert!(check_facade(source));
    }

    #[test]
    fn facade_detects_pub_mod() {
        let source = r#"
            pub mod calc;
            pub mod store;
        "#;
        assert!(check_facade(source));
    }

    #[test]
    fn facade_rejects_private_mods() {
        let source = r#"
            mod calc;
            mod store;
        "#;
        assert!(!check_facade(source));
    }

    #[test]
    fn observer_detects_channel() {
        let source = r#"
            use std::sync::mpsc::Sender;
            use std::sync::mpsc::Receiver;
            pub fn create_bus() -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
                std::sync::mpsc::channel()
            }
        "#;
        assert!(check_observer(source));
    }

    #[test]
    fn observer_detects_callback_trait() {
        let source = r#"
            pub trait EventBus {
                fn subscribe(&mut self, handler: Box<dyn Fn(Event)>);
                fn notify(&self, event: Event);
            }
        "#;
        assert!(check_observer(source));
    }

    #[test]
    fn observer_rejects_plain_struct() {
        let source = r#"
            pub struct Logger {
                path: String,
            }
            impl Logger {
                pub fn log(&self, msg: &str) {
                    println!("{}", msg);
                }
            }
        "#;
        assert!(!check_observer(source));
    }

    #[test]
    fn check_pattern_dispatches_correctly() {
        let strategy_src = "pub trait Algo { fn run(&self); }";
        assert!(check_pattern("Strategy", strategy_src));
        assert!(!check_pattern("Observer", strategy_src));
        assert!(!check_pattern("UnknownPattern", strategy_src));
    }
}
