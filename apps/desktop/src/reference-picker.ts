import { invoke } from "@tauri-apps/api/core";

type Context = { world_id: string; day: number; revision: number };
type Candidate = { id: string; label: string; kind?: string };

// Reuse the bounded, author-only semantic candidate service. This picker
// only edits the existing reference field; Rust still validates every write.
export function appendReferencePicker(parent: HTMLElement, target: HTMLInputElement,
  context: Context, kind: "entry" | "source" | "tag", label: string) {
  const host = document.createElement("details");
  host.dataset.referencePicker = "";
  const summary = document.createElement("summary");
  summary.textContent = "Find " + label;
  const phrase = document.createElement("input");
  phrase.type = "search";
  phrase.maxLength = 800;
  phrase.setAttribute("aria-label", "Search " + label);
  phrase.placeholder = "Name or phrase";
  const search = document.createElement("button");
  search.type = "button";
  search.textContent = "Search references";
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.textContent = "Cancel reference search";
  cancel.hidden = true;
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  const results = document.createElement("ul");
  host.append(summary, phrase, search, cancel, status, results);
  parent.append(host);
  let generation = 0;
  let active: string | undefined;
  function invalidate() {
    ++generation;
    const id = active;
    active = undefined;
    cancel.hidden = true;
    results.replaceChildren();
    status.textContent = "";
    if (id) void invoke("workspace_cancel_query", { requestId: id }).catch(() => {});
  }
  phrase.addEventListener("input", invalidate);
  target.addEventListener("input", invalidate);
  host.addEventListener("toggle", () => { if (!host.open) invalidate(); });
  cancel.addEventListener("click", () => { invalidate(); status.textContent = "Reference search cancelled."; });
  async function find() {
    const term = phrase.value.trim();
    if (!term || target.readOnly || target.matches(":disabled")) return;
    invalidate();
    const token = generation;
    const initial = target.value;
    const requestId = crypto.randomUUID();
    active = requestId;
    cancel.hidden = false;
    status.textContent = "Searching the selected author view…";
    const current = () => generation === token && host.isConnected && target.isConnected &&
      host.open && target.value === initial && !target.readOnly && !target.matches(":disabled");
    try {
      const found = await invoke<{ rows: Candidate[]; truncated: boolean }>(
        "workspace_semantic_candidates", { request: context, requestId, kind, phrase: term });
      if (!current()) return;
      for (const row of found.rows) {
        const item = document.createElement("li");
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = row.label + (row.kind ? " · " + row.kind.replaceAll("_", " ") : "") + " · " + row.id;
        button.addEventListener("click", () => {
          if (!current()) return;
          target.value = row.id;
          target.dispatchEvent(new Event("input", { bubbles: true }));
          host.open = false;
          target.focus();
        });
        item.append(button);
        results.append(item);
      }
      status.textContent = found.rows.length + " compatible references." +
        (found.truncated ? " Refine the phrase to see further matches." : "");
    } catch (error) {
      if (current()) status.textContent = String(error);
    } finally {
      if (active === requestId) { active = undefined; cancel.hidden = true; }
    }
  }
  search.addEventListener("click", () => { void find(); });
  phrase.addEventListener("keydown", event => {
    if (event.key === "Enter" && !event.isComposing) {
      event.preventDefault(); void find();
    }
  });
}
