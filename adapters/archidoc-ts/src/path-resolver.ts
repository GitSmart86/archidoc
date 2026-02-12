/**
 * @c4 component
 *
 * Converts file paths to dot-notation module paths.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `path-resolver.ts` | -- | File path to module path conversion | stable |
 */

import * as path from "node:path";

/**
 * Convert an index.ts file path to a dot-notation module path.
 *
 * Examples:
 * - `root/dashboard/index.ts` relative to `root/` -> `dashboard`
 * - `root/dashboard/charts/index.ts` relative to `root/` -> `dashboard.charts`
 */
export function pathToModuleName(filePath: string, root: string): string {
  const relative = path.relative(root, filePath);
  const dir = path.dirname(relative);

  // Convert path separators to dots
  const parts = dir
    .split(path.sep)
    .filter((p) => p && p !== ".");

  return parts.join(".");
}
