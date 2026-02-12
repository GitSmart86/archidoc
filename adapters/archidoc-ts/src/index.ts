#!/usr/bin/env node

/**
 * @c4 container
 *
 * TypeScript language adapter for archidoc. Parses @c4 JSDoc annotations
 * from index.ts files and emits JSON IR to stdout.
 *
 * @c4 uses archidoc_core "JSON IR" "stdout"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `index.ts` | Facade | CLI entry point | stable |
 * | `types.ts` | -- | IR type definitions | stable |
 * | `parser.ts` | -- | JSDoc annotation extraction | stable |
 * | `walker.ts` | -- | Directory traversal | stable |
 * | `path-resolver.ts` | -- | File path to module path | stable |
 */

import * as fs from "node:fs";
import { extractAllDocs } from "./walker.js";

function main(): void {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    console.log(`archidoc-ts — TypeScript adapter for archidoc

Usage: archidoc-ts <root-dir>

Walks <root-dir> finding index.ts files with @c4 JSDoc annotations.
Emits ModuleDoc[] JSON IR to stdout.

Pipe output to archidoc core:
  archidoc-ts ./src | archidoc --from-json

Or save and validate:
  archidoc-ts ./src > ir.json
  archidoc --from-json-file ir.json --validate-ir`);
    process.exit(0);
  }

  const root = args[0];
  if (!root) {
    console.error("Usage: archidoc-ts <root-dir>");
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
