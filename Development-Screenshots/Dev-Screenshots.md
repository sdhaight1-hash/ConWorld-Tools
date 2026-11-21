# Development Screenshots & Videos

This directory is the **chronological visual development record** for ConWorld Tools. Coding and research agents (including Lola) may save authentic screenshots, short screen recordings, and relevant visual comparisons captured during design, implementation, testing, debugging, and review.

These media are **development evidence**, not substitutes for working code, reproducible tests, logs, design decisions, or ticket acceptance criteria. They are not part of the application's published fictional-world image library.

## What belongs here

- Screenshots of actual running UI, Node Studio, Canvas Studio, Display Studio, graph/timeline renderers, and tools used to validate functionality.
- Before/after screenshots that demonstrate a specific fix, regression, layout change, or visible behavior.
- Short screen recordings showing an interaction, reproduced bug, performance/rendering behavior, or validated feature workflow.
- Annotated copies when annotations help explain a finding; retain the original capture or disclose edits.

Do **not** add stock images, mockups presented as working software, unrelated assets, raw generated graphics, or fabricated screenshots. Clearly label prototypes and mockups when they are useful to show.

## Required chronological filename convention

Use the **actual capture time in UTC**, with 24-hour clock and zero-padded fields:

```text
YYYY-MM-DD_HH-mm-ssZ_phase-NN_area_short-description_SS.ext
```

| Segment | Rule |
| --- | --- |
| `YYYY-MM-DD_HH-mm-ssZ` | Real capture date/time in UTC, not an in-world/fictional date or commit date. Use hyphens instead of colons. |
| `phase-NN` | Zero-padded roadmap phase number, e.g. `phase-00`, `phase-02`; use `phase-NA` when the capture genuinely does not belong to a roadmap phase. |
| `area` | Short lowercase kebab-case subsystem, such as `node-studio`, `canvas`, `relationships`, `query`, or `wgpu`. |
| `short-description` | Brief lowercase kebab-case description of what was observed. Avoid vague names like `screenshot` or `test`. |
| `SS` | Two-digit sequence (`01`, `02`, ...) for captures sharing the same UTC second/phase/area/description; never overwrite an older capture to reuse a filename. |
| `ext` | Use `.png`, `.webp`, or `.jpg` for still screenshots; `.mp4` or `.webm` for video. |

**Examples (illustrative filenames, not existing media):**

```text
2026-11-20_22-14-00Z_phase-00_wgpu-webview_canvas-integration_01.png
2026-11-20_22-14-03Z_phase-02_relationships_betrayal-timeline_01.png
2026-11-20_22-16-45Z_phase-02_relationships_betrayal-timeline_01.mp4
2026-11-21_01-05-12Z_phase-04_canvas_template-drag-and-drop_01.webm
2026-11-21_01-05-12Z_phase-04_canvas_template-drag-and-drop_02.png
```

Filename sorting by name is chronological **across UTC timestamps**. Keep the media in this directory unless a later explicit repository convention creates dated subfolders. Do not rename existing media casually: preserve links from tickets and reviews.

## Capture and documentation workflow

1. **Capture actual behavior** while the relevant app/build is running. Do not fabricate screenshots or claim to have tested a UI that was not launched.
2. **Name and check the file** using the UTC convention. Prefer an appropriately cropped, readable image; for video, show the essential interaction in a reasonably short recording and avoid unnecessarily huge files.
3. **Describe the evidence** in the relevant GitHub issue or pull request, including the linked file, what it demonstrates, observed result, build/commit or branch, environment if relevant, and whether it is a failure, prototype, or verified working behavior.
4. **Link back from the media index below** for important captures. Use a dated entry with the actual file link, relevant issue/PR, and a short observation. Preserve chronological order.
5. **Verify before citing** that the media is uploaded and the link opens. Screenshots complement—not replace—automated tests and reproducible logs.
6. **Protect private material.** Check for credentials, personal information, file paths with sensitive identifiers, private messages, confidential project content, or unpublished world secrets before committing. Remove/redact sensitive information, and state when a capture has been redacted. Avoid putting such information in filenames.
7. **Keep the repository manageable.** Prefer efficient formats and short clips. For videos too large or numerous for normal Git history, use a specifically approved Git LFS or release/artifact storage approach and link to it instead of committing oversized files.

For capture-specific requirements, consult [the root README](../README.md), [AI Engineering Instructions](../Design-Docs/11-AI-Engineering-Instructions.md), and [Acceptance and Verification](../Design-Docs/09-Acceptance-and-Verification.md). Link to the relevant `Roadmap Phase`, `Research:`, or `New Feature:` issue as appropriate.

## Chronological development media index

Add entries when actual screenshots or videos are committed. Example format:

```text
- YYYY-MM-DD HH:mm:ss UTC — [filename](filename) — Issue #N, commit abc123 — What was observed and why this capture matters.
```

