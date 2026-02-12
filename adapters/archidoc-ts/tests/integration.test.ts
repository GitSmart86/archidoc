import { describe, it, expect } from "vitest";
import * as path from "node:path";
import * as fs from "node:fs";
import { extractAllDocs } from "../src/walker.js";
import type { ModuleDoc } from "../src/types.js";

describe("JSON IR output", () => {
  const fixturesDir = path.resolve(import.meta.dirname, "fixtures");

  it("produces valid JSON matching IR schema structure", () => {
    const docs = extractAllDocs(fixturesDir);
    const json = JSON.stringify(docs, null, 2);
    const parsed: ModuleDoc[] = JSON.parse(json);

    expect(Array.isArray(parsed)).toBe(true);
    for (const doc of parsed) {
      // Required fields exist
      expect(doc).toHaveProperty("module_path");
      expect(doc).toHaveProperty("content");
      expect(doc).toHaveProperty("source_file");
      expect(doc).toHaveProperty("c4_level");
      expect(doc).toHaveProperty("pattern");
      expect(doc).toHaveProperty("pattern_status");
      expect(doc).toHaveProperty("description");
      expect(doc).toHaveProperty("parent_container");
      expect(doc).toHaveProperty("relationships");
      expect(doc).toHaveProperty("files");

      // Enum constraints
      expect(["container", "component", "unknown"]).toContain(doc.c4_level);
      expect(["planned", "verified"]).toContain(doc.pattern_status);

      for (const rel of doc.relationships) {
        expect(rel).toHaveProperty("target");
        expect(rel).toHaveProperty("label");
        expect(rel).toHaveProperty("protocol");
      }

      for (const file of doc.files) {
        expect(file).toHaveProperty("name");
        expect(file).toHaveProperty("pattern");
        expect(file).toHaveProperty("pattern_status");
        expect(file).toHaveProperty("purpose");
        expect(file).toHaveProperty("health");
        expect(["planned", "verified"]).toContain(file.pattern_status);
        expect(["planned", "active", "stable"]).toContain(file.health);
      }
    }
  });

  it("round-trips through JSON serialization", () => {
    const docs = extractAllDocs(fixturesDir);
    const json = JSON.stringify(docs);
    const parsed: ModuleDoc[] = JSON.parse(json);
    expect(parsed).toEqual(docs);
  });

  it("produces consistent output across runs", () => {
    const run1 = extractAllDocs(fixturesDir);
    const run2 = extractAllDocs(fixturesDir);
    expect(JSON.stringify(run1)).toBe(JSON.stringify(run2));
  });
});
