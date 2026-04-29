#!/usr/bin/env node

/**
 * @c4 container
 *
 * Markdown adapter for archidoc. Parses @c4 HTML comment annotations
 * from _index.md files and emits JSON IR to stdout.
 *
 * Annotation format (invisible in rendered markdown):
 *
 * ```markdown
 * <!-- @c4 container -->
 *
 * One paragraph description of what this directory contains.
 *
 * <!-- @c4 uses some.other.module "label" "convention" -->
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `file.md` | -- | What this file does | stable |
 * ```
 *
 * @c4 uses archidoc_core "JSON IR" "stdout"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `index.ts` | Facade | CLI entry point | stable |
 * | `types.ts` | -- | IR type definitions | stable |
 * | `parser.ts` | -- | HTML comment annotation extraction | stable |
 * | `walker.ts` | -- | Directory traversal for _index.md files | stable |
 * | `path-resolver.ts` | -- | File path to module path conversion | stable |
 */

import * as fs from "node:fs";
import { extractAllDocs } from "./walker.js";

function main(): void {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    console.log(`archidoc-md — Markdown adapter for archidoc

Usage: archidoc-md <root-dir>

Walks <root-dir> finding _index.md files with @c4 HTML comment annotations.
Emits ModuleDoc[] JSON IR to stdout.

Annotation format (markers are invisible in rendered markdown):

  <!-- @c4 container -->

  One paragraph description of what this directory contains.

  <!-- @c4 uses some.other.module "label" "convention" -->

  | File | Pattern | Purpose | Health |
  |------|---------|---------|--------|
  | \`file.md\` | -- | What this file does | stable |

Pipe output to archidoc core:
  archidoc-md ./docs | archidoc --from-json

Merge with Rust IR:
  archidoc . --emit-ir > rust-ir.json
  archidoc-md ./docs > md-ir.json
  archidoc --merge-ir --from-json-file rust-ir.json --from-json-file md-ir.json .`);
    process.exit(0);
  }

  const root = args[0];
  if (!root) {
    console.error("Usage: archidoc-md <root-dir>");
    process.exit(1);
  }

  if (!fs.existsSync(root)) {
    console.error(`Error: directory not found: ${root}`);
    process.exit(1);
  }

  const docs = extractAllDocs(root);
  console.log(JSON.stringify(docs, null, 2));
}

main();
