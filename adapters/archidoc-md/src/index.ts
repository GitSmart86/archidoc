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
import * as path from "node:path";
import { extractAllDocs } from "./walker.js";

// ---------------------------------------------------------------------------
// Suggest subcommand helpers
// ---------------------------------------------------------------------------

const DEFAULT_SUGGEST_TEMPLATE = `<!-- @c4 {{c4_level}} -->

[TODO: describe this directory's responsibility]

| File | Purpose | Health |
|------|---------|--------|
{{file_rows}}`;

/**
 * Load the custom suggest template from `.archidoc/custom/_index.md` in cwd.
 * Returns null if the file does not exist (caller uses hardcoded default).
 */
function loadCustomSuggestTemplate(cwd: string): string | null {
  const customPath = path.join(cwd, ".archidoc", "custom", "_index.md");
  try {
    return fs.readFileSync(customPath, "utf-8");
  } catch {
    return null;
  }
}

/**
 * Substitute `{{key}}` tokens in a template string.
 */
function substitute(template: string, vars: Record<string, string>): string {
  let result = template;
  for (const [key, value] of Object.entries(vars)) {
    result = result.replaceAll(`{{${key}}}`, value);
  }
  return result;
}

/**
 * Scan a directory for notable items: markdown files and subdirectories.
 * Excludes `_index.md` itself and hidden/system directories.
 */
function scanDirectory(dir: string): string[] {
  const SKIP = new Set(["node_modules", ".git", "target", "dist", ".ragd", ".vite", ".claude"]);
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }

  const items: string[] = [];
  for (const entry of entries) {
    if (SKIP.has(entry.name)) continue;
    if (entry.name === "_index.md") continue;
    if (entry.name.startsWith(".")) continue;

    if (entry.isDirectory()) {
      items.push(`${entry.name}/`);
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      items.push(entry.name);
    }
  }
  return items.sort();
}

/**
 * Infer C4 level from directory depth relative to the scan root.
 * Depth 1 from root = container, deeper = component.
 */
function inferC4Level(dir: string, cwd: string): string {
  const rel = path.relative(cwd, dir);
  const depth = rel === "" ? 0 : rel.split(path.sep).length;
  return depth <= 1 ? "container" : "component";
}

/**
 * Run the suggest subcommand for a markdown directory.
 */
function runSuggest(dir: string): void {
  if (!fs.existsSync(dir)) {
    console.error(`Error: directory not found: ${dir}`);
    process.exit(1);
  }
  if (!fs.statSync(dir).isDirectory()) {
    console.error(`Error: path is not a directory: ${dir}`);
    process.exit(1);
  }

  const cwd = process.cwd();
  const template = loadCustomSuggestTemplate(cwd) ?? DEFAULT_SUGGEST_TEMPLATE;

  const dirName = path.basename(dir);
  const c4Level = inferC4Level(path.resolve(dir), cwd);
  const items = scanDirectory(dir);

  const fileRows = items
    .map((item) => `| \`${item}\` | [TODO] | active |`)
    .join("\n");

  const output = substitute(template, {
    module_name: dirName,
    c4_level: c4Level,
    file_rows: fileRows,
  });

  process.stdout.write(output + "\n");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main(): void {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    console.log(`archidoc-md — Markdown adapter for archidoc

Usage:
  archidoc-md <root-dir>           Walk root-dir and emit ModuleDoc[] JSON IR
  archidoc-md suggest <dir>        Generate _index.md template for a directory

Annotation format (markers are invisible in rendered markdown):

  <!-- @c4 container -->

  One paragraph description of what this directory contains.

  <!-- @c4 uses some.other.module "label" "convention" -->

  | File | Purpose | Health |
  |------|---------|--------|
  | \`file.md\` | What this file does | stable |

Custom suggest template:
  Place a custom template at .archidoc/custom/_index.md
  Supported tokens: {{module_name}}, {{c4_level}}, {{file_rows}}
  Falls back to built-in default if file does not exist.

Merge with Rust IR:
  archidoc . --emit-ir > rust-ir.json
  archidoc-md ./docs > md-ir.json
  archidoc --merge-ir --from-json-file rust-ir.json --from-json-file md-ir.json .`);
    process.exit(0);
  }

  // suggest subcommand
  if (args[0] === "suggest") {
    const dir = args[1];
    if (!dir) {
      console.error("Usage: archidoc-md suggest <dir>");
      process.exit(1);
    }
    runSuggest(dir);
    return;
  }

  // default: emit IR
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
