import { describe, it, expect } from "vitest";
import * as path from "node:path";
import { extractIR } from "../src/walker.js";
import { pathToRelative, pathToModuleName } from "../src/path-resolver.js";

describe("pathToRelative", () => {
  it("converts simple directory to relative path", () => {
    expect(pathToRelative("src/dashboard/index.ts", "src")).toBe("dashboard");
  });

  it("converts nested path to slash-separated", () => {
    expect(
      pathToRelative("src/dashboard/charts/index.ts", "src")
    ).toBe("dashboard/charts");
  });
});

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

describe("extractIR", () => {
  const fixturesDir = path.resolve(import.meta.dirname, "fixtures");

  it("produces ArchitectureIR v2.0 with correct version", () => {
    const ir = extractIR(fixturesDir);
    expect(ir.version).toBe("2.0");
    expect(ir.scan_root).toBe(fixturesDir);
    expect(ir.root.name).toBe(".");
    expect(ir.root.path).toBe(".");
  });

  it("builds tree with dirs and files", () => {
    const ir = extractIR(fixturesDir);
    // Should have a dashboard dir
    const dashboard = ir.root.dirs?.find((d) => d.name === "dashboard");
    expect(dashboard).toBeDefined();
    expect(dashboard!.path).toBe("dashboard");
  });

  it("overlays annotations on annotated dirs", () => {
    const ir = extractIR(fixturesDir);
    const dashboard = ir.root.dirs?.find((d) => d.name === "dashboard");
    expect(dashboard).toBeDefined();
    expect(dashboard!.c4_level).toBe("container");
    expect(dashboard!.description).toBe(
      "Real-time trading dashboard with WebGL charts and streaming data."
    );
  });

  it("nested component has parent set", () => {
    const ir = extractIR(fixturesDir);
    const dashboard = ir.root.dirs?.find((d) => d.name === "dashboard");
    const charts = dashboard?.dirs?.find((d) => d.name === "charts");
    expect(charts).toBeDefined();
    expect(charts!.c4_level).toBe("component");
    expect(charts!.parent).toBe("dashboard");
  });

  it("parses relationships", () => {
    const ir = extractIR(fixturesDir);
    const dashboard = ir.root.dirs?.find((d) => d.name === "dashboard");
    expect(dashboard!.relationships).toBeDefined();
    expect(dashboard!.relationships!.length).toBeGreaterThanOrEqual(1);
  });

  it("overlays file table entries onto FileNodes", () => {
    const ir = extractIR(fixturesDir);
    const dashboard = ir.root.dirs?.find((d) => d.name === "dashboard");
    // Dashboard should have files with health/purpose from annotation
    const annotatedFiles = dashboard?.files?.filter((f) => f.health !== undefined);
    expect(annotatedFiles).toBeDefined();
    expect(annotatedFiles!.length).toBeGreaterThan(0);
  });

  it("returns tree even for directory with no annotations", () => {
    // dist/ has .js files but no annotated index.ts
    const distDir = path.resolve(import.meta.dirname, "..", "dist");
    const ir = extractIR(distDir);
    expect(ir.version).toBe("2.0");
    expect(ir.root.name).toBe(".");
    // No annotated dirs, but tree exists
    expect(ir.root.c4_level).toBeUndefined();
  });
});
