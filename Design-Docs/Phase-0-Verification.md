# Phase Zero platform verification

[Roadmap issue #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) tracks the platform gates. The [architecture baseline ADR](ADRs/phase-0-architecture-baseline.md) records the layout and boundaries. [Phase One draft #11](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/11) retains the persistence work for integration after this platform baseline.

## Run from a checkout

Install stable Rust and Node.js 24. Install the [Tauri native prerequisites](https://v2.tauri.app/start/prerequisites/) for the operating system. Windows uses the MSVC build tools and Edge WebView2.

```sh
cd apps/desktop
npm ci
npm run tauri dev
```

To test embedded assets, stop the development window and run these commands from the repository root.

```sh
npm --prefix apps/desktop run build
cargo build --workspace --locked --features custom-protocol
```

Launch `target/debug/atlas-desktop` on Linux or `target\debug\atlas-desktop.exe` on Windows. The embedded build needs no development web server. A successful native launch displays `Local scene · linux` or `Local scene · windows` along with the five Northbridge nodes. Select a node with Tab and Enter, compare its linked records in the inspector, zoom with the controls, and scroll the focused diagram. A browser preview identifies itself as such. The [Linux runtime capture](../Development-Screenshots/2026-11-20_22-52-16Z_phase-00_desktop_keyboard-canvas-fallback_01.png) shows keyboard selection in the embedded desktop window.

The debug build also accepts `--platform-smoke`. It launches the embedded WebView, loads the fixture through Rust IPC, verifies the inspector and a rendered Canvas2D node, and emits one structured report with the native WebView version and WebGPU adapter status. The process exits with a failure code when the scene or Canvas2D verification fails. Windows CI runs this check after building the executable.

```powershell
./apps/desktop/scripts/windows-platform-smoke.ps1 -Binary ./target/debug/atlas-desktop.exe
```

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --features custom-protocol -- -D warnings
cargo test --workspace --locked --features custom-protocol
cd apps/desktop
npx playwright install chromium --only-shell
npm test
```

## Observations

| Check | Observed result |
| --- | --- |
| Rust 2024 / Tauri | Built successfully on Debian 13 x86-64 with Rust 1.100.0, Tauri 2.12.7, GTK 3.24.49 and WebKitGTK 2.54.0. Rust formatting, Clippy and the scene-integrity unit test passed. |
| Embedded native runtime | The binary launched in an Xvfb window with no development server. Its WebKitGTK frontend displayed `Local scene · linux` from the typed Rust IPC response. Tab and Enter selected Beatrice and displayed her guild membership. |
| AT-31 platform portion | The native fixture and frontend assets loaded locally. Full offline authoring, search and publication require the later application phases. |
| AT-32 fallback portion | The native window reported `Canvas2D · WebGPU unavailable` and drew the Northbridge graph. The virtual display had no exposed GPU device. |
| Browser fallback | Two Playwright tests passed in Chromium Headless Shell 156 with GPU disabled. They exercised keyboard selection, zoom and scroll with WebGPU unavailable, the HTML-only inspector with Canvas2D unavailable, and zero external scene requests. |
| Native automated smoke | Linux WebKitGTK 2.54.0 reported checked inspector and Canvas2D rendering with `api_unavailable` for WebGPU. |
| Windows target | The original Phase Zero workflow passed the Windows native build. The Windows runtime smoke now runs in CI. Its result and target GPU behavior require separate evidence. |

The Phase Zero exit still requires running the desktop shell in Windows WebView2, recording the runtime and adapter result, and validating the wgpu/WASM graph or documenting target platform limits with the working fallback. A five-node Canvas2D fixture does not cover the 2,000-label benchmark or device-loss recovery. The branch/time, world identity, transaction and publishing choices are documented contracts for later runtime tests.
