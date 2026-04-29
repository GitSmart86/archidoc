/**
 * @c4 component
 *
 * Converts _index.md file paths to dot-notation module paths.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `path-resolver.ts` | -- | File path to module path conversion | stable |
 */

import * as path from "node:path";

/**
 * Convert an _index.md file path to a dot-notation module path.
 *
 * The _index.md at the root of the scanned directory is excluded
 * (it would produce an empty string, meaning "the repo itself").
 *
 * Examples:
 * - `root/terrain/_index.md` relative to `root/` -> `terrain`
 * - `root/terrain/TOI/_index.md` relative to `root/` -> `terrain.TOI`
 * - `root/_index.md` relative to `root/` -> null (root-level, excluded)
 */
export function pathToModuleName(filePath: string, root: string): string | null {
  const relative = path.relative(root, filePath);
  const dir = path.dirname(relative);

  const parts = dir
    .split(path.sep)
    .filter((p) => p && p !== ".");

  if (parts.length === 0) return null;

  return parts.join(".");
}
