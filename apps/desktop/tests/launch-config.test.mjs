import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const app = new URL("../", import.meta.url);
const read = (file) => readFileSync(new URL(file, app), "utf8");
const config = JSON.parse(read("src-tauri/tauri.conf.json"));
const pkg = JSON.parse(read("package.json"));
const manifest = read("src-tauri/Cargo.toml");

test("ordinary Cargo builds embed frontend assets instead of loading the dev URL", () => {
  const features = manifest.split(/^\[features\]\s*$/m)[1];
  assert.ok(features, "desktop Cargo manifest must declare a features section");
  assert.match(features, /^default\s*=\s*\[\s*"custom-protocol"\s*\]/m);
  assert.match(features, /^custom-protocol\s*=\s*\[\s*"tauri\/custom-protocol"\s*\]/m);
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.build.beforeBuildCommand, "npm run build");
});

test("Tauri and Vite agree on one explicit IPv4 loopback development URL", () => {
  const url = new URL(config.build.devUrl);
  assert.equal(url.protocol, "http:");
  assert.equal(url.hostname, "127.0.0.1");
  assert.equal(url.port, "1420");
  assert.equal(config.build.beforeDevCommand, "npm run dev");
  assert.match(pkg.scripts.dev, /--host 127\.0\.0\.1 --port 1420 --strictPort/);
  assert.ok(config.app.security.devCsp.includes("ws://127.0.0.1:1420"));
});
