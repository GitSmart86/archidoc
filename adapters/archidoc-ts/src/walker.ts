/**
 * @c4 component
 *
 * Walks a directory tree finding index.ts module entry files.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `walker.ts` | -- | Directory traversal for index.ts files | stable |
 */

import * as fs from "node:fs";
import * as path from "node:path";

import * as parser from "./parser.js";
import { pathToModuleName } from "./path-resolver.js";
import type { ModuleDoc } from "./types.js";

/**
 * Walk a source tree and extract ModuleDocs from all index.ts files.
 *
 * Finds `index.ts` files, extracts JSDoc blocks, and builds
 * ModuleDoc structs from the parsed annotations.
 */
export function extractAllDocs(root: string): ModuleDoc[] {
  const docs: ModuleDoc[] = [];
  walkDir(root, (filePath) => {
    if (path.basename(filePath) !== "index.ts") return;

    let content: string;
    try {
      content = fs.readFileSync(filePath, "utf-8");
    } catch {
      return;
    }
    const jsDoc = parser.extractJsDoc(content);
    if (!jsDoc) return;

    const modulePath = pathToModuleName(filePath, root);
    if (!modulePath) return;

    const c4Level = parser.extractC4Level(jsDoc);
    const pattern = parser.extractPattern(jsDoc);
    const patternStatus = parser.extractPatternStatus(jsDoc);
    const description = parser.extractDescription(jsDoc);
    const parentContainer = parser.extractParentContainer(modulePath);
    const relationships = parser.extractRelationships(jsDoc);
    const files = parser.extractFileTable(jsDoc);

    docs.push({
      module_path: modulePath,
      content: jsDoc,
      source_file: filePath,
      c4_level: c4Level,
      pattern,
      pattern_status: patternStatus,
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
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory() && entry.name !== "node_modules" && entry.name !== ".git") {
      walkDir(fullPath, callback);
    } else if (entry.isFile()) {
      callback(fullPath);
    }
  }
}
