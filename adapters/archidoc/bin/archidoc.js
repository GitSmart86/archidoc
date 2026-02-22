#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const ext = process.platform === "win32" ? ".exe" : "";
const bin = path.join(__dirname, `archidoc${ext}`);

if (!fs.existsSync(bin)) {
  console.error("archidoc binary not found at: " + bin);
  console.error("");
  console.error("The postinstall script may have failed to download it.");
  console.error("Try reinstalling:  npm install archidoc");
  console.error("Or install from source:  cargo install archidoc-cli");
  process.exit(1);
}

const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });
child.on("close", (code) => process.exit(code ?? 1));
child.on("error", (err) => {
  console.error("Failed to run archidoc: " + err.message);
  console.error("Try reinstalling:  npm install archidoc");
  process.exit(1);
});
