import { describe, it, expect } from "vitest";
import {
  extractImportRelationships,
  mergeRelationships,
} from "../src/parser.js";

describe("extractImportRelationships", () => {
  it("extracts import from parent directory", () => {
    const content = `import { EventBus } from "../events/index.js";`;
    const rels = extractImportRelationships(content, "dashboard.charts");

    expect(rels).toHaveLength(1);
    expect(rels[0]).toEqual({
      target: "dashboard.events",
      label: "imports",
      protocol: "ES module",
    });
  });

  it("ignores imports from subdirectory (internal to module)", () => {
    const content = `import { Config } from "./config/index.js";`;
    const rels = extractImportRelationships(content, "core");

    // Subdirectories are part of the module's internal structure
    expect(rels).toHaveLength(0);
  });

  it("extracts import from sibling module", () => {
    const content = `import { Auth } from "../auth/index.js";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("auth");
  });

  it("extracts import going up multiple levels", () => {
    const content = `import { Core } from "../../core/index.js";`;
    const rels = extractImportRelationships(content, "dashboard.charts.axis");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("dashboard.core");
  });

  it("extracts import to top-level from nested module", () => {
    const content = `import { Events } from "../../../events/index.js";`;
    const rels = extractImportRelationships(content, "dashboard.charts.axis");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("events");
  });

  it("handles import without file extension", () => {
    const content = `import { Utils } from "../utils";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("utils");
  });

  it("handles export re-export syntax", () => {
    const content = `export { Something } from "../shared/index.js";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0]).toEqual({
      target: "shared",
      label: "imports",
      protocol: "ES module",
    });
  });

  it("handles namespace imports", () => {
    const content = `import * as Events from "../events/index.js";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("events");
  });

  it("handles default imports", () => {
    const content = `import EventBus from "../events/index.js";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("events");
  });

  it("deduplicates multiple imports from same module", () => {
    const content = `
      import { A } from "../events/index.js";
      import { B } from "../events/index.js";
    `;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("events");
  });

  it("extracts multiple imports from different modules", () => {
    const content = `
      import { Auth } from "../auth/index.js";
      import { DB } from "../database/index.js";
      import { Events } from "../events/index.js";
    `;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(3);
    const targets = rels.map((r) => r.target).sort();
    expect(targets).toEqual(["auth", "database", "events"]);
  });

  it("ignores node_modules imports and internal files", () => {
    const content = `
      import { express } from "express";
      import { Config } from "./config/index.js";
    `;
    const rels = extractImportRelationships(content, "api");

    // Both node_modules and internal ./config should be filtered
    expect(rels).toHaveLength(0);
  });

  it("ignores internal imports within same module", () => {
    const content = `
      import { Helper } from "./helper.js";
      import { Utils } from "./utils/index.js";
    `;
    const rels = extractImportRelationships(content, "dashboard");

    // Internal files within same module (./) should be filtered
    expect(rels).toHaveLength(0);
  });

  it("handles /index suffix removal", () => {
    const content = `import { X } from "../events/index";`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toHaveLength(1);
    expect(rels[0].target).toBe("events");
  });

  it("returns empty array when no imports", () => {
    const content = `export const foo = 42;`;
    const rels = extractImportRelationships(content, "dashboard");

    expect(rels).toEqual([]);
  });
});

describe("mergeRelationships", () => {
  it("keeps explicit relationships as-is", () => {
    const explicit = [
      { target: "auth", label: "Authentication", protocol: "REST" },
    ];
    const imports = [];

    const merged = mergeRelationships(explicit, imports);
    expect(merged).toEqual(explicit);
  });

  it("adds import relationships when no explicit ones", () => {
    const explicit = [];
    const imports = [
      { target: "events", label: "imports", protocol: "ES module" },
    ];

    const merged = mergeRelationships(explicit, imports);
    expect(merged).toEqual(imports);
  });

  it("explicit takes priority over import for same target", () => {
    const explicit = [
      { target: "auth", label: "User sessions", protocol: "gRPC" },
    ];
    const imports = [
      { target: "auth", label: "imports", protocol: "ES module" },
      { target: "events", label: "imports", protocol: "ES module" },
    ];

    const merged = mergeRelationships(explicit, imports);

    expect(merged).toHaveLength(2);
    expect(merged[0]).toEqual({
      target: "auth",
      label: "User sessions",
      protocol: "gRPC",
    });
    expect(merged[1]).toEqual({
      target: "events",
      label: "imports",
      protocol: "ES module",
    });
  });

  it("merges multiple explicit and import relationships", () => {
    const explicit = [
      { target: "api", label: "Data fetching", protocol: "GraphQL" },
      { target: "db", label: "Persistence", protocol: "PostgreSQL" },
    ];
    const imports = [
      { target: "events", label: "imports", protocol: "ES module" },
      { target: "utils", label: "imports", protocol: "ES module" },
      { target: "api", label: "imports", protocol: "ES module" }, // Should be ignored
    ];

    const merged = mergeRelationships(explicit, imports);

    expect(merged).toHaveLength(4);
    expect(merged.map((r) => r.target).sort()).toEqual([
      "api",
      "db",
      "events",
      "utils",
    ]);

    // Verify explicit wins for 'api'
    const apiRel = merged.find((r) => r.target === "api");
    expect(apiRel?.label).toBe("Data fetching");
  });
});
