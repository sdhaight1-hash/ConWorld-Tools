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

## Renderer benchmark

Install the WASM Rust target and the binding generator used by the lockfile. Build the optional renderer bundle before building or starting the desktop frontend.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm --prefix apps/desktop run build:renderer
npm --prefix apps/desktop run build
```

Open **Renderer benchmark** in the native shell. The deterministic fixture has 2,000 nodes and 3,910 edges. Pan with the scrollable diagram, adjust zoom, and select a node in the diagram or inspector. When the adapter and bundle load, wgpu submits the node and edge geometry and Canvas2D draws the labels. **Simulate device loss** destroys the active WebGPU device. **Retry WebGPU** rebuilds the renderer with the same selection, zoom and scroll state. The optional Playwright path for a compatible Chromium software implementation runs with `ATLAS_SOFTWARE_WEBGPU=1 npm test` in `apps/desktop`.

## Observations

| Check | Observed result |
| --- | --- |
| Rust 2024 / Tauri | Built successfully on Debian 13 x86-64 with Rust 1.100.0, Tauri 2.12.7, GTK 3.24.49 and WebKitGTK 2.54.0. Rust formatting, Clippy and the scene-integrity unit test passed. |
| Embedded native runtime | The binary launched in an Xvfb window with no development server. Its WebKitGTK frontend displayed `Local scene · linux` from the typed Rust IPC response. Tab and Enter selected Beatrice and displayed her guild membership. |
| AT-31 platform portion | The native fixture and frontend assets loaded locally. Full offline authoring, search and publication require the later application phases. |
| AT-32 fallback portion | The native window reported `Canvas2D · WebGPU unavailable` and drew the Northbridge graph. The virtual display had no exposed GPU device. |
| Browser fallback | Two Playwright tests passed in Chromium Headless Shell 156 with GPU disabled. They exercised keyboard selection, zoom and scroll with WebGPU unavailable, the HTML-only inspector with Canvas2D unavailable, and zero external scene requests. |
| Native automated smoke | Linux WebKitGTK 2.54.0 reported checked inspector and Canvas2D rendering with `api_unavailable` for WebGPU. |
| Hosted Windows fallback | WebView2 157.0.4341.46 ran the embedded app, verified the scene and selected-node pixels, and reported `adapter_unavailable`. [Windows CI evidence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44832568493/job/136620893121). |
| 2,000-node workload | Three fallback browser tests passed, including selection of the final benchmark node. The [embedded Linux capture](../Development-Screenshots/2026-11-20_23-13-39Z_phase-00_renderer_benchmark-canvas-fallback_01.png) records the full benchmark viewport with WebGPU unavailable. |
| Accelerated benchmark | The WASM renderer built and passed Clippy for `wasm32-unknown-unknown`. Both local Chromium builds exposed a software adapter and lost the device before stable presentation. A separate minimal WebGPU clear also lost its device in this environment. Hardware rendering, successful recovery and timing measurements are unverified. |

The Phase Zero exit still requires exercising the wgpu/WASM graph on GPU-capable Windows WebView2, or establishing a target platform limitation and verifying the fallback there. The branch/time, world identity, transaction and publishing choices are documented contracts for later runtime tests.
