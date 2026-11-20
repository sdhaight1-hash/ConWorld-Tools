import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const wasm = "target/wasm32-unknown-unknown/release/atlas_renderer.wasm";
const out = "apps/desktop/public/wgpu";
const run = (program, args) => execFileSync(program, args, { cwd: repo, stdio: "inherit" });

run("cargo", ["build", "--locked", "--release", "--target", "wasm32-unknown-unknown", "-p", "atlas-renderer", "--target-dir", "target"]);
run("wasm-bindgen", ["--target", "web", "--no-typescript", "--out-dir", out, wasm]);
