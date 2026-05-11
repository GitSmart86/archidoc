/**
 * @c4 component
 *
 * Converts file paths to relative slash-separated paths.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `path-resolver.ts` | -- | File path to relative path conversion | stable |
 */

import * as path from "node:path";

/**
 * Convert a file path to a slash-separated path relative to root.
 *
 * Examples:
 * - `root/dashboard/index.ts` relative to `root/` -> `"dashboard"`
 * - `root/dashboard/charts/index.ts` relative to `root/` -> `"dashboard/charts"`
 */
export function pathToRelative(filePath: string, root: string): string {
  const dir = path.dirname(filePath);
  const relative = path.relative(root, dir);
  return relative.replace(/\\/g, "/") || ".";
}

/**
 * Convert a file path to dot-notation module path (for import resolution).
 *
 * Examples:
 * - `root/dashboard/index.ts` relative to `root/` -> `"dashboard"`
 * - `root/dashboard/charts/index.ts` relative to `root/` -> `"dashboard.charts"`
 */
export function pathToModuleName(filePath: string, root: string): string {
  const relative = path.relative(root, filePath);
  const dir = path.dirname(relative);
  const parts = dir.split(path.sep).filter((p) => p && p !== ".");
  return parts.join(".");
}
