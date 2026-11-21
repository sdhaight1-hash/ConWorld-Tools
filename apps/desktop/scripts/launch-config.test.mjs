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

test("root Windows launcher starts Vite through Tauri, including first-run npm setup", () => {
  const launcher = readFileSync(new URL("../../../start.bat", import.meta.url), "utf8");
  assert.match(launcher, /pushd "%~dp0apps\\desktop"/i);
  assert.match(launcher, /where npm >nul 2>&1/i);
  assert.match(launcher, /where cargo >nul 2>&1/i);
  assert.match(launcher, /node_modules\\\.bin\\tauri\.cmd/i);
  assert.match(launcher, /call npm ci/i);
  assert.match(launcher, /call npm run tauri dev/i);
  assert.doesNotMatch(launcher, /target\\debug\\atlas-desktop\.exe/i);
});

test("desktop command handler, build manifest, and main-window capability agree", () => {
  const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
  const handler = read("../src-tauri/src/main.rs").match(/tauri::generate_handler!\[([^\]]+)\]/)[1];
  const declared = [...read("../src-tauri/build.rs").matchAll(/"([a-z_]+)"/g)].map((match) => match[1]);
  const registered = handler.split(",").map((command) => command.trim().split("::").at(-1)).filter(Boolean);
  const capability = JSON.parse(read("../src-tauri/capabilities/platform-shell.json"));
  assert.deepEqual(declared.sort(), registered.sort());
  assert.deepEqual(capability.permissions.sort(), registered.map((command) => "allow-" + command.replaceAll("_", "-")).sort());
});
