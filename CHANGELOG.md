# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-12

### Added

- Rust language adapter (`archidoc-rust`): extracts C4 annotations from `//!` doc comments in `mod.rs` and `lib.rs`
- TypeScript language adapter (`archidoc-ts`): extracts `@c4` JSDoc annotations from `index.ts`
- Core generator engine (`archidoc-engine`): produces Mermaid C4 diagrams, Markdown docs, and draw.io CSV exports
- CLI binary (`archidoc`): orchestrates adapter + engine with modes for generate, check, health, validate, emit-ir, from-json, validate-ir
- JSON IR schema (`ModuleDoc[]`): portable contract between language adapters and the core engine
- Documentation drift detection (`--check`): exits non-zero when generated docs are stale
- Architecture health reporting (`--health`): aggregates file maturity and pattern confidence
- File table validation (`--validate`): detects ghost entries and orphan files
- Pattern validation: structural heuristics for Observer, Strategy, and Facade patterns with automatic planned-to-verified promotion
- BDD test infrastructure: DSL (Facade), protocol drivers (Strategy), and fakes — 79 Rust tests + 35 TypeScript tests
- Cross-language pipeline: any adapter emitting conforming JSON IR can use the full engine
- Language adapter guide with working Python example
