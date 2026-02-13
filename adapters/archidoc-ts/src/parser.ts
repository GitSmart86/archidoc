/**
 * @c4 component
 *
 * Parses JSDoc blocks from TypeScript index.ts files and extracts C4 annotations.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `parser.ts` | -- | JSDoc annotation extraction | stable |
 */

import type {
  C4Level,
  FileEntry,
  HealthStatus,
  PatternStatus,
  Relationship,
} from "./types.js";

const GOF_PATTERNS = [
  "Mediator",
  "Observer",
  "Strategy",
  "Facade",
  "Adapter",
  "Repository",
  "Singleton",
  "Factory",
  "Builder",
  "Decorator",
  "Active Object",
  "Memento",
  "Command",
  "Chain of Responsibility",
  "Registry",
  "Composite",
  "Interpreter",
  "Flyweight",
  "Publisher",
];

/**
 * Extract the first JSDoc block from file content.
 *
 * Returns the content between the opening and closing markers
 * with leading ` * ` prefixes stripped from each line.
 */
export function extractJsDoc(content: string): string | null {
  const match = content.match(/\/\*\*([\s\S]*?)\*\//);
  if (!match) return null;

  const lines = match[1].split(/\r?\n/).map((line) => {
    const trimmed = line.trimStart();
    if (trimmed.startsWith("* ")) return trimmed.slice(2);
    if (trimmed === "*") return "";
    return trimmed;
  });

  const result = lines.join("\n").trim();
  return result || null;
}

/**
 * Extract the C4 level from JSDoc content.
 *
 * Looks for `@c4 container` or `@c4 component` tags.
 */
export function extractC4Level(content: string): C4Level {
  if (/@c4\s+container\b/.test(content)) return "container";
  if (/@c4\s+component\b/.test(content)) return "component";
  return "unknown";
}

/**
 * Extract the primary GoF pattern name from content.
 *
 * Scans for known pattern names. Returns the first match or "--".
 */
export function extractPattern(content: string): string {
  for (const name of GOF_PATTERNS) {
    if (content.includes(name)) return name;
  }
  return "--";
}

/**
 * Extract pattern status from content.
 *
 * Looks for "(verified)" near a pattern name. Defaults to "planned".
 */
export function extractPatternStatus(content: string): PatternStatus {
  return content.includes("(verified)") ? "verified" : "planned";
}

/**
 * Extract the description — first non-empty, non-marker, non-table line.
 */
export function extractDescription(content: string): string {
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (
      trimmed &&
      !trimmed.startsWith("@c4") &&
      !trimmed.startsWith("#") &&
      !trimmed.includes("<<") &&
      !trimmed.startsWith("|") &&
      !trimmed.startsWith("GoF:")
    ) {
      return trimmed;
    }
  }
  return "*No description*";
}

/**
 * Extract the parent container from a dot-notation module path.
 *
 * "dashboard.charts" -> "dashboard"
 * "dashboard" -> null
 */
export function extractParentContainer(modulePath: string): string | null {
  const idx = modulePath.indexOf(".");
  return idx >= 0 ? modulePath.slice(0, idx) : null;
}

/**
 * Parse `@c4 uses target "label" "protocol"` tags from content.
 */
export function extractRelationships(content: string): Relationship[] {
  const rels: Relationship[] = [];
  const re = /@c4\s+uses\s+(\S+)\s+"([^"]+)"\s+"([^"]+)"/g;
  let match;
  while ((match = re.exec(content)) !== null) {
    rels.push({
      target: match[1],
      label: match[2],
      protocol: match[3],
    });
  }
  return rels;
}

/**
 * Extract import relationships from TypeScript import/export statements.
 *
 * Parses `import ... from "path"` and `export ... from "path"` and converts
 * them to relationships. Only processes imports that navigate UP the directory
 * tree (using ..), indicating cross-module dependencies.
 *
 * Internal imports within the same module (./file or ./subdir/file) are ignored.
 *
 * @param content - File content to scan for imports
 * @param currentModulePath - Dot-notation module path (e.g., "dashboard.charts")
 * @returns Array of relationships representing imports
 */
export function extractImportRelationships(
  content: string,
  currentModulePath: string
): Relationship[] {
  const rels: Relationship[] = [];

  // Match both import and export statements with relative paths
  const importRe = /(?:import|export)\s+(?:{[^}]*}|\*\s+as\s+\w+|\w+)\s+from\s+["'](\.[^"']+)["']/g;
  let match;

  while ((match = importRe.exec(content)) !== null) {
    const importPath = match[1];

    // Only process imports that go UP the tree (..)
    // ./file and ./subdir/file are internal to the current module
    if (!importPath.startsWith("..")) {
      continue;
    }

    // Convert relative path to module path
    const targetModule = resolveImportPath(importPath, currentModulePath);

    // Only add if it resolves to a different module
    if (targetModule && targetModule !== currentModulePath) {
      rels.push({
        target: targetModule,
        label: "imports",
        protocol: "ES module",
      });
    }
  }

  // Deduplicate by target
  const seen = new Set<string>();
  return rels.filter((r) => {
    if (seen.has(r.target)) return false;
    seen.add(r.target);
    return true;
  });
}

