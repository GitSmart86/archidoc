# How to Write a Language Adapter

A language adapter scans source files in a specific programming language and produces JSON IR conforming to the `ModuleDoc[]` schema. The core generator then consumes this IR to produce documentation, diagrams, and exports — regardless of which language produced it.

## The Contract

Your adapter must output a JSON array of `ModuleDoc` objects to stdout. The schema is defined in `core/spec/archidoc-ir-schema.json`.

Each `ModuleDoc` represents one architectural element (a C4 container or component) discovered in the source tree.

## Required Fields

```json
{
  "module_path": "bus.calc",
  "content": "@c4 component\n\n# Calc\n\nIndicator calculations.\n\nGoF: Strategy\n\n| File | Pattern | Purpose | Health |\n|------|---------|---------|--------|\n| `executor.rs` | -- | Order execution | active |",
  "source_file": "src/bus/calc/mod.rs",
  "c4_level": "component",
  "pattern": "Strategy",
  "pattern_status": "planned",
  "description": "Indicator calculations",
  "parent_container": "bus",
  "relationships": [
    {"target": "bus.lanes", "label": "Calculation results", "protocol": "channel"}
  ],
  "files": [
    {"name": "executor.rs", "pattern": "--", "pattern_status": "planned", "purpose": "Order execution", "health": "active"}
  ]
}
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `module_path` | string | Dot-notation path derived from directory hierarchy |
| `content` | string | Raw annotation text extracted from doc comments |
| `source_file` | string | Path to the source file (relative or absolute) |
| `c4_level` | `"container"` \| `"component"` \| `"unknown"` | C4 architecture level |
| `pattern` | string | GoF design pattern name, or `"--"` if none |
| `pattern_status` | `"planned"` \| `"verified"` | Whether the pattern is structurally confirmed |
| `description` | string | First paragraph after the C4 marker |
| `parent_container` | string \| null | Module path of the parent container, null for top-level |
| `relationships` | array | Dependencies declared via `@c4 uses` markers |
| `files` | array | File catalog entries from the module's file table |

### Enum Constraints

- `c4_level`: must be exactly `"container"`, `"component"`, or `"unknown"`
- `pattern_status`: must be exactly `"planned"` or `"verified"`
- `health` (in FileEntry): must be exactly `"planned"`, `"active"`, or `"stable"`

## Adapter Responsibilities

Your adapter must:

1. **Walk the source tree** to find module entry files (language-specific convention)
2. **Extract structured documentation** from those files
3. **Parse C4 markers**: `@c4 container`, `@c4 component`
4. **Parse relationships**: `@c4 uses target "label" "protocol"`
5. **Parse file tables**: `| File | Pattern | Purpose | Health |`
6. **Derive module paths** from directory hierarchy (dot notation)
7. **Derive parent containers** from module path nesting
8. **Output JSON** to stdout

## Module Entry Files by Language

| Language | Entry File | Comment Style |
|----------|-----------|---------------|
| Rust | `mod.rs`, `lib.rs` | `//!` doc comments |
| TypeScript | `index.ts` | `/** @c4 ... */` JSDoc |
| Python | `__init__.py` | `"""` module docstrings |

## Minimal Example: Python Adapter

```python
#!/usr/bin/env python3
"""Minimal Python language adapter for archidoc."""
import json, os, re, sys

def walk_modules(root):
    docs = []
    for dirpath, _, filenames in os.walk(root):
        if "__init__.py" not in filenames:
            continue
        path = os.path.join(dirpath, "__init__.py")
        with open(path) as f:
            content = f.read()

        # Skip files without C4 markers
        if "@c4 container" not in content and "@c4 component" not in content:
            continue

        rel_path = os.path.relpath(dirpath, root)
        module_path = rel_path.replace(os.sep, ".")

        # Parse C4 level
        if "@c4 container" in content:
            c4_level = "container"
        elif "@c4 component" in content:
            c4_level = "component"
        else:
            c4_level = "unknown"

        # Derive parent from module path
        parts = module_path.split(".")
        parent = ".".join(parts[:-1]) if len(parts) > 1 else None

        # Extract description (first non-marker paragraph)
        lines = content.strip().split("\n")
        description = ""
        for line in lines:
            stripped = line.strip().strip('"').strip("'")
            if stripped and "@c4" not in stripped and not stripped.startswith("#"):
                description = stripped
                break

        docs.append({
            "module_path": module_path,
            "content": content,
            "source_file": path,
            "c4_level": c4_level,
            "pattern": "",
            "pattern_status": "planned",
            "description": description,
            "parent_container": parent,
            "relationships": [],
            "files": []
        })

    docs.sort(key=lambda d: d["module_path"])
    return docs

if __name__ == "__main__":
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    print(json.dumps(walk_modules(root), indent=2))
```

## Using Your Adapter

```bash
# Emit IR and validate
python archidoc-py.py ./src > ir.json
archidoc --from-json-file ir.json --validate-ir

# Generate ARCHITECTURE.md from IR
archidoc --from-json-file ir.json .

# Pipe directly (Unix)
python archidoc-py.py ./src | archidoc --from-json .

# Combine adapters for polyglot projects
archidoc --emit-ir ./backend/src > backend-ir.json
python archidoc-py.py ./services > services-ir.json
archidoc --merge-ir --from-json-file backend-ir.json --from-json-file services-ir.json .
```

## Validating Your Adapter

1. Run your adapter and capture the output:
   ```bash
   your-adapter ./src > ir.json
   ```

2. Validate against the schema:
   ```bash
   archidoc --from-json-file ir.json --validate-ir
   ```

3. Generate ARCHITECTURE.md and inspect:
   ```bash
   archidoc --from-json-file ir.json .
   cat ARCHITECTURE.md
   ```

4. Check that the inline Mermaid diagrams render correctly.

## Reference Implementation

The Rust adapter in `adapters/archidoc-rust/` is the reference:

| File | Purpose |
|------|---------|
| `walker.rs` | Directory traversal — finds `mod.rs`/`lib.rs` |
| `parser.rs` | Annotation extraction — C4 markers, file tables, relationships |
| `path_resolver.rs` | File path → dot-notation module path conversion |
