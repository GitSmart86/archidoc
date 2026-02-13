# RFC: C4 Annotation Syntax — Adoption of @c4 and Removal of <<>> Syntax

## Status

**Implemented** — 2026-02-13

## Historical Context

The archidoc Rust adapter originally supported two syntaxes for C4 level markers:

**Option 1: `<<container>>` syntax (now removed)**

```rust
//! # Bus <<container>>
//!
//! Central messaging backbone for cross-module communication.
```

This syntax was borrowed from C4-PlantUML notation for visual consistency. However, it triggered rustdoc HTML tag validation warnings because rustdoc interpreted `<<...>>` as malformed HTML tags. Users had to add `#![allow(rustdoc::invalid_html_tags)]` to their crate root to suppress these warnings.

**Option 2: `@c4` syntax (now the universal standard)**

```rust
//! @c4 container
//!
//! # Bus
//!
//! Central messaging backbone for cross-module communication.
```

The TypeScript adapter used `@c4` JSDoc tags from the beginning, integrating naturally with the language's documentation conventions.

## Decision Evolution

### Phase 1: Dual Syntax Support (2026-02-12)

Initially, both `<<container>>` and `@c4 container` syntax were supported in the Rust adapter, with `@c4` recommended for new projects.

**Rationale:**
- Backward compatibility for existing users
- Cross-language consistency with TypeScript adapter
- No rustdoc warnings when using `@c4` syntax
- Gradual migration path for existing codebases

### Phase 2: Remove `<<>>` Syntax (2026-02-13)

After evaluating user feedback and analyzing usage patterns, the decision was made to **remove `<<>>` syntax entirely** and make `@c4` the universal standard across all language adapters.

**Rationale:**
- **Single canonical syntax**: Eliminates confusion about which syntax to use
- **Cross-language consistency**: All adapters (Rust, TypeScript, Python) now use identical `@c4` syntax
- **No rustdoc warnings**: Eliminates the need for `#![allow(rustdoc::invalid_html_tags)]`
- **Simplified parser**: No need to check for multiple syntax variants
- **Better tooling support**: Modern editors can provide better autocomplete for `@c4` tags

## Final Implementation

The Rust parser's `extract_c4_level` function now checks only for `@c4` syntax:

```rust
pub fn extract_c4_level(content: &str) -> C4Level {
    if content.contains("@c4 container") {
        C4Level::Container
    } else if content.contains("@c4 component") {
        C4Level::Component
    } else {
        C4Level::Unknown
    }
}
```

Relationship markers were also updated from `<<uses: target, "label", "protocol">>` to `@c4 uses target "label" "protocol"` for consistency.

## Migration Path

### For all projects

The `<<>>` syntax has been completely removed. All projects must use `@c4` syntax:

```rust
//! @c4 container
//!
//! # Bus
//!
//! Central messaging backbone for cross-module communication.
```

Relationship markers:

```rust
//! @c4 uses database "Persists user data" "sqlx"
//! @c4 uses events "Domain events" "crossbeam channel"
```

### Migration Steps

For projects using the old `<<>>` syntax:

1. Replace `//! # Name <<container>>` with:
   ```rust
   //! @c4 container
   //!
   //! # Name
   ```

2. Replace `//! # Name <<component>>` with:
   ```rust
   //! @c4 component
   //!
   //! # Name
   ```

3. Replace `//! <<uses: target, "label", "protocol">>` with:
   ```rust
   //! @c4 uses target "label" "protocol"
   ```

4. Remove `#![allow(rustdoc::invalid_html_tags)]` from crate roots (no longer needed)

## Cross-Language Consistency

| Language   | C4 Container Syntax | C4 Component Syntax | Relationship Syntax |
|------------|-------------------|-------------------|---------------------|
| Rust       | `@c4 container` | `@c4 component` | `@c4 uses target "label" "protocol"` |
| TypeScript | `@c4 container` | `@c4 component` | `@c4 uses target "label" "protocol"` |
| Python*    | `@c4 container` (planned) | `@c4 component` (planned) | `@c4 uses target "label" "protocol"` (planned) |

\* Python adapter not yet implemented

The `@c4` syntax provides a consistent pattern across all language adapters.

## Conclusion

The `@c4` syntax is now the universal standard for archidoc annotations. The `<<>>` syntax has been removed with no backwards compatibility. This decision provides:

- **Consistency**: All language adapters use identical syntax
- **No warnings**: No rustdoc HTML tag warnings
- **Simplicity**: Single canonical syntax eliminates confusion
- **Better tooling**: Modern editors can provide better autocomplete for `@c4` tags

Projects using the old `<<>>` syntax must migrate to `@c4` to work with archidoc v0.2.0 and later.

## References

- [Rust RFC 1946](https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html) — intra-doc link syntax
- [rustdoc HTML tag validation](https://doc.rust-lang.org/rustdoc/lints.html#invalid_html_tags)
- [C4-PlantUML notation](https://github.com/plantuml-stdlib/C4-PlantUML) — origin of `<<container>>` syntax
- [TypeScript JSDoc tags](https://www.typescriptlang.org/docs/handbook/jsdoc-supported-types.html)
