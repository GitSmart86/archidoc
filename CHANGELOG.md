# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-02-21

### Added

- **Polyglot auto-detection** — `archidoc .` now auto-detects TypeScript projects (via `package.json`), shells out to `archidoc-ts`, and merges IR from both Rust and TypeScript adapters. All downstream modes (`--check`, `--health`, `--validate`, `--emit-ir`) use the unified polyglot result. Graceful degradation when `archidoc-ts` is not installed.
- **npm binary wrapper** (`archidoc` on npm) — `npm install archidoc` downloads the prebuilt native binary from GitHub Releases and includes `archidoc-ts` as a dependency. One install for full polyglot support, no Rust toolchain needed.
- **GitHub Release binary naming fix** — release assets are now named with target triples (`archidoc-x86_64-unknown-linux-gnu`, etc.) to avoid name conflicts across platforms.
- **CLI polyglot integration tests** — 6 new tests covering Rust-only, TS-only, and polyglot project detection.
- **npm wrapper tests** — 21 tests for platform detection, URL construction, and package structure.

## [0.3.1] - 2026-02-16

### Added

- **Intent-tier sections in `archidoc init`** — the scaffolded root template now includes Business Context, Domain Model, Users & Stakeholders, Success Criteria, and Constraints & Trade-offs sections before the C4 structure sections, giving a top-down flow from intent to architecture.

### Changed

- README polish: clearer opening blurb, added "What It Does" summary, fixed markdown formatting for fenced code blocks.

## [0.3.0] - 2026-02-14

### Added

- **AI context output** — `archidoc` now generates `ARCHITECTURE.ai.md` alongside `ARCHITECTURE.md` by default. Token-optimized tree format for LLM consumption (~75% fewer tokens). Strips Mermaid diagrams, ASCII art, and markdown tables. Each module appears once with its GoF pattern and description. Suppress with `--no-ai`.
- **`archidoc init` subcommand** — scaffolds a root-level `lib.rs` / `index.ts` template with TODO sections for purpose, C4 context diagram, data flow, concurrency patterns, deployment, and external dependencies. Auto-detects language from `Cargo.toml` / `package.json`, or use `--lang rust` / `--lang ts`.
- CLI: `--no-ai` flag to suppress `ARCHITECTURE.ai.md` generation
- Engine: `ai_context.rs` — token-optimized AI context generator with orphaned header cleanup and ancestor-aware tree indentation
- Engine: `init.rs` — root template generator with Rust and TypeScript comment style support
- Documentation: README rewritten with greenfield and brownfield getting-started paths
- Documentation: updated LLM guide with ARCHITECTURE.ai.md usage and scaffolding commands
- Documentation: updated annotating-your-project guide with Step 0 (root scaffolding)

## [0.2.0] - 2026-02-13

### Changed

- **Single ARCHITECTURE.md output** — `archidoc .` now generates one file with inline Mermaid diagrams, a component index table linking to source files, and a relationship map. Replaces the old `docs/generated/` directory tree (per-module `.md` files, separate diagram files).
- **Default output path** changed from `docs/generated/` to `ARCHITECTURE.md`
- **Drift detection** simplified to single-file comparison against ARCHITECTURE.md
- **PlantUML and draw.io** are now opt-in sidecar outputs (`--plantuml`, `--drawio`)
- **`@c4` syntax only** — removed deprecated `<<container>>`/`<<component>>`/`<<uses:...>>` syntax with no backwards compatibility

### Added

- CLI: clap-based argument parsing with `--output/-o`, `--quiet/--verbose`, `--json` flags
- CLI: `init-adapter` subcommand to scaffold new language adapters
- CLI: `suggest` subcommand to generate annotation templates for unannotated directories
- CLI: `--merge-ir` flag for combining IR from multiple language adapters (polyglot projects)
- CLI: first-run guidance when no annotated modules are found
- Engine: PlantUML C4 diagram output (`--plantuml`)
- Engine: IR merging for polyglot projects
- Engine: annotation scaffolding (`suggest`)
- Rust adapter: flat crate support (`src/foo.rs` alongside `src/foo/mod.rs`)
- Rust adapter: `cargo-modules` integration for C5/C6 extraction
- Rust adapter: pattern heuristics for 9 GoF patterns (Observer, Strategy, Facade, Builder, Factory, Adapter, Decorator, Singleton, Command)
- TypeScript adapter: auto-discovery of import/export relationships between modules
- Documentation: annotation spec, annotation RFC, LLM guide, annotating-your-project guide
- Examples: annotated Rust and TypeScript example projects

### Removed

- `--emit-context` flag (merged into default `archidoc .` output)
- `markdown.rs` (per-module .md generation)
- `context.rs` (consolidated context generation — replaced by `architecture.rs`)
- `<<container>>`/`<<component>>`/`<<uses:...>>` annotation syntax

## [0.1.0] - 2026-02-12

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
- BDD test infrastructure: DSL (Facade), protocol drivers (Strategy), and fakes
- Cross-language pipeline: any adapter emitting conforming JSON IR can use the full engine
- Language adapter guide with working Python example
