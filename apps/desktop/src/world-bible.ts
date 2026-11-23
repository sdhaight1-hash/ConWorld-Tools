import { invoke } from "@tauri-apps/api/core";
import { canLeaveEntryDrafts } from "./entry-drafts";
import { openEntryEditor } from "./entry-editor";
import { appendConnectionEditor } from "./connection-editor";

export type EntryTarget = { id: string; label: string; kind?: string };
type Context = { world_id: string; day: number; revision: number };
type Ref = { world: string; id: string };
type NetworkFact = { id: string; subject_label: string; target_label: string;
  input: { subject: Ref; target: Ref; source: Ref; kind: string; role: string; starts_at: number; ends_before: number | null } };

// Shared launch path for directory, graph, search results and authored backlinks.
export function entryNavigator(host: HTMLElement, context: Context, editable: boolean,
  valid: () => boolean, targets: EntryTarget[], saved: (id: string) => Promise<void>,
  navigate?: (id: string) => void) {
  function open(id: string) {
    if (!valid() || !host.isConnected || !canLeaveEntryDrafts()) return;
    host.replaceChildren();
    void openEntryEditor(host, context.world_id, id, context.day, context.revision,
      editable, () => saved(id), targets, navigate ?? open);
  }
  return open;
}

// Existing native IdentityKind only. Other facts and events use specialized commands.
const identityKinds = ["person", "place", "concept", "organization", "household", "family_house",
  "faction", "conflict", "office", "title", "asset", "population", "conflict_issue", "conflict_phase", "succession_rule"];
const humanize = (kind: string) => kind.replaceAll("_", " ");
function button(label: string, action: () => void) {
  const element = document.createElement("button");
  element.type = "button"; element.textContent = label; element.addEventListener("click", action);
  return element;
}

