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

_No screenshots or videos were added when this guide was created._
