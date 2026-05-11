import { describe, it, expect } from "vitest";
import * as path from "node:path";
import { extractIR } from "../src/walker.js";
import type { ArchitectureIR, DirNode } from "../src/types.js";

describe("ArchitectureIR v2.0 output", () => {
  const fixturesDir = path.resolve(import.meta.dirname, "fixtures");

  it("produces valid JSON matching IR v2.0 schema", () => {
    const ir = extractIR(fixturesDir);
    const json = JSON.stringify(ir, null, 2);
    const parsed: ArchitectureIR = JSON.parse(json);

    // Top-level fields
    expect(parsed.version).toBe("2.0");
    expect(parsed.scan_root).toBeTruthy();
    expect(parsed.root).toBeDefined();
    expect(parsed.root.name).toBe(".");
    expect(parsed.root.path).toBe(".");
  });

  it("annotated DirNodes have correct fields", () => {
    const ir = extractIR(fixturesDir);

    function collectAnnotated(node: DirNode): DirNode[] {
      const result: DirNode[] = [];
      if (node.c4_level !== undefined) result.push(node);
      for (const child of node.dirs ?? []) {
        result.push(...collectAnnotated(child));
      }
      return result;
    }

    const annotated = collectAnnotated(ir.root);
    expect(annotated.length).toBeGreaterThan(0);

    for (const dir of annotated) {
      expect(dir.name).toBeTruthy();
      expect(dir.path).toBeTruthy();
      expect(["container", "component"]).toContain(dir.c4_level);
      expect(dir.description).toBeTruthy();
      expect(dir.source_file).toBeTruthy();

      // Relationships if present
      if (dir.relationships) {
        for (const rel of dir.relationships) {
          expect(rel.target).toBeTruthy();
          expect(rel.label).toBeTruthy();
          expect(rel.protocol).toBeTruthy();
        }
      }

      // Files with health if present
      if (dir.files) {
        for (const file of dir.files) {
          expect(file.name).toBeTruthy();
          if (file.health !== undefined) {
            expect(["planned", "active", "stable"]).toContain(file.health);
          }
          if (file.pattern_status !== undefined) {
            expect(["planned", "verified"]).toContain(file.pattern_status);
          }
        }
      }
    }
  });

  it("round-trips through JSON serialization", () => {
    const ir = extractIR(fixturesDir);
    const json = JSON.stringify(ir);
    const parsed: ArchitectureIR = JSON.parse(json);
    expect(parsed).toEqual(ir);
  });

  it("produces consistent output across runs", () => {
    const run1 = extractIR(fixturesDir);
    const run2 = extractIR(fixturesDir);
    expect(JSON.stringify(run1)).toBe(JSON.stringify(run2));
  });

  it("unannotated dirs have no strategy fields", () => {
    const ir = extractIR(fixturesDir);

    function findUnannotated(node: DirNode): DirNode | undefined {
      if (node.c4_level === undefined && node.path !== ".") return node;
      for (const child of node.dirs ?? []) {
        const found = findUnannotated(child);
        if (found) return found;
      }
      return undefined;
    }

    // May not have unannotated dirs in fixtures, but if we do, verify
    const unannotated = findUnannotated(ir.root);
    if (unannotated) {
      expect(unannotated.c4_level).toBeUndefined();
      expect(unannotated.description).toBeUndefined();
      expect(unannotated.content).toBeUndefined();
    }
  });
});