// Author-only directory. No complete scene or client-side public filtering.
export function mountWorldBible(host: HTMLElement, context: Context, editable: boolean,
  valid: () => boolean, refresh: () => Promise<void>,
  reopen: (target: EntryTarget, connections?: boolean) => void, initial?: EntryTarget, initialConnections = false) {
  host.innerHTML = `<div class="world-bible-header"><h2>World Bible</h2>
    <p class="hint">Author directory · Main history · Find identities, including those without an article.</p></div>
    <div class="world-bible-columns"><div class="world-bible-directory">
    <form class="editor-form" data-bible-search><label>Find an entry<input name="phrase" type="search" maxlength="800" required placeholder="Name or phrase"></label>
    <button>Find entry</button><button type="button" data-bible-cancel hidden>Cancel directory search</button></form>
    <p role="status" data-bible-search-status>Search any supported entry type.</p>
    <ul data-bible-results aria-label="World Bible directory"></ul>
    <form class="editor-form" data-bible-create><h3>Create identity</h3><label>Entry type<select name="kind"></select></label>
    <label>Entry name<input name="name" required maxlength="200"></label><button>Create identity only</button>
    <p class="hint">Creates a canonical identity effective from the selected world day, not a page or inferred relationship.</p></form>
    <p role="status" data-bible-create-status></p></div>
    <section class="world-bible-entry" aria-label="World Bible entry"><p>Choose an identity to edit its article or inspect typed connections.</p></section></div>`;
  const search = host.querySelector<HTMLFormElement>("[data-bible-search]")!;
  const phrase = search.elements.namedItem("phrase") as HTMLInputElement;
  const cancel = host.querySelector<HTMLButtonElement>("[data-bible-cancel]")!;
  const status = host.querySelector<HTMLElement>("[data-bible-search-status]")!;
  const results = host.querySelector<HTMLUListElement>("[data-bible-results]")!;
  const create = host.querySelector<HTMLFormElement>("[data-bible-create]")!;
  const createStatus = host.querySelector<HTMLElement>("[data-bible-create-status]")!;
  const kind = create.elements.namedItem("kind") as HTMLSelectElement;
  kind.replaceChildren(...identityKinds.map(value => new Option(humanize(value), value)));
  create.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("input,select,button")
    .forEach(control => control.disabled = !editable);
  const entry = host.querySelector<HTMLElement>(".world-bible-entry")!;
  const current = () => valid() && entry.isConnected;
  let generation = 0;
  let pending: string | undefined;
  let rows: EntryTarget[] = [];
  let inspected = 0;
  let creating = false;
  function invalidate() {
    ++generation;
    const id = pending; pending = undefined;
    cancel.hidden = true;
    results.replaceChildren();
    rows = [];
    if (id) void invoke("workspace_cancel_query", { requestId: id }).catch(() => {});
  }
  phrase.addEventListener("input", () => { invalidate(); status.textContent = "Search the selected historical view."; });
  cancel.addEventListener("click", () => { invalidate(); status.textContent = "Directory search cancelled."; });
  function open(target: EntryTarget, connections = false) {
    if (!current() || !canLeaveEntryDrafts()) return;
    ++inspected;
    entry.replaceChildren();
    const title = document.createElement("h2");
    title.textContent = target.label; title.tabIndex = -1;
    const meta = document.createElement("p");
    meta.className = "hint"; meta.textContent = (target.kind ? humanize(target.kind) + " · " : "") + "Author revision " + context.revision + " · day " + context.day;
    const nav = document.createElement("nav");
    nav.setAttribute("aria-label", "Entry tools");
    const body = document.createElement("div");
    const showEditor = () => {
      if (!current() || !body.isConnected || !canLeaveEntryDrafts()) return;
      ++inspected;
      const targets = [...rows.filter(row => row.id !== target.id), target];
      entryNavigator(body, context, editable, current, targets, async () => {
        await refresh(); reopen(target);
      }, id => open(rows.find(row => row.id === id) ?? { id, label: id }))(target.id);
    };
    const showConnections = async () => {
      if (!current() || !body.isConnected || !canLeaveEntryDrafts()) return;
      const token = ++inspected;
      body.textContent = "Loading recorded network facts…";
      try {
        const facts = await invoke<NetworkFact[]>("workspace_network_facts", { request: context, focus: target.id });
        if (!current() || !body.isConnected || token !== inspected) return;
        body.replaceChildren();
        const note = document.createElement("p");
        note.className = "hint"; note.textContent = "Recorded network facts only. Parentage, social state, classifications and claims use separate domain services.";
        body.append(note);
        const list = document.createElement("ul");
        for (const fact of facts) {
          const f = fact.input;
          const other = f.subject.id === target.id ? f.target : f.subject;
          const label = f.subject.id === target.id ? fact.target_label : fact.subject_label;
          const item = document.createElement("li");
          item.textContent = fact.subject_label + " → " + fact.target_label + " · " + humanize(f.kind) + " · " +
            f.role + " · day " + f.starts_at + "–" + (f.ends_before ?? "open") + " · source " + f.source.id + " · fact " + fact.id + " ";
          if (other.world === context.world_id) item.append(button("Open " + label, () => open(rows.find(row => row.id === other.id) ?? { id: other.id, label })));
          list.append(item);
        }
        if (!facts.length) body.append("No recorded network facts at this view.");
        body.append(list);
        if (editable && target.kind) {
          appendConnectionEditor(body, target, context, () => current() && body.isConnected && token === inspected,
            async () => { await refresh(); reopen(target, true); });
        }
      } catch (error) {
        if (current() && body.isConnected && token === inspected) body.textContent = String(error);
      }
    };
    nav.append(button("Edit article", showEditor), button("Connections", () => void showConnections()),
      button("Back to directory", () => {
        if (!current() || !canLeaveEntryDrafts()) return;
        ++inspected; entry.textContent = "Choose an identity from the directory."; phrase.focus();
      }));
    entry.append(title, meta, nav, body);
    if (connections) void showConnections(); else showEditor();
    title.focus();
  }
  search.addEventListener("submit", async event => {
    event.preventDefault();
    if (!current() || !canLeaveEntryDrafts()) return;
    const term = phrase.value.trim(); if (!term) return;
    invalidate();
    const token = generation;
    const requestId = crypto.randomUUID(); pending = requestId;
    cancel.hidden = false; status.textContent = "Finding identities…";
    try {
      const found = await invoke<{ rows: EntryTarget[]; truncated: boolean }>("workspace_semantic_candidates",
        { request: context, kind: "entry", phrase: term, requestId });
      if (!current() || token !== generation) return;
      rows = found.rows;
      for (const target of rows) {
        const item = document.createElement("li");
        item.append(button(target.label + " · " + humanize(target.kind ?? "entry") + " · " + target.id,
          () => { if (current() && token === generation) open(target); }));
        results.append(item);
      }
      status.textContent = rows.length + " matching identities." + (found.truncated ? " Refine your search for further matches." : "");
    } catch (error) {
      if (current() && token === generation) status.textContent = String(error);
    } finally { if (pending === requestId) { pending = undefined; cancel.hidden = true; } }
  });
  create.addEventListener("submit", async event => {
    event.preventDefault();
    if (!current() || !editable || creating || !canLeaveEntryDrafts()) return;
    const label = (create.elements.namedItem("name") as HTMLInputElement).value.trim();
    if (!label) return;
    creating = true;
    const save = create.querySelector<HTMLButtonElement>("button")!; save.disabled = true;
    const type = kind.value;
    try {
      const record = await invoke<Ref>("workspace_create_identity", { request: context, kind: type, label });
      if (!current() || record.world !== context.world_id) return;
      createStatus.textContent = "Identity created. Loading its entry…";
      await refresh(); reopen({ id: record.id, label, kind: type });
    } catch (error) { if (current()) createStatus.textContent = String(error); }
    finally { creating = false; if (current()) save.disabled = !editable; }
  });
  if (initial) open(initial, initialConnections);
  return { clear: () => { invalidate(); ++inspected; host.replaceChildren(); } };
}
