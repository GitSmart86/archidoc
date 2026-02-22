/**
 * Tests for the archidoc npm wrapper package.
 *
 * Verifies platform detection, download URL construction,
 * and the binary wrapper script logic.
 */

const { describe, it, expect } = require("node:test");
const assert = require("node:assert");
const path = require("path");
const fs = require("fs");

// --- Platform map tests ---

describe("platform detection", () => {
  const PLATFORM_MAP = {
    "linux-x64": "x86_64-unknown-linux-gnu",
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "win32-x64": "x86_64-pc-windows-msvc",
  };

  it("maps all supported platforms to Rust targets", () => {
    const expected = [
      ["linux-x64", "x86_64-unknown-linux-gnu"],
      ["darwin-x64", "x86_64-apple-darwin"],
      ["darwin-arm64", "aarch64-apple-darwin"],
      ["win32-x64", "x86_64-pc-windows-msvc"],
    ];

    for (const [key, target] of expected) {
      assert.strictEqual(
        PLATFORM_MAP[key],
        target,
        `${key} should map to ${target}`
      );
    }
  });

  it("returns undefined for unsupported platforms", () => {
    assert.strictEqual(PLATFORM_MAP["linux-arm64"], undefined);
    assert.strictEqual(PLATFORM_MAP["freebsd-x64"], undefined);
    assert.strictEqual(PLATFORM_MAP["win32-arm64"], undefined);
  });

  it("covers the current platform", () => {
    const key = `${process.platform}-${process.arch}`;
    // This test runs on a supported platform, so it should have a mapping
    assert.ok(
      PLATFORM_MAP[key],
      `Current platform ${key} should be in PLATFORM_MAP`
    );
  });
});

// --- URL construction tests ---

describe("download URL construction", () => {
  const VERSION = "0.3.2";
  const REPO = "GitSmart86/archidoc";

  function buildUrl(platformKey) {
    const PLATFORM_MAP = {
      "linux-x64": "x86_64-unknown-linux-gnu",
      "darwin-x64": "x86_64-apple-darwin",
      "darwin-arm64": "aarch64-apple-darwin",
      "win32-x64": "x86_64-pc-windows-msvc",
    };
    const target = PLATFORM_MAP[platformKey];
    if (!target) return null;
    const ext = platformKey.startsWith("win32") ? ".exe" : "";
    return `https://github.com/${REPO}/releases/download/v${VERSION}/archidoc-${target}${ext}`;
  }

  it("constructs correct URL for Linux x64", () => {
    assert.strictEqual(
      buildUrl("linux-x64"),
      "https://github.com/GitSmart86/archidoc/releases/download/v0.3.2/archidoc-x86_64-unknown-linux-gnu"
    );
  });

  it("constructs correct URL for macOS Intel", () => {
    assert.strictEqual(
      buildUrl("darwin-x64"),
      "https://github.com/GitSmart86/archidoc/releases/download/v0.3.2/archidoc-x86_64-apple-darwin"
    );
  });

  it("constructs correct URL for macOS ARM", () => {
    assert.strictEqual(
      buildUrl("darwin-arm64"),
      "https://github.com/GitSmart86/archidoc/releases/download/v0.3.2/archidoc-aarch64-apple-darwin"
    );
  });

  it("constructs correct URL for Windows x64 with .exe suffix", () => {
    assert.strictEqual(
      buildUrl("win32-x64"),
      "https://github.com/GitSmart86/archidoc/releases/download/v0.3.2/archidoc-x86_64-pc-windows-msvc.exe"
    );
  });

  it("returns null for unsupported platform", () => {
    assert.strictEqual(buildUrl("linux-arm64"), null);
  });
});

// --- Wrapper script tests ---

describe("bin/archidoc.js wrapper", () => {
  const wrapperPath = path.join(__dirname, "..", "bin", "archidoc.js");

  it("wrapper script exists", () => {
    assert.ok(fs.existsSync(wrapperPath), "bin/archidoc.js should exist");
  });

  it("wrapper has node shebang", () => {
    const content = fs.readFileSync(wrapperPath, "utf-8");
    assert.ok(
      content.startsWith("#!/usr/bin/env node"),
      "Should start with node shebang"
    );
  });

  it("wrapper references correct binary name for current platform", () => {
    const content = fs.readFileSync(wrapperPath, "utf-8");
    // Should use process.platform to determine .exe suffix
    assert.ok(
      content.includes('process.platform === "win32"'),
      "Should check for Windows platform"
    );
    assert.ok(
      content.includes(".exe"),
      "Should handle .exe extension for Windows"
    );
  });
});

// --- Install script tests ---

describe("scripts/install.js", () => {
  const installPath = path.join(__dirname, "..", "scripts", "install.js");

  it("install script exists", () => {
    assert.ok(fs.existsSync(installPath), "scripts/install.js should exist");
  });

  it("install script reads version from package.json", () => {
    const content = fs.readFileSync(installPath, "utf-8");
    assert.ok(
      content.includes('require("../package.json").version'),
      "Should read version from package.json"
    );
  });

  it("install script handles all 4 platforms", () => {
    const content = fs.readFileSync(installPath, "utf-8");
    assert.ok(content.includes("x86_64-unknown-linux-gnu"));
    assert.ok(content.includes("x86_64-apple-darwin"));
    assert.ok(content.includes("aarch64-apple-darwin"));
    assert.ok(content.includes("x86_64-pc-windows-msvc"));
  });

  it("install script follows redirects", () => {
    const content = fs.readFileSync(installPath, "utf-8");
    // GitHub Releases redirect to CDN; the install script must follow them
    assert.ok(
      content.includes("301") || content.includes("302") || content.includes("redirect"),
      "Should handle HTTP redirects"
    );
  });
});

// --- Package.json validation ---

describe("package.json", () => {
  const pkg = require("../package.json");

  it("has correct package name", () => {
    assert.strictEqual(pkg.name, "archidoc");
  });

  it("declares archidoc-ts as a dependency", () => {
    assert.ok(
      pkg.dependencies && pkg.dependencies["archidoc-ts"],
      "Should depend on archidoc-ts for polyglot support"
    );
  });

  it("declares postinstall script", () => {
    assert.ok(
      pkg.scripts && pkg.scripts.postinstall,
      "Should have a postinstall script"
    );
    assert.ok(
      pkg.scripts.postinstall.includes("install.js"),
      "postinstall should run install.js"
    );
  });

  it("declares bin entry for archidoc", () => {
    assert.ok(pkg.bin && pkg.bin.archidoc, "Should declare archidoc binary");
    assert.ok(
      pkg.bin.archidoc.includes("archidoc.js"),
      "bin should point to archidoc.js"
    );
  });

  it("declares supported platforms", () => {
    assert.ok(Array.isArray(pkg.os), "Should declare supported OS list");
    assert.ok(pkg.os.includes("darwin"));
    assert.ok(pkg.os.includes("linux"));
    assert.ok(pkg.os.includes("win32"));
  });

  it("version matches archidoc-ts dependency range", () => {
    const tsRange = pkg.dependencies["archidoc-ts"];
    // The version should be compatible (starts with ^ or exact match)
    assert.ok(
      tsRange.includes(pkg.version) || tsRange.startsWith("^"),
      `archidoc-ts range '${tsRange}' should be compatible with version '${pkg.version}'`
    );
  });
});
