#!/usr/bin/env node

/**
 * @c4 container
 *
 * TypeScript language adapter for archidoc. Scans index.ts files for @c4
 * JSDoc annotations and emits ArchitectureIR v2.0 JSON to stdout.
 *
 * @c4 uses archidoc_core "ArchitectureIR v2.0 JSON" "stdout"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `index.ts` | Facade | CLI entry point | stable |
 * | `types.ts` | -- | IR type definitions | stable |
 * | `parser.ts` | -- | JSDoc annotation extraction | stable |
 * | `walker.ts` | -- | Directory traversal + IR builder | stable |
 * | `path-resolver.ts` | -- | File path to relative path | stable |
 */

import * as fs from "node:fs";
import { extractIR } from "./walker.js";
import type { ArchitectureIR } from "./types.js";

/**
 * Serialize an ArchitectureIR to compact JSON, omitting undefined and
 * empty arrays to match Rust's skip_serializing_if behavior.
 */
function serializeIR(ir: ArchitectureIR): string {
  return JSON.stringify(
    ir,
    (_key, value) => {
      if (value === undefined) return undefined;
      if (Array.isArray(value) && value.length === 0) return undefined;
      return value;
    },
    2
  );
}

function main(): void {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    console.log(`archidoc-ts — TypeScript adapter for archidoc

Scans a directory for index.ts files with @c4 JSDoc annotations.
Emits ArchitectureIR v2.0 JSON to stdout.

Usage: archidoc-ts <root-dir>

Examples:
  archidoc-ts ./src > _context/current.json
  archidoc render md _context/current.json
  archidoc render context _context/current.json
  archidoc validate _context/architecture.json _context/current.json

Output: ArchitectureIR v2.0 — a nested directory tree where each node
carries structure (dirs + files), strategy (C4 level, pattern, description),
and health (per-file maturity status).`);
    process.exit(0);
  }

  if (args.includes("--version") || args.includes("-V")) {
    console.log("archidoc-ts 0.3.4");
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

  const ir = extractIR(root);
  console.log(serializeIR(ir));
}

main();
