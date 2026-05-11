/**
 * @c4 component
 *
 * Walks a directory tree, builds a DirNode tree, and overlays @c4 annotations.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `walker.ts` | -- | Directory traversal + IR builder | stable |
 */

import * as fs from "node:fs";
import * as path from "node:path";

import * as parser from "./parser.js";
import { pathToModuleName } from "./path-resolver.js";
import type { ArchitectureIR, DirNode, FileNode } from "./types.js";

/** Directories to exclude from the tree scan. */
const EXCLUDE_DIRS = new Set([
  "node_modules",
  ".git",
  "dist",
  "build",
  ".archidoc",
  "__pycache__",
  "coverage",
  ".next",
  ".turbo",
]);

/**
 * Scan a source tree and produce an ArchitectureIR v2.0.
 *
 * Three phases:
 * 1. Build bare tree (structure only — every dir and file)
 * 2. Overlay annotations (find index.ts with @c4 JSDoc, set strategy fields)
 * 3. Resolve parents (set nearest annotated ancestor on each annotated node)
 */
export function extractIR(root: string): ArchitectureIR {
  const absRoot = path.resolve(root);

  const rootNode = buildTree(absRoot, absRoot);
  overlayAnnotations(absRoot, absRoot, rootNode);
  resolveParents(rootNode, undefined);

  return {
    version: "2.0",
    scan_root: absRoot,
    root: rootNode,
  };
}

// ---------------------------------------------------------------------------
// Phase 1: Build bare tree
// ---------------------------------------------------------------------------

function buildTree(root: string, dir: string): DirNode {
  const name = dir === root ? "." : path.basename(dir);
  const relPath =
    dir === root ? "." : path.relative(root, dir).replace(/\\/g, "/");

  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return { name, path: relPath };
  }

  const dirs: DirNode[] = entries
    .filter((e) => e.isDirectory() && !EXCLUDE_DIRS.has(e.name))
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((e) => buildTree(root, path.join(dir, e.name)));

  const files: FileNode[] = entries
    .filter((e) => e.isFile())
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((e) => ({ name: e.name }));

  return { name, path: relPath, dirs, files };
}

// ---------------------------------------------------------------------------
// Phase 2: Overlay annotations
// ---------------------------------------------------------------------------

function overlayAnnotations(
  root: string,
  dir: string,
  node: DirNode
): void {
  const indexPath = path.join(dir, "index.ts");

  if (fs.existsSync(indexPath)) {
    let content: string;
    try {
      content = fs.readFileSync(indexPath, "utf-8");
    } catch {
      content = "";
    }

    if (content) {
      const jsDoc = parser.extractJsDoc(content);
      if (jsDoc) {
        const c4Level = parser.extractC4Level(jsDoc);
        if (c4Level !== "unknown") {
          // Set strategy fields
          node.c4_level = c4Level;
          node.description = parser.extractDescription(jsDoc);

          const pat = parser.extractPattern(jsDoc);
          if (pat !== "--") node.pattern = pat;
          node.pattern_status = parser.extractPatternStatus(jsDoc);
          node.content = jsDoc;
          node.source_file = path
            .relative(root, indexPath)
            .replace(/\\/g, "/");

          // Relationships: explicit @c4 uses + auto-discovered imports
          const modulePath = pathToModuleName(indexPath, root);
          const explicitRels = parser.extractRelationships(jsDoc);
          const importRels = parser.extractImportRelationships(
            content,
            modulePath
          );
          const merged = parser.mergeRelationships(explicitRels, importRels);
          if (merged.length > 0) {
            node.relationships = merged;
          }

          // Overlay file table entries onto existing FileNodes
          const fileEntries = parser.extractFileTable(jsDoc);
          for (const entry of fileEntries) {
            const existing = node.files?.find((f) => f.name === entry.name);
            if (existing) {
              if (entry.pattern !== "--") existing.pattern = entry.pattern;
              existing.pattern_status = entry.pattern_status;
              if (entry.purpose) existing.purpose = entry.purpose;
              existing.health = entry.health;
              if (entry.extra && Object.keys(entry.extra).length > 0) {
                existing.extra = entry.extra;
              }
            } else {
              // File in annotation but not on disk — still record it
              const fileNode: FileNode = { name: entry.name };
              if (entry.pattern !== "--") fileNode.pattern = entry.pattern;
              fileNode.pattern_status = entry.pattern_status;
              if (entry.purpose) fileNode.purpose = entry.purpose;
              fileNode.health = entry.health;
              if (entry.extra && Object.keys(entry.extra).length > 0) {
                fileNode.extra = entry.extra;
              }
              if (!node.files) node.files = [];
              node.files.push(fileNode);
            }
          }
        }
      }
    }
  }

  // Recurse into child dirs
  for (const child of node.dirs ?? []) {
    overlayAnnotations(root, path.join(dir, child.name), child);
  }
}

// ---------------------------------------------------------------------------
// Phase 3: Resolve parents
// ---------------------------------------------------------------------------

function resolveParents(
  node: DirNode,
  nearestAnnotatedAncestor: string | undefined
): void {
  let nextAncestor = nearestAnnotatedAncestor;

  if (node.c4_level !== undefined) {
    if (nearestAnnotatedAncestor !== undefined) {
      node.parent = nearestAnnotatedAncestor;
    }
    nextAncestor = node.path;
  }

  for (const child of node.dirs ?? []) {
    resolveParents(child, nextAncestor);
  }
}
