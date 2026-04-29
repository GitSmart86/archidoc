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

type ColumnKind = "name" | "pattern" | "purpose" | "health" | "extra";

/** Known file table column names and the field they populate. */
const KNOWN_FILE_COLUMNS: Array<[string, ColumnKind]> = [
  ["file", "name"],
  ["name", "name"],
  ["pattern", "pattern"],
  ["purpose", "purpose"],
  ["description", "purpose"],
  ["health", "health"],
  ["status", "health"],
];

function classifyColumn(col: string): ColumnKind {
  const lower = col.trim().toLowerCase();
  const found = KNOWN_FILE_COLUMNS.find(([name]) => name === lower);
  return found ? found[1] : "extra";
}

/**
 * Parse the markdown file table into FileEntry structs.
 *
 * Detects the header row by the presence of a "File" or "Name" column.
 * Column order is driven by the header, not hardcoded positions.
 * Unknown column names are stored in `FileEntry.extra`.
 */
export function extractFileTable(content: string): FileEntry[] {
  const entries: FileEntry[] = [];
  let colKinds: ColumnKind[] = [];
  let colNames: string[] = [];
  let inTable = false;
  let headerSeen = false;

  for (const line of content.split("\n")) {
    const trimmed = line.trim();

    if (!inTable) {
      if (trimmed.startsWith("|")) {
        const headerCells = trimmed
          .split("|")
          .filter((s) => s.trim())
          .map((s) => s.trim());
        const hasFileCol = headerCells.some((c) => {
          const lower = c.toLowerCase();
          return lower === "file" || lower === "name";
        });
        if (hasFileCol) {
          colKinds = headerCells.map(classifyColumn);
          colNames = headerCells.map((c) => c.trim().toLowerCase());
          inTable = true;
          continue;
        }
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

      let name = "";
      let pattern = "--";
      let patternStatus: PatternStatus = "planned";
      let purpose = "";
      let health: HealthStatus = "planned";
      const extra: Record<string, string> = {};

      cells.forEach((cell, i) => {
        const kind = colKinds[i];
        const colName = colNames[i];
        switch (kind) {
          case "name":
            name = cell.replace(/`/g, "").trim();
            break;
          case "pattern": {
            const [p, ps] = parsePatternField(cell);
            pattern = p;
            patternStatus = ps;
            break;
          }
          case "purpose":
            purpose = cell.trim();
            break;
          case "health":
            health = parseHealth(cell);
            break;
          default:
            if (colName) extra[colName] = cell.trim();
        }
      });

      if (name) {
        entries.push({
          name,
          pattern,
          pattern_status: patternStatus,
          purpose,
          health,
          ...(Object.keys(extra).length > 0 ? { extra } : {}),
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