- 2026-11-20 22:52:16 UTC — [2026-11-20_22-52-16Z_phase-00_desktop_keyboard-canvas-fallback_01.png](2026-11-20_22-52-16Z_phase-00_desktop_keyboard-canvas-fallback_01.png) — [Phase Zero #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1), [PR #12](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/12) — The embedded Tauri build on Linux WebKitGTK 2.54.0 displayed Beatrice’s guild membership after keyboard selection, with the Canvas2D graph active and WebGPU unavailable.
- 2026-11-20 23:13:39 UTC — [2026-11-20_23-13-39Z_phase-00_renderer_benchmark-canvas-fallback_01.png](2026-11-20_23-13-39Z_phase-00_renderer_benchmark-canvas-fallback_01.png) — [Phase Zero #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1), [PR #12](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/12) — The embedded Linux desktop opened the 2,000-node benchmark with the Canvas2D fallback and node inspector.
- 2026-11-21 03:34:12 UTC — [Relationship effect](2026-11-21_03-34-12Z_phase-02_relationships_recorded-before-and-after_01.png) — [Phase 2 #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3), [PR #16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16), build [5f1001b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5f1001b14fdedbc63cff291365744afab0fea1ae) — The saved relationship inspector shows trust changing from 90 to 12, the dated tags, the attributed successful action, and its citation.
- 2026-11-21 03:34:13 UTC — [Multi-actor event](2026-11-21_03-34-13Z_phase-02_graph_multi-actor-event_01.png) — [Phase 2 #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3), [PR #16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16), build [5f1001b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5f1001b14fdedbc63cff291365744afab0fea1ae) — Following the relationship's source link opens the saved event, with a failed action, a successful action, and a separately identified witness.
- 2026-11-21 03:34:14 UTC — [Saved action correction](2026-11-21_03-34-14Z_phase-02_events_saved-action-correction_01.png) — [Phase 2 #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3), [PR #16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16), build [5f1001b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5f1001b14fdedbc63cff291365744afab0fea1ae) — The event inspector opens the saved event in the editor and loads the failed action for correction while preserving its actor identity.

These captures came from the native Linux Tauri build with WebKitGTK 2.54.0, Canvas2D, and an isolated SQLite database containing the Northbridge Chronicle test world.

- 2026-11-21 04:41:30 UTC — [Author account inspector](2026-11-21_04-41-30Z_phase-02_evidence_author-account-inspector_01.png) — [Phase 2 #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3), [PR #16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16), build [460abe5](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/460abe5ba1d7410a33f49173ec570366346c4698) — The migrated Northbridge world shows the configured Mediator and Financier roles, canonical action outcomes, and a private sourced account in the author inspector.
- 2026-11-21 04:41:31 UTC — [Public account preview](2026-11-21_04-41-31Z_phase-02_evidence_public-rumor-preview_01.png) — [Phase 2 #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3), [PR #16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16), build [460abe5](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/460abe5ba1d7410a33f49173ec570366346c4698) — The account preview shows the public rumor and omits private accounts and account citations. The author inspector above it still records Alice's action as failed. Captured on native Linux WebKitGTK 2.54.0.

- 2026-11-21 05:12:22 UTC — [Native renderer fallback at 125%](2026-11-21_05-12-22Z_phase-00_renderer_benchmark-fallback-state_01.png) — [Phase 0 #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1), branch `work/phase-0-acceptance` — Linux WebKitGTK 2.54.0 uses Canvas2D with WebGPU unavailable and keeps the selected node, zoom and scroll usable on the 2,000-node benchmark.

- 2026-11-21 18:34:46 UTC — [Pinned historical search, page 2](2026-11-21_18-34-46Z_phase-03_query_mock-native-page-two_01.png) — [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), [PR #37](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/37), build [262d3a94](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/262d3a94f663d23646af7ee0e8caac80b96ef10b), [CI run](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44935806242). Actual Chromium Playwright capture with synthetic native IPC: author revision 7, page 2, distinct person identity and focused exploration/reference controls. Not a native Tauri/SQLite runtime capture.

- 2026-11-21 19:00:22 UTC — [Saved query parameter override](2026-11-21_19-00-22Z_phase-03_query_saved-parameter-override_01.png) — [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), [PR #41](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/41), build [3c844f6](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3c844f6ae1c62ff171bb9c02b57d2ec20474e8fb), [CI run](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44938182599). Actual Chromium Playwright capture of the saved-query form with typed override Bob; synthetic native IPC, not native Tauri/SQLite runtime proof.

- 2026-11-21 19:18:45 UTC — [Focused historical claim discovery](2026-11-21_19-18-45Z_phase-03_discovery_focused-claim_01.png) — [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), [PR #44](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/44), build [41d31df](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/41d31dfbb0ea85f0a7d046bbf6483a5efef1622e), [CI run](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44939778595). Actual Chromium Playwright capture with synthetic native IPC: oversized-world focused claim preview, pinned revision, exact source span and separate collision metadata. Not native Tauri/SQLite runtime proof.

- 2026-11-21 20:07:42 UTC — [Saved character page and public preview](2026-11-21_20-07-42Z_phase-04_character_public-page-preview_01.png), [entry design #5](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5), [privacy #8](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8). Native Linux Tauri/WebKitGTK with SQLite, Canvas2D and the schema-22 development build based on 8d0af9d. Mara Vale's page was saved, the app restarted, and the stored public projection reopened. The upper fields belong to the author editor. The public response below the preview button contains only the released sentence. A subsequent native check unpublished the entry and confirmed it disappeared from public preview.

- 2026-11-21 20:38:21 UTC — [Ada Vale’s saved parents and fictional comparison](2026-11-21_20-38-21Z_phase-05_family_saved-parentage-comparison_01.png). Native Linux Tauri/WebKitGTK, schema-23 development build based on e36ca50. The restarted app shows the two accepted parents of Ada Vale and the computed shared immediate-contributor intervals with Elise Vale. The left canvas displays the existing person nodes. The direct family records appear in the inspector. [Genealogy #6](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) · [Fictional inheritance #9](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9).
