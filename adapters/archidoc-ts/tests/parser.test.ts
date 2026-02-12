import { describe, it, expect } from "vitest";
import {
  extractJsDoc,
  extractC4Level,
  extractPattern,
  extractPatternStatus,
  extractDescription,
  extractParentContainer,
  extractRelationships,
  extractFileTable,
} from "../src/parser.js";

describe("extractJsDoc", () => {
  it("extracts JSDoc block with stripped prefixes", () => {
    const input = `/**\n * @c4 container\n *\n * Dashboard for trading.\n */\nexport {};`;
    const result = extractJsDoc(input);
    expect(result).toContain("@c4 container");
    expect(result).toContain("Dashboard for trading.");
    expect(result).not.toContain("* @c4");
  });

  it("returns null when no JSDoc block exists", () => {
    expect(extractJsDoc("const x = 1;")).toBeNull();
  });

  it("handles Windows line endings", () => {
    const input = "/**\r\n * @c4 component\r\n *\r\n * A module.\r\n */\r\n";
    const result = extractJsDoc(input);
    expect(result).toContain("@c4 component");
    expect(result).toContain("A module.");
    expect(result).not.toContain("\r");
  });
});

describe("extractC4Level", () => {
  it("detects container", () => {
    expect(extractC4Level("@c4 container\n\nSome desc")).toBe("container");
  });

  it("detects component", () => {
    expect(extractC4Level("@c4 component\n\nSome desc")).toBe("component");
  });

  it("returns unknown when no marker", () => {
    expect(extractC4Level("Just some text")).toBe("unknown");
  });
});

describe("extractPattern", () => {
  it("finds first GoF pattern in content", () => {
    expect(extractPattern("Uses the Mediator pattern")).toBe("Mediator");
  });

  it("returns -- when no pattern found", () => {
    expect(extractPattern("No patterns here")).toBe("--");
  });

  it("finds Strategy in file table content", () => {
    const content = "| `renderer.ts` | Strategy | Pluggable rendering | active |";
    expect(extractPattern(content)).toBe("Strategy");
  });
});

describe("extractPatternStatus", () => {
  it("detects verified status", () => {
    expect(extractPatternStatus("Mediator (verified)")).toBe("verified");
  });

  it("defaults to planned", () => {
    expect(extractPatternStatus("Mediator")).toBe("planned");
  });
});

describe("extractDescription", () => {
  it("extracts first non-marker line", () => {
    const content = "@c4 container\n\nReal-time dashboard.\n\n| File |";
    expect(extractDescription(content)).toBe("Real-time dashboard.");
  });

  it("skips @c4 lines and table lines", () => {
    const content = "@c4 component\n@c4 uses foo \"bar\" \"baz\"\n\nThe description.\n| File |";
    expect(extractDescription(content)).toBe("The description.");
  });

  it("returns default when no description", () => {
    expect(extractDescription("@c4 container")).toBe("*No description*");
  });
});

describe("extractParentContainer", () => {
  it("extracts parent from nested path", () => {
    expect(extractParentContainer("dashboard.charts")).toBe("dashboard");
  });

  it("returns null for top-level", () => {
    expect(extractParentContainer("dashboard")).toBeNull();
  });

  it("extracts first segment from deeply nested", () => {
    expect(extractParentContainer("dashboard.charts.axis")).toBe("dashboard");
  });
});

describe("extractRelationships", () => {
  it("parses @c4 uses tags", () => {
    const content = '@c4 uses api_gateway "Market data" "WebSocket"';
    const rels = extractRelationships(content);
    expect(rels).toHaveLength(1);
    expect(rels[0]).toEqual({
      target: "api_gateway",
      label: "Market data",
      protocol: "WebSocket",
    });
  });

  it("parses multiple relationships", () => {
    const content = [
      '@c4 uses auth "Session tokens" "REST"',
      '@c4 uses db "Persistence" "SQLite"',
    ].join("\n");
    const rels = extractRelationships(content);
    expect(rels).toHaveLength(2);
    expect(rels[0].target).toBe("auth");
    expect(rels[1].target).toBe("db");
  });

  it("returns empty array when no relationships", () => {
    expect(extractRelationships("No uses here")).toEqual([]);
  });
});

describe("extractFileTable", () => {
  it("parses file table entries", () => {
    const content = [
      "| File | Pattern | Purpose | Health |",
      "|------|---------|---------|--------|",
      "| `core.ts` | Facade | Entry point | stable |",
      "| `renderer.ts` | Strategy | Rendering | active |",
    ].join("\n");
    const files = extractFileTable(content);
    expect(files).toHaveLength(2);
    expect(files[0]).toEqual({
      name: "core.ts",
      pattern: "Facade",
      pattern_status: "planned",
      purpose: "Entry point",
      health: "stable",
    });
    expect(files[1].health).toBe("active");
  });

  it("parses pattern status in parentheses", () => {
    const content = [
      "| File | Pattern | Purpose | Health |",
      "|------|---------|---------|--------|",
      "| `core.ts` | Mediator (verified) | Orchestration | stable |",
    ].join("\n");
    const files = extractFileTable(content);
    expect(files[0].pattern).toBe("Mediator");
    expect(files[0].pattern_status).toBe("verified");
  });

  it("returns empty array when no table", () => {
    expect(extractFileTable("Just some text")).toEqual([]);
  });

  it("handles -- pattern", () => {
    const content = [
      "| File | Pattern | Purpose | Health |",
      "|------|---------|---------|--------|",
      "| `config.ts` | -- | Configuration | planned |",
    ].join("\n");
    const files = extractFileTable(content);
    expect(files[0].pattern).toBe("--");
    expect(files[0].health).toBe("planned");
  });
});
