import { describe, it, expect } from "vitest";
import * as path from "node:path";
import { extractAllDocs } from "../src/walker.js";
import { pathToModuleName } from "../src/path-resolver.js";

describe("pathToModuleName", () => {
  it("converts simple directory to module name", () => {
    expect(pathToModuleName("src/dashboard/index.ts", "src")).toBe("dashboard");
  });

  it("converts nested path to dot notation", () => {
    expect(
      pathToModuleName("src/dashboard/charts/index.ts", "src")
    ).toBe("dashboard.charts");
  });

  it("handles deeply nested paths", () => {
    expect(
      pathToModuleName("src/a/b/c/index.ts", "src")
    ).toBe("a.b.c");
  });
});

describe("extractAllDocs", () => {
  const fixturesDir = path.resolve(import.meta.dirname, "fixtures");

  it("finds all index.ts files with C4 annotations", () => {
    const docs = extractAllDocs(fixturesDir);
    expect(docs).toHaveLength(2);
  });

  it("returns docs sorted by module path", () => {
    const docs = extractAllDocs(fixturesDir);
    expect(docs[0].module_path).toBe("dashboard");
    expect(docs[1].module_path).toBe("dashboard.charts");
  });

  it("parses container correctly", () => {
    const docs = extractAllDocs(fixturesDir);
    const dashboard = docs[0];
    expect(dashboard.c4_level).toBe("container");
    expect(dashboard.description).toBe(
      "Real-time trading dashboard with WebGL charts and streaming data."
    );
    expect(dashboard.parent_container).toBeNull();
    expect(dashboard.relationships).toHaveLength(2);
    expect(dashboard.files).toHaveLength(3);
  });

  it("parses component with parent container", () => {
    const docs = extractAllDocs(fixturesDir);
    const charts = docs[1];
    expect(charts.c4_level).toBe("component");
    expect(charts.parent_container).toBe("dashboard");
    expect(charts.relationships).toHaveLength(2);
  });

  it("returns empty array for directory with no annotations", () => {
    // Use dist/ directory which has .js files but no annotated index.ts
    const docs = extractAllDocs(path.resolve(import.meta.dirname, "..", "dist"));
    expect(docs).toEqual([]);
  });
});
