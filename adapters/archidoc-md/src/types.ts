/**
 * @c4 component
 *
 * Shared types matching the archidoc JSON IR schema.
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `types.ts` | -- | IR type definitions | stable |
 */

export type C4Level = "container" | "component" | "unknown";
export type PatternStatus = "planned" | "verified";
export type HealthStatus = "planned" | "active" | "stable";

export interface Relationship {
  target: string;
  label: string;
  protocol: string;
}

export interface FileEntry {
  name: string;
  pattern: string;
  pattern_status: PatternStatus;
  purpose: string;
  health: HealthStatus;
}

export interface ModuleDoc {
  module_path: string;
  content: string;
  source_file: string;
  c4_level: C4Level;
  pattern: string;
  pattern_status: PatternStatus;
  description: string;
  parent_container: string | null;
  relationships: Relationship[];
  files: FileEntry[];
}
