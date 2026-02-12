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
