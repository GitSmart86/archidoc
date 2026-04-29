/**
 * @c4 component
 *
 * Parses HTML comment @c4 annotations from _index.md files.
 *
 * Annotation format uses HTML comments so markers are invisible
 * in rendered markdown (GitHub, VSCode preview, etc.):
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
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `parser.ts` | -- | HTML comment annotation extraction | stable |
 */

import type {
  C4Level,
  FileEntry,
  HealthStatus,
  PatternStatus,
  Relationship,
} from "./types.js";

/**
 * Extract the C4 level from _index.md content.
 *
 * Looks for `<!-- @c4 container -->` or `<!-- @c4 component -->`.
 */
export function extractC4Level(content: string): C4Level {
  if (/<!--\s*@c4\s+container\s*-->/.test(content)) return "container";
  if (/<!--\s*@c4\s+component\s*-->/.test(content)) return "component";
  return "unknown";
}

/**
 * Extract the description — first non-empty prose paragraph
 * that is not an HTML comment, heading, or table row.
 */
export function extractDescription(content: string): string {
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (
      trimmed &&
      !trimmed.startsWith("<!--") &&
      !trimmed.startsWith("#") &&
      !trimmed.startsWith("|") &&
      !trimmed.startsWith("```") &&
      !trimmed.startsWith(">")
    ) {
      return trimmed;
    }
  }
  return "*No description*";
}

/**
 * Extract the parent container from a dot-notation module path.
 *
 * "terrain.TOI" -> "terrain"
 * "terrain" -> null
 */
export function extractParentContainer(modulePath: string): string | null {
  const idx = modulePath.indexOf(".");
  return idx >= 0 ? modulePath.slice(0, idx) : null;
}

/**
 * Parse `<!-- @c4 uses target "label" "convention" -->` tags from content.
 */
export function extractRelationships(content: string): Relationship[] {
  const rels: Relationship[] = [];
  const re = /<!--\s*@c4\s+uses\s+(\S+)\s+"([^"]+)"\s+"([^"]+)"\s*-->/g;
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
 * Parse the markdown file table into FileEntry structs.
 *
 * Expects standard markdown table format:
 * ```
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `file.md` | -- | Description | stable |
 * ```
 *
 * Pattern column is optional for markdown files and defaults to "--".
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
        /purpose/i.test(trimmed)
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

      if (cells.length >= 3) {
        const filename = cells[0].replace(/`/g, "").trim();

        // Support both 3-column (File | Purpose | Health)
        // and 4-column (File | Pattern | Purpose | Health) tables
        let pattern = "--";
        let patternStatus: PatternStatus = "planned";
        let purpose: string;
        let health: HealthStatus;

        if (cells.length >= 4) {
          [pattern, patternStatus] = parsePatternField(cells[1]);
          purpose = cells[2].trim();
          health = parseHealth(cells[3]);
        } else {
          purpose = cells[1].trim();
          health = parseHealth(cells[2]);
        }

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
