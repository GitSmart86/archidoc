# Archidoc — Publishing & Roadmap TODO

## v0.1.0 — Pre-Publish Checklist

### Must Fix (blocks publishing)

- [x] Update README test count: "48 Rust tests" → "79 Rust tests + 35 TypeScript tests = 114 total"
- [x] Add `--version` flag to CLI (every published CLI tool needs one)
- [x] Improve crate descriptions for crates.io listings:
  - `archidoc-types`: fine as-is
  - `archidoc-engine`: fine as-is
  - `archidoc-cli`: rewritten to "Architecture documentation compiler — generates C4 diagrams and docs from source code annotations"
- [x] Add value proposition hook to top of README (2-3 sentences explaining WHY, not just WHAT)
- [x] Update README install section: after publishing, primary instruction should be `cargo install archidoc-cli`, source build becomes secondary
- [x] Create CHANGELOG.md with v0.1.0 "initial release" entry
- [ ] Create GitHub org `archidoc` and repo `archidoc/archidoc`
- [ ] Push code to GitHub
- [x] Commit all uncommitted changes (D9/D10/D11 tests, metadata, LICENSE, fixes)

### Should Fix (quality / discoverability)

- [x] Add GitHub Actions CI workflow (cargo test + npm test, even a minimal 10-line one)
- [ ] Add CI badge to README (after CI runs on GitHub)
- [ ] Add crates.io and npm badges to README after publishing
- [x] Add "Getting Started: Annotating Your Project" section to README — quick-start guide for someone adopting archidoc on an existing codebase (not writing an adapter)

### Publish Steps (in order)

- [ ] `cargo login`
- [ ] `cargo publish -p archidoc-types`
- [ ] `cargo publish -p archidoc-engine`
- [ ] `cargo publish -p archidoc-rust`
- [ ] `cargo publish -p archidoc-cli`
- [ ] `npm login` (from `adapters/archidoc-ts`)
- [ ] `npm run build` (from `adapters/archidoc-ts`)
- [ ] `npm publish` (from `adapters/archidoc-ts`)
- [ ] Tag release: `git tag v0.1.0`
- [ ] Create GitHub Release with release notes

---

## v0.2.0 — Roadmap

### CLI Improvements

- [ ] Add `--output` / `-o` flag to configure output directory (currently hardcoded to `docs/generated/`)
- [ ] Migrate from hand-rolled arg parsing to `clap` for auto-generated help, shell completions, and better error messages
- [ ] Add `--quiet` / `--verbose` flags for controlling output verbosity
- [ ] Support flat crate structures (`src/foo.rs` not just `src/foo/mod.rs`) in the Rust adapter

### Annotation Convention

- [ ] Evaluate whether `<<container>>` syntax should be revised for cross-language compatibility (currently triggers rustdoc HTML warnings requiring `#![allow(rustdoc::invalid_html_tags)]` in user crates)
- [ ] Consider supporting `@c4 container` syntax in Rust doc comments as an alternative to `<<container>>`
- [ ] Document recommended `#![allow(rustdoc::invalid_html_tags)]` in the annotation guide for Rust users

### Pattern Validation

- [ ] Expand pattern heuristics beyond Observer, Strategy, Facade (currently only 3 of 23 GoF patterns supported)
- [ ] Allow users to register custom pattern heuristics
- [ ] Consider `planned` → `verified` auto-promotion for additional patterns (Builder, Factory, Adapter, etc.)

### Adapters

- [ ] Complete C5: Integrate `cargo-modules` for import graph extraction (optional validation)
- [ ] Complete C6: Integrate `cargo-modules orphans` for undocumented file detection
- [ ] Complete F6: Parse `import`/`export` statements in TS adapter for relationship extraction
- [ ] Community adapter template: scaffold a new adapter project with `archidoc init-adapter --lang <name>`

### Documentation & Adoption

- [ ] Write "Annotating Your Project" guide — step-by-step walkthrough for adopting archidoc on an existing codebase: what to annotate first, what good annotations look like, common mistakes
- [ ] Write "archidoc for LLMs" guide — structured prompt/context document that an LLM can read to annotate any project correctly (annotation spec + examples + common pitfalls)
- [ ] Add example annotated projects (a small Rust crate, a small TS package) as reference implementations
- [ ] Publish the annotation spec as a standalone document (currently embedded in README)

### CI/CD & Release

- [ ] Add GitHub Actions workflow for automated testing on PR
- [ ] Add GitHub Actions workflow for automated crate/npm publishing on tag
- [ ] Add pre-commit hook example using `archidoc --check` for drift detection
- [ ] Cross-platform binary releases via `cargo-dist` or GitHub Actions matrix (Windows, macOS, Linux)

### Engine

- [ ] Make output directory configurable (not hardcoded to `docs/generated/`)
- [ ] Support merging IR from multiple adapters in a single invocation (polyglot projects)
- [ ] Add PlantUML output format alongside Mermaid
- [ ] Add JSON output for health and validation reports (machine-readable, not just human-readable text)

### Deferred from v0.1.0

- [ ] C5: `cargo-modules` import graph integration
- [ ] C6: `cargo-modules` orphan detection
- [ ] F6: TypeScript import/export relationship extraction
