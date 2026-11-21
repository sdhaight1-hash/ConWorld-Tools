import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const appDirectory = fileURLToPath(new URL("../", import.meta.url));
const vite = fileURLToPath(new URL("../node_modules/vite/bin/vite.js", import.meta.url));
const address = "http://127.0.0.1:1420";
const server = spawn(process.execPath, [vite, "--host", "127.0.0.1", "--port", "1420", "--strictPort"], {
  cwd: appDirectory,
  stdio: ["ignore", "pipe", "pipe"],
  windowsHide: true,
});
let output = "";
server.stdout.on("data", (chunk) => { output += chunk.toString(); });
server.stderr.on("data", (chunk) => { output += chunk.toString(); });

try {
  let ready = false;
  for (let attempt = 0; attempt < 60; attempt++) {
    if (server.exitCode !== null) throw new Error(`Vite stopped before serving the page: ${output}`);
    try {
      const response = await fetch(address, { signal: AbortSignal.timeout(1_000) });
      if (response.ok) {
        const html = await response.text();
        assert.match(html, /Living History Atlas/);
        const entry = await fetch(`${address}/src/main.ts`, { signal: AbortSignal.timeout(1_000) });
        assert.equal(entry.status, 200, "Vite must serve the frontend entry point");
        ready = true;
        break;
      }
    } catch (error) {
      if (error instanceof assert.AssertionError) throw error;
    }
    await delay(150);
  }
  assert.ok(ready, `Vite never served the frontend at ${address}: ${output}`);
  console.log(`Dev frontend served successfully at ${address}`);
} finally {
  server.kill();
}
