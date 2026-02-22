/**
 * postinstall script — downloads the archidoc native binary from GitHub Releases.
 *
 * Platform detection → build download URL → follow redirects → save to bin/.
 * Exits 0 on failure so `npm install` is never blocked.
 */

const https = require("https");
const http = require("http");
const fs = require("fs");
const path = require("path");

const VERSION = require("../package.json").version;
const REPO = "GitSmart86/archidoc";

const PLATFORM_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};

function main() {
  const key = `${process.platform}-${process.arch}`;
  const target = PLATFORM_MAP[key];

  if (!target) {
    console.warn(`archidoc: unsupported platform ${key}`);
    console.warn(`Supported: ${Object.keys(PLATFORM_MAP).join(", ")}`);
    console.warn("Install from source: cargo install archidoc-cli");
    process.exit(0);
  }

  const ext = process.platform === "win32" ? ".exe" : "";
  const asset = `archidoc-${target}${ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${asset}`;
  const dest = path.join(__dirname, "..", "bin", `archidoc${ext}`);

  console.log(`archidoc: downloading v${VERSION} for ${key}...`);

  download(url, dest, 0, (err) => {
    if (err) {
      console.warn(`archidoc: download failed — ${err.message}`);
      console.warn("Install from source: cargo install archidoc-cli");
      process.exit(0); // don't block npm install
    }

    // Make executable on Unix
    if (process.platform !== "win32") {
      try {
        fs.chmodSync(dest, 0o755);
      } catch (e) {
        // non-fatal
      }
    }

    console.log(`archidoc: installed v${VERSION} for ${key}`);
  });
}

/**
 * Download a URL to a file, following up to 5 redirects.
 * GitHub Releases redirect to S3/CloudFront, so redirect following is required.
 */
function download(url, dest, redirects, callback) {
  if (redirects > 5) {
    callback(new Error("too many redirects"));
    return;
  }

  const mod = url.startsWith("https") ? https : http;

  mod
    .get(url, { headers: { "User-Agent": "archidoc-npm-installer" } }, (res) => {
      // Follow redirects (301, 302, 303, 307, 308)
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume(); // drain response
        download(res.headers.location, dest, redirects + 1, callback);
        return;
      }

      if (res.statusCode !== 200) {
        res.resume();
        callback(new Error(`HTTP ${res.statusCode} from ${url}`));
        return;
      }

      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on("finish", () => {
        file.close(() => callback(null));
      });
      file.on("error", (err) => {
        fs.unlink(dest, () => {}); // clean up partial file
        callback(err);
      });
    })
    .on("error", (err) => {
      callback(err);
    });
}

main();