/**
 * Resolve a relative import path to a module path.
 *
 * Examples:
 * - "../events" from "dashboard.charts" -> "dashboard.events"
 * - "./submodule" from "dashboard" -> "dashboard.submodule"
 * - "../../core" from "dashboard.charts.axis" -> "core"
 *
 * @param importPath - Relative import path from source
 * @param currentModule - Current module in dot notation
 * @returns Resolved module path or null if invalid
 */
function resolveImportPath(importPath: string, currentModule: string): string | null {
  // Remove file extensions and /index suffix
  let cleanPath = importPath
    .replace(/\.(ts|js|tsx|jsx)$/, "")
    .replace(/\/index$/, "");

  // Split current module into parts
  const currentParts = currentModule ? currentModule.split(".") : [];

  // Parse the import path
  const segments = cleanPath.split("/").filter(s => s);

  let workingParts = [...currentParts];

  for (const segment of segments) {
    if (segment === "..") {
      // Go up one level
      if (workingParts.length > 0) {
        workingParts.pop();
      }
    } else if (segment === ".") {
      // Current directory - no change
      continue;
    } else {
      // Add to path
      workingParts.push(segment);
    }
  }

  return workingParts.length > 0 ? workingParts.join(".") : null;
}

/**
 * Merge explicit @c4 uses relationships with auto-discovered import relationships.
 *
 * Explicit relationships take priority - if a target is already declared with
 * @c4 uses, the import relationship is ignored.
 *
 * @param explicitRels - Relationships from @c4 uses tags
 * @param importRels - Relationships from import statements
 * @returns Merged array with no duplicate targets
 */
export function mergeRelationships(
  explicitRels: Relationship[],
  importRels: Relationship[]
): Relationship[] {
  const explicitTargets = new Set(explicitRels.map((r) => r.target));

  // Start with all explicit relationships
  const merged = [...explicitRels];

  // Add import relationships that aren't already declared
  for (const importRel of importRels) {
    if (!explicitTargets.has(importRel.target)) {
      merged.push(importRel);
    }
  }

  return merged;
}

/**
 * Parse the markdown file table into FileEntry structs.
 *
 * Expects format (inside JSDoc, with ` * ` prefix already stripped):
 * ```
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `core.ts` | Facade | Entry point | stable |
 * ```
 */
export function extractFileTable(content: string): FileEntry[] {
  const entries: FileEntry[] = [];
  let inTable = false;
  let headerSeen = false;

  for (const line of content.split("\n")) {
    const trimmed = line.trim();

    if (!inTable) {
      if (
        trimmed.startsWith("|") &&
        /file/i.test(trimmed) &&
        /pattern/i.test(trimmed)
      ) {
        inTable = true;
        continue;
      }
    } else if (!headerSeen) {
      if (trimmed.startsWith("|") && trimmed.includes("---")) {
        headerSeen = true;
        continue;
      }
    } else {
      if (!trimmed.startsWith("|")) break;

      const cells = trimmed
        .split("|")
        .filter((s) => s.trim())
        .map((s) => s.trim());

      if (cells.length >= 4) {
        const filename = cells[0].replace(/`/g, "").trim();
        const [pattern, patternStatus] = parsePatternField(cells[1]);
        const purpose = cells[2].trim();
        const health = parseHealth(cells[3]);

        entries.push({
          name: filename,
          pattern,
          pattern_status: patternStatus,
          purpose,
          health,
        });
      }
    }
  }

  return entries;
}

function parsePatternField(field: string): [string, PatternStatus] {
  const trimmed = field.trim();
  const idx = trimmed.indexOf("(");
  if (idx >= 0) {
    const pattern = trimmed.slice(0, idx).trim();
    const statusStr = trimmed.slice(idx + 1, trimmed.indexOf(")")).trim();
    return [pattern, statusStr === "verified" ? "verified" : "planned"];
  }
  return [trimmed, "planned"];
}

function parseHealth(field: string): HealthStatus {
  const trimmed = field.trim().toLowerCase();
  if (trimmed === "active") return "active";
  if (trimmed === "stable") return "stable";
  return "planned";
}
