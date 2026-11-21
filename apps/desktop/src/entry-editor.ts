import { invoke } from "@tauri-apps/api/core";

import type { ParameterValue, SavedQuery } from "./author-workspace";

type Binding = { world: string; query_id: string; definition_revision: number;
  bindings: { name: string; value: ParameterValue }[] };
type Block = { id: string; audience: string; content: { Text?: string; Stack?: Block[]; SavedQuery?: Binding } };
type Document = { schema_version: number; entry: { world: string; id: string }; root: Block };
type Display = { content: { Text?: string; Stack?: Display[];
  QueryRows?: { records: [{ world: string; id: string }, string][]; truncated: boolean } } };
type EntrySearch = { matches: [string, string][]; truncated: boolean };

export async function openEntryEditor(panel: HTMLElement, worldId: string, entryId: string,
  day: number, revision: number, editable: boolean, afterSave: () => Promise<void>) {
  const host = document.createElement("section");
  panel.append(host);
  host.innerHTML = `<h3>Character page</h3><form class="editor-form"><fieldset disabled>
    <label>Page text <textarea name="body" required></textarea></label>
    <label>Author notes <textarea name="notes"></textarea></label>
    <label>Private saved query <select name="saved_query"><option value="">No query table</option></select></label>
    <p class="hint">Author only. Pin a definition revision; results follow the selected historical view.</p>
    <div data-binding-parameters></div>
    <label><input type="checkbox" name="published"> Publish page text</label>
    <button>Save page</button></fieldset></form>
    <button type="button" data-author-preview-button disabled>Preview author layout</button>
    <div data-author-preview aria-live="polite"></div>
    <button type="button" data-public-preview-button>View current public page</button>
    <form class="editor-form" data-public-search>
      <label>Find in public page <input name="text" required maxlength="256"></label>
      <button>Find</button>
    </form><div data-public-result aria-live="polite"></div>`;
  const form = host.querySelector("form")!;
  const fields = host.querySelector("fieldset")!;
  const body = form.elements.namedItem("body") as HTMLTextAreaElement;
  const notes = form.elements.namedItem("notes") as HTMLTextAreaElement;
  const published = form.elements.namedItem("published") as HTMLInputElement;
  const result = host.querySelector<HTMLElement>("[data-public-result]")!;
  const authorResult = host.querySelector<HTMLElement>("[data-author-preview]")!;
  const authorPreview = host.querySelector<HTMLButtonElement>("[data-author-preview-button]")!;
  const querySelect = form.elements.namedItem("saved_query") as HTMLSelectElement;
  const parameterHost = host.querySelector<HTMLElement>("[data-binding-parameters]")!;
  let authorSequence = 0;
  let activeAuthor: string | undefined;
  function clearAuthorPreview() {
    ++authorSequence;
    const requestId = activeAuthor;
    activeAuthor = undefined;
    authorResult.replaceChildren();
    if (requestId) void invoke("workspace_cancel_query", { requestId }).catch(() => {});
  }
  form.addEventListener("input", clearAuthorPreview);
  const args = { worldId, entryId, day };
  let request = 0;
  const find = host.querySelector<HTMLFormElement>("[data-public-search]")!;
  find.addEventListener("input", () => { request++; result.replaceChildren(); });
  find.addEventListener("submit", async event => {
    event.preventDefault();
    const text = (find.elements.namedItem("text") as HTMLInputElement).value.trim();
    if (!text) return;
    const current = ++request;
    result.textContent = "Searching current public page…";
    try {
      const found = await invoke<EntrySearch | null>("workspace_search_public_entry", { ...args, text });
      if (current !== request || !host.isConnected) return;
      result.replaceChildren();
      if (!found) { result.textContent = "Unpublished at this date."; return; }
      const summary = document.createElement("p");
      summary.textContent = found.truncated ? "Showing first 50 matching blocks." :
        `${found.matches.length} matching blocks.`;
      result.append(summary);
      for (const [, text] of found.matches) {
        const p = document.createElement("p");
        p.textContent = text;
        result.append(p);
      }
    } catch (error) {
      if (current === request && host.isConnected) result.textContent = String(error);
    }
  });
  host.querySelector("[data-public-preview-button]")!.addEventListener("click", async () => {
    const current = ++request;
    result.textContent = "Loading current public page…";
    try {
      const display = await invoke<Display | null>("workspace_public_entry", args);
      if (current !== request || !host.isConnected) return;
      result.replaceChildren();
      if (!display) result.textContent = "Unpublished at this date.";
      function render(node: Display) {
        if (node.content.Text !== undefined) {
          const p = document.createElement("p");
          p.textContent = node.content.Text;
          result.append(p);
        }
        node.content.Stack?.forEach(render);
      }
      if (display) render(display);
    } catch (error) {
      if (current === request && host.isConnected) result.textContent = String(error);
    }
  });
  try {
    const saved = await invoke<[Document, boolean] | null>("workspace_entry_layout", { ...args, revision });
    const block = (audience: string): Block => ({ id: crypto.randomUUID(), audience, content: { Text: "" } });
    const layout: Document = saved?.[0] ?? { schema_version: 1, entry: { world: worldId, id: entryId },
      root: { id: crypto.randomUUID(), audience: "Public", content: { Stack: [block("Public"), block("Author")] } } };
    const children = layout.root.content.Stack;
    if (layout.root.audience !== "Public" || !children || children.length < 2 || children.length > 3 ||
        children[0].audience !== "Public" || children[1].audience !== "Author" ||
        typeof children[0].content.Text !== "string" || typeof children[1].content.Text !== "string" ||
        (children.length === 3 && (children[2].audience !== "Author" || !children[2].content.SavedQuery))) {
      result.textContent = "This layout uses components outside the character page editor.";
      return;
    }
    body.value = children[0].content.Text;
    notes.value = children[1].content.Text;
    published.checked = saved?.[1] ?? false;
    const original = children[2]?.content.SavedQuery;
    const key = (id: string, revision: number) => id + "@" + revision;
    const definitions = await invoke<SavedQuery[]>("workspace_saved_queries", { worldId, revision });
    if (!host.isConnected) return;
    if (original && !definitions.some(query => query.id === original.query_id &&
        query.recorded_at === original.definition_revision)) {
      const historical = await invoke<SavedQuery[]>("workspace_saved_queries",
        { worldId, revision: original.definition_revision });
      const pinned = historical.find(query => query.id === original.query_id &&
        query.recorded_at === original.definition_revision);
      if (!pinned) {
        result.textContent = "The pinned saved query is unavailable. Layout left unchanged.";
        return;
      }
      definitions.push(pinned);
    }
    if (!host.isConnected) return;
    const choices = new Map(definitions.map(query => [key(query.id, query.recorded_at), query]));
    for (const [value, query] of choices) querySelect.add(new Option(
      query.name + " · definition revision " + query.recorded_at, value));
    if (original) querySelect.value = key(original.query_id, original.definition_revision);
    const parameterInputs = new Map<string, HTMLInputElement | HTMLSelectElement>();
    function renderParameters(useOriginal: boolean) {
      parameterInputs.clear();
      parameterHost.replaceChildren();
      const query = choices.get(querySelect.value);
      for (const parameter of query?.parameters ?? []) {
        const label = document.createElement("label");
        label.textContent = parameter.name.replaceAll("_", " ");
        const selected = useOriginal && original
          ? original.bindings.find(binding => binding.name === parameter.name)?.value ?? parameter.default
          : parameter.default;
        let control: HTMLInputElement | HTMLSelectElement;
        if (parameter.default.type === "object_kind") {
          control = document.createElement("select");
          for (const kind of ["person", "definition", "relationship", "event_involvement", "actor_action",
            "historical_event", "source", "claim"]) control.add(new Option(kind.replaceAll("_", " "), kind));
        } else {
          control = document.createElement("input");
          control.maxLength = parameter.default.type === "source" ? 36 : 800;
        }
        control.required = true;
        control.value = selected.type === "source" ? selected.source_id : selected.value;
        label.append(control);
        parameterHost.append(label);
        parameterInputs.set(parameter.name, control);
      }
    }
    renderParameters(true);
    querySelect.addEventListener("change", () => { clearAuthorPreview(); renderParameters(false); });
    const queryElementId = children[2]?.id ?? crypto.randomUUID();
    function draft(): Document {
      const query = choices.get(querySelect.value);
      const blocks: Block[] = [
        { ...children![0], content: { Text: body.value } },
        { ...children![1], content: { Text: notes.value } },
      ];
      if (query) {
        const bindings = (query.parameters ?? []).map(parameter => {
          const value = parameterInputs.get(parameter.name)!.value;
          const bound: ParameterValue = parameter.default.type === "source"
            ? { type: "source", world_id: worldId, source_id: value }
            : { type: parameter.default.type, value };
          return { name: parameter.name, value: bound };
        });
        blocks.push({ id: queryElementId, audience: "Author", content: { SavedQuery: {
          world: worldId, query_id: query.id, definition_revision: query.recorded_at, bindings,
        } } });
      }
      return { ...layout, root: { ...layout.root, content: { Stack: blocks } } };
    }
    authorPreview.disabled = false;
    authorPreview.addEventListener("click", async () => {
      if (editable && !form.reportValidity()) return;
      clearAuthorPreview();
      const sequence = authorSequence;
      const requestId = crypto.randomUUID();
      activeAuthor = requestId;
      authorResult.textContent = "Compiling private author preview…";
      try {
        const display = await invoke<Display | null>("workspace_preview_author_layout",
          { worldId, day, revision, document: draft(), requestId });
        if (sequence !== authorSequence || !host.isConnected) return;
        authorResult.replaceChildren();
        function render(node: Display) {
          if (node.content.Text !== undefined) {
            const p = document.createElement("p");
            p.textContent = node.content.Text;
            authorResult.append(p);
          }
          if (node.content.QueryRows) {
            const table = document.createElement("table");
            const caption = table.createCaption();
            caption.textContent = "Private query table" + (node.content.QueryRows.truncated ? " · limited results" : "");
            const header = table.createTHead().insertRow();
            for (const title of ["Name", "Stable identity"]) {
              const th = document.createElement("th");
              th.scope = "col";
              th.textContent = title;
              header.append(th);
            }
            const tbody = table.createTBody();
            for (const [object, label] of node.content.QueryRows.records) {
              const row = tbody.insertRow();
              row.insertCell().textContent = label;
              row.insertCell().textContent = object.id;
            }
            if (!tbody.rows.length) {
              const cell = tbody.insertRow().insertCell();
              cell.colSpan = 2;
              cell.textContent = "No matching records.";
            }
            authorResult.append(table);
          }
          node.content.Stack?.forEach(render);
        }
        if (display) render(display);
      } catch (error) {
        if (sequence === authorSequence && host.isConnected) authorResult.textContent = String(error);
      } finally {
        if (sequence === authorSequence) activeAuthor = undefined;
      }
    });
    fields.disabled = !editable;
    form.addEventListener("submit", async event => {
      event.preventDefault();
      if (fields.disabled) return;
      const document = draft();
      clearAuthorPreview();
      fields.disabled = true;
      authorPreview.disabled = true;
      try {
        await invoke("workspace_save_layout", { worldId, day, expectedRevision: revision, document, published: published.checked });
        if (host.isConnected) await afterSave();
      } catch (error) { result.textContent = String(error); fields.disabled = !editable; authorPreview.disabled = false; }
    });
  } catch (error) { result.textContent = String(error); }
}
