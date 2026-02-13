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

/// Check if Rust source code structurally matches the Builder pattern.
///
/// Looks for chained setter methods returning Self, or a `build()` method.
pub fn check_builder(source: &str) -> bool {
    if let Ok(file) = syn::parse_file(source) {
        for item in &file.items {
            if let Item::Impl(impl_item) = item {
                let mut has_self_return = 0;
                let mut has_build = false;

                for method in &impl_item.items {
                    if let syn::ImplItem::Fn(m) = method {
                        let name = m.sig.ident.to_string();
                        if name == "build" {
                            has_build = true;
                        }
                        // Check for methods returning Self or &mut Self
                        if let syn::ReturnType::Type(_, ty) = &m.sig.output {
                            let ty_str = quote::quote!(#ty).to_string();
                            if ty_str.contains("Self") {
                                has_self_return += 1;
                            }
                        }
                    }
                }

                // Builder pattern: build() method, or 2+ chained setters returning Self
                if has_build || has_self_return >= 2 {
                    return true;
                }
            }
        }
    }

    // String-based fallback
    let indicators = ["fn build(self)", "fn build(&self)", "fn build(&mut self)"];
    indicators.iter().any(|i| source.contains(i))
}

/// Check if Rust source code structurally matches the Factory pattern.
///
/// Looks for functions returning trait objects or named create/make methods.
pub fn check_factory(source: &str) -> bool {
    let indicators = [
        "-> Box<dyn",
        "-> Arc<dyn",
        "-> Rc<dyn",
        "fn create(",
        "fn create_",
        "fn make(",
        "fn make_",
    ];

    for indicator in &indicators {
        if source.contains(indicator) {
            return true;
        }
    }

    if let Ok(file) = syn::parse_file(source) {
        for item in &file.items {
            if let Item::Fn(func) = item {
                if let syn::ReturnType::Type(_, ty) = &func.sig.output {
                    let ty_str = quote::quote!(#ty).to_string();
                    if ty_str.contains("Box < dyn") || ty_str.contains("impl ") {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if Rust source code structurally matches the Adapter pattern.
///
/// Looks for a struct wrapping another type combined with a trait implementation.
pub fn check_adapter(source: &str) -> bool {
    if let Ok(file) = syn::parse_file(source) {
        let mut has_wrapper_struct = false;
        let mut has_trait_impl = false;

        for item in &file.items {
            match item {
                Item::Struct(s) => {
                    // A wrapper struct typically has 1-2 fields
                    if let syn::Fields::Named(fields) = &s.fields {
                        if (1..=2).contains(&fields.named.len()) {
                            has_wrapper_struct = true;
                        }
                    }
                }
                Item::Impl(impl_item) => {
                    if impl_item.trait_.is_some() {
                        has_trait_impl = true;
                    }
                }
                _ => {}
            }
        }

        return has_wrapper_struct && has_trait_impl;
    }

    false
}

/// Check if Rust source code structurally matches the Decorator pattern.
///
/// Looks for a struct containing a trait object field that implements the same trait.
pub fn check_decorator(source: &str) -> bool {
    let indicators = [
        "Box<dyn",
        "Arc<dyn",
    ];

    let has_trait_object_field = indicators.iter().any(|i| source.contains(i));

    if has_trait_object_field {
        if let Ok(file) = syn::parse_file(source) {
            let mut has_struct_with_dyn = false;
            let mut has_trait_impl = false;

            for item in &file.items {
                match item {
                    Item::Struct(s) => {
                        if let syn::Fields::Named(fields) = &s.fields {
                            for field in &fields.named {
                                let ty_str = quote::quote!(#field).to_string();
                                if ty_str.contains("Box < dyn") || ty_str.contains("Arc < dyn") {
                                    has_struct_with_dyn = true;
                                }
                            }
                        }
                    }
                    Item::Impl(impl_item) => {
                        if impl_item.trait_.is_some() {
                            has_trait_impl = true;
                        }
                    }
                    _ => {}
                }
            }

            return has_struct_with_dyn && has_trait_impl;
        }
    }

    false
}

/// Check if Rust source code structurally matches the Singleton pattern.
///
/// Looks for static/lazy initialization patterns or instance() methods.
pub fn check_singleton(source: &str) -> bool {
    let indicators = [
        "lazy_static!",
        "once_cell::sync::Lazy",
        "OnceLock",
        "OnceCell",
        "static ref ",
        "fn instance()",
        "fn get_instance()",
    ];

    indicators.iter().any(|i| source.contains(i))
}

/// Check if Rust source code structurally matches the Command pattern.
///
/// Looks for traits with execute/run methods, or enums used for dispatch.
pub fn check_command(source: &str) -> bool {
    if let Ok(file) = syn::parse_file(source) {
        for item in &file.items {
            if let Item::Trait(trait_item) = item {
                for method in &trait_item.items {
                    if let syn::TraitItem::Fn(m) = method {
                        let name = m.sig.ident.to_string();
                        if matches!(
                            name.as_str(),
                            "execute" | "exec" | "run" | "invoke" | "perform" | "undo" | "redo"
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

/// Run the appropriate heuristic for a named GoF pattern.
pub fn check_pattern(pattern: &str, source: &str) -> bool {
    match pattern {
        "Observer" => check_observer(source),
        "Strategy" => check_strategy(source),
        "Facade" => check_facade(source),
        "Builder" => check_builder(source),
        "Factory" => check_factory(source),
        "Adapter" => check_adapter(source),
        "Decorator" => check_decorator(source),
        "Singleton" => check_singleton(source),
        "Command" => check_command(source),
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
