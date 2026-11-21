import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const config = JSON.parse(readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
const pkg = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
const manifest = readFileSync(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");

test("Tauri, Vite, and CSP agree on the IPv4 loopback development server", () => {
  assert.equal(config.build.devUrl, "http://127.0.0.1:1420");
  assert.equal(config.build.beforeDevCommand, "npm run dev");
  assert.match(pkg.scripts.dev, /--host 127\.0\.0\.1\b/);
  assert.match(pkg.scripts.dev, /--port 1420\b/);
  assert.match(pkg.scripts.dev, /--strictPort\b/);
  assert.match(config.app.security.devCsp, /ws:\/\/127\.0\.0\.1:1420\b/);
});

test("default Cargo builds use embedded frontend assets", () => {
  assert.match(manifest, /\[features\]\s*default\s*=\s*\["custom-protocol"\]/);
  assert.match(manifest, /custom-protocol\s*=\s*\["tauri\/custom-protocol"\]/);
  assert.equal(config.build.frontendDist, "../dist");
});
