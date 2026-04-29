/**
 * @c4 component
 *
 * Walks a directory tree finding _index.md annotation files.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `walker.ts` | -- | Directory traversal for _index.md files | stable |
 */

import * as fs from "node:fs";
import * as path from "node:path";

import * as parser from "./parser.js";
import { pathToModuleName } from "./path-resolver.js";
import type { ModuleDoc } from "./types.js";

const SKIP_DIRS = new Set([
  "node_modules",
  ".git",
  "target",
  "dist",
  ".ragd",
  ".vite",
  ".claude",
]);

/**
 * Walk a documentation tree and extract ModuleDocs from all _index.md files.
 *
 * Finds `_index.md` files, extracts HTML comment @c4 annotations, and builds
 * ModuleDoc structs from the parsed content.
 *
 * The root-level _index.md (if present) is skipped — it has no meaningful
 * module_path and describes the repo itself, not a sub-module.
 */
export function extractAllDocs(root: string): ModuleDoc[] {
  const docs: ModuleDoc[] = [];
  root = path.resolve(root);

  walkDir(root, (filePath) => {
    if (path.basename(filePath) !== "_index.md") return;

    let content: string;
    try {
      content = fs.readFileSync(filePath, "utf-8");
    } catch {
      return;
    }

    const modulePath = pathToModuleName(filePath, root);
    if (!modulePath) return;

    const c4Level = parser.extractC4Level(content);
    const description = parser.extractDescription(content);
    const parentContainer = parser.extractParentContainer(modulePath);
    const relationships = parser.extractRelationships(content);
    const files = parser.extractFileTable(content);

    docs.push({
      module_path: modulePath,
      content,
      source_file: filePath,
      c4_level: c4Level,
      pattern: "--",
      pattern_status: "planned",
      description,
      parent_container: parentContainer,
      relationships,
      files,
    });
  });

  docs.sort((a, b) => a.module_path.localeCompare(b.module_path));
  return docs;
}

function walkDir(dir: string, callback: (filePath: string) => void): void {
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }

  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue;

    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkDir(fullPath, callback);
    } else if (entry.isFile()) {
      callback(fullPath);
    }
  }
}
