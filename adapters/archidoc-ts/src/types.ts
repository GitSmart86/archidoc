/**
 * @c4 component
 *
 * Shared types matching the ArchitectureIR v2.0 JSON schema.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `types.ts` | -- | IR type definitions | stable |
 */

export type C4Level = "container" | "component" | "unknown";
export type PatternStatus = "planned" | "verified";
export type HealthStatus = "planned" | "active" | "stable";

/** Top-level IR document — a nested directory tree. */
export interface ArchitectureIR {
  version: string;
  scan_root: string;
  root: DirNode;
}

/** A directory node carrying optional strategy fields from @c4 annotations. */
export interface DirNode {
  name: string;
  path: string;
  c4_level?: C4Level;
  description?: string;
  pattern?: string;
  pattern_status?: PatternStatus;
  content?: string;
  source_file?: string;
  parent?: string;
  relationships?: Relationship[];
  dirs?: DirNode[];
  files?: FileNode[];
}

/** A file node carrying optional attributes from file tables. */
export interface FileNode {
  name: string;
  pattern?: string;
  pattern_status?: PatternStatus;
  purpose?: string;
  health?: HealthStatus;
  extra?: Record<string, string>;
}

/** A runtime dependency between directories. */
export interface Relationship {
  target: string;
  label: string;
  protocol: string;
}

// ---------------------------------------------------------------------------
// Parser-internal types (used by parser.ts, not in the IR output)
// ---------------------------------------------------------------------------

/** Raw file table entry as parsed from JSDoc markdown. */
export interface ParsedFileEntry {
  name: string;
  pattern: string;
  pattern_status: PatternStatus;
  purpose: string;
  health: HealthStatus;
  extra?: Record<string, string>;
}
