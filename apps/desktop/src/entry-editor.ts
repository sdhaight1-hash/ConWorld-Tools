import { invoke } from "@tauri-apps/api/core";

import type { ParameterValue, SavedQuery } from "./author-workspace";
import { publicReader, layoutContainer, type ObjectLink, type Backlinks } from "./public-reader";
import { editCanvasTree } from "./canvas-tree";

export type Binding = { world: string; query_id: string; definition_revision: number;
  bindings: { name: string; value: ParameterValue }[] };
export type Block = { id: string; audience: string; content: { Text?: string; Stack?: Block[]; Panel?: Block[];
  Grid?: { columns: number; children: Block[] }; Template?: { key: string; version: number; children: Block[] };
  SavedQuery?: Binding; ObjectLink?: ObjectLink; Query?: unknown; FamilyTable?: { world: string; id: string }; NetworkTable?: { world: string; id: string };
  PublishedReferences?: { world: string; id: string } } };
type Document = { schema_version: number; entry: { world: string; id: string }; root: Block };
type Display = { content: { Text?: string; Stack?: Display[]; Panel?: Display[];
  Grid?: { columns: number; children: Display[] }; ObjectLink?: ObjectLink;
  ReferenceRows?: { links: ObjectLink[]; truncated: boolean };
  QueryRows?: { records: [{ world: string; id: string }, string][]; truncated: boolean } } };
type EntrySearch = { matches: [string, string][]; truncated: boolean };

export async function openEntryEditor(panel: HTMLElement, worldId: string, entryId: string,
  day: number, revision: number, editable: boolean, afterSave: () => Promise<void>,
  targets: { id: string; label: string }[] = [],
  openAuthor?: (id: string) => void) {
  const host = document.createElement("section");
  panel.append(host);
  host.innerHTML = `<h3 data-entry-heading></h3><p data-entry-identity class="hint"></p><form class="editor-form"><fieldset disabled>
    <button type="button" data-open-tree>Open Canvas tree</button>
    <fieldset data-simple-fields>
    <label>Page arrangement <select name="arrangement">
      <option value="stack">Stack</option><option value="panel">Panel</option>
      <option value="2">Two-column grid</option><option value="3">Three-column grid</option>
      <option value="4">Four-column grid</option></select></label>
    <label>Page text <textarea name="body" required></textarea></label>
    <label>Author notes <textarea name="notes"></textarea></label>
    <label>Private saved query <select name="saved_query"><option value="">No query table</option></select></label>
    <p class="hint">Author only. Pin a definition revision; results follow the selected historical view.</p>
    <div data-binding-parameters></div>
    <h4>Entry links</h4><div data-entry-links></div>
    <button type="button" data-add-link>Add entry link</button>
    <p class="hint">A public link appears only when its target page is released. Private links remain author-only.</p>
    </fieldset><div data-tree-host hidden></div>
    <label><input type="checkbox" name="published"> Publish page text</label>
    <button>Save page</button></fieldset></form>
    <button type="button" data-author-preview-button disabled>Preview author layout</button>
    <div data-author-preview aria-live="polite"></div>
    <button type="button" data-author-backlinks>Find authored backlinks</button>
    <div data-author-backlink-results aria-live="polite"></div>
    <button type="button" data-public-preview-button>View current public page</button>
    <form class="editor-form" data-public-search>
      <label>Find in public page <input name="text" required maxlength="256"></label>
      <button>Find</button>
    </form><div data-public-result aria-live="polite"></div>`;
  host.querySelector("[data-entry-heading]")!.textContent =
    "Entry page · " + (targets.find(target => target.id === entryId)?.label ?? entryId);
  host.querySelector("[data-entry-identity]")!.textContent = "Stable identity: " + entryId;
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
  const incoming = host.querySelector<HTMLButtonElement>("[data-author-backlinks]")!;
  const incomingResults = host.querySelector<HTMLElement>("[data-author-backlink-results]")!;
  incoming.addEventListener("click", async () => {
    incoming.disabled = true;
    incomingResults.textContent = "Finding authored references…";
    try {
      const found = await invoke<Backlinks>("workspace_author_entry_backlinks",
        { ...args, revision, requestId: crypto.randomUUID() });
      if (!host.isConnected) return;
      const summary = document.createElement("p");
      summary.textContent = found.truncated ? "First 50 authored references." :
        found.matches.length + " authored references.";
      incomingResults.replaceChildren(summary);
      for (const match of found.matches) {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = match.label + " · " + match.source.id;
        button.disabled = !openAuthor;
        button.addEventListener("click", () => openAuthor?.(match.source.id));
        incomingResults.append(button);
      }
    } catch (error) {
      if (host.isConnected) incomingResults.textContent = String(error);
    } finally { incoming.disabled = false; }
  });
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
  host.querySelector("[data-public-preview-button]")!.addEventListener("click", () => {
    const current = ++request;
    void publicReader(result, worldId, day, () => current === request && host.isConnected, () => {}).open(entryId);
  });
  try {
    const saved = await invoke<[Document, boolean] | null>("workspace_entry_layout", { ...args, revision });
    const block = (audience: string): Block => ({ id: crypto.randomUUID(), audience, content: { Text: "" } });
    const layout: Document = saved?.[0] ?? { schema_version: 1, entry: { world: worldId, id: entryId },
      root: { id: crypto.randomUUID(), audience: "Public", content: { Stack: [block("Public"), block("Author")] } } };
    if (!host.isConnected) return;
    published.checked = saved?.[1] ?? false;
    const treeButton = host.querySelector<HTMLButtonElement>("[data-open-tree]")!;
    const simpleFields = host.querySelector<HTMLFieldSetElement>("[data-simple-fields]")!;
    const treeHost = host.querySelector<HTMLElement>("[data-tree-host]")!;
    let currentDraft = () => layout;
    function activateTree(document: Document) {
      clearAuthorPreview();
      simpleFields.disabled = true;
      simpleFields.hidden = true;
      treeButton.hidden = true;
      treeHost.hidden = false;
      const tree = editCanvasTree(treeHost, document.root, worldId, entryId, revision, editable, clearAuthorPreview);
      currentDraft = () => ({ ...document, root: tree.root() });
    }
    function installEditorActions() {
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
            { worldId, day, revision, document: currentDraft(), requestId });
          if (sequence !== authorSequence || !host.isConnected) return;
          authorResult.replaceChildren();
          function render(node: Display, parent = authorResult) {
            if (node.content.Text !== undefined) {
              const p = document.createElement("p");
              p.textContent = node.content.Text;
              parent.append(p);
            }
            if (node.content.ObjectLink) {
              const link = node.content.ObjectLink;
              const button = document.createElement("button");
              button.type = "button";
              button.textContent = link.label;
              button.disabled = !openAuthor || link.target.world !== worldId;
              button.addEventListener("click", () => openAuthor?.(link.target.id));
              parent.append(button);
            }
            if (node.content.ReferenceRows) {
              const group = layoutContainer(parent, "panel");
              const heading = document.createElement("p");
              heading.textContent = "Published references only";
              group.append(heading);
              for (const link of node.content.ReferenceRows.links) {
                const button = document.createElement("button");
                button.type = "button";
                button.textContent = link.label;
                button.disabled = !openAuthor || link.target.world !== worldId;
                button.onclick = () => openAuthor?.(link.target.id);
                group.append(button);
              }
              if (!node.content.ReferenceRows.links.length) group.append("No published references in this view.");
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
              parent.append(table);
            }
            node.content.Stack?.forEach(child => render(child, parent));
            if (node.content.Panel) {
              const container = layoutContainer(parent, "panel");
              node.content.Panel.forEach(child => render(child, container));
            }
            if (node.content.Grid) {
              const container = layoutContainer(parent, "grid", node.content.Grid.columns);
              node.content.Grid.children.forEach(child => render(child, container));
            }
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
        const document = currentDraft();
        ++request;
        result.replaceChildren();
        clearAuthorPreview();
        fields.disabled = true;
        authorPreview.disabled = true;
        try {
          await invoke("workspace_save_layout", { worldId, day, expectedRevision: revision, document, published: published.checked });
          if (host.isConnected) await afterSave();
        } catch (error) { result.textContent = String(error); fields.disabled = !editable; authorPreview.disabled = false; }
      });
    }
    const arrangement = form.elements.namedItem("arrangement") as HTMLSelectElement;
    const rootChildren = layout.root.content.Stack;
    const wrapper = rootChildren?.length === 1 && rootChildren[0].audience === "Public" &&
      (rootChildren[0].content.Panel || rootChildren[0].content.Grid) ? rootChildren[0] : undefined;
    const children = wrapper ? (wrapper.content.Panel ?? wrapper.content.Grid?.children) : rootChildren;
    const wrapperId = wrapper?.id ?? crypto.randomUUID();
    if (wrapper?.content.Grid) arrangement.value = String(wrapper.content.Grid.columns);
    else if (wrapper?.content.Panel) arrangement.value = "panel";
    const extras = children?.slice(2) ?? [];
    const savedQueries = extras.filter(block => block.content.SavedQuery !== undefined);
    if (layout.root.audience !== "Public" || !arrangement.value || !children || children.length < 2 ||
        children[0].audience !== "Public" || children[1].audience !== "Author" ||
        typeof children[0].content.Text !== "string" || typeof children[1].content.Text !== "string" ||
        savedQueries.length > 1 || extras.some(block => block.content.SavedQuery
          ? block.audience !== "Author"
          : !block.content.ObjectLink || !["Public", "Author"].includes(block.audience))) {
      activateTree(layout);
      installEditorActions();
      return;
    }
    body.value = children[0].content.Text;
    notes.value = children[1].content.Text;
    published.checked = saved?.[1] ?? false;
    const original = savedQueries[0]?.content.SavedQuery;
    const links = new Map<string, { target: HTMLInputElement; caption: HTMLInputElement; audience: HTMLSelectElement }>();
    const linkHost = host.querySelector<HTMLElement>("[data-entry-links]")!;
    const addLink = host.querySelector<HTMLButtonElement>("[data-add-link]")!;
    const listId = "link-targets-" + crypto.randomUUID();
    const datalist = document.createElement("datalist");
    datalist.id = listId;
    for (const target of targets) datalist.append(new Option(target.label, target.id));
    linkHost.append(datalist);
    function addLinkRow(block?: Block) {
      const id = block?.id ?? crypto.randomUUID();
      const link = block?.content.ObjectLink;
      const row = document.createElement("fieldset");
      const target = document.createElement("input");
      target.setAttribute("list", listId);
      target.required = true;
      target.value = link?.target.id ?? "";
      const caption = document.createElement("input");
      caption.required = true;
      caption.maxLength = 800;
      caption.value = link?.label ?? "";
      const audience = document.createElement("select");
      audience.add(new Option("Private", "Author"));
      audience.add(new Option("Public", "Public"));
      audience.value = block?.audience ?? "Author";
      for (const [text, element] of [["Link target identity", target], ["Link caption", caption],
        ["Link audience", audience]] as const) {
        const label = document.createElement("label");
        label.textContent = text;
        label.append(element);
        row.append(label);
      }
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "Remove entry link";
      remove.addEventListener("click", () => {
        links.delete(id); row.remove(); addLink.disabled = false; clearAuthorPreview();
      });
      row.append(remove);
      links.set(id, { target, caption, audience });
      linkHost.append(row);
      addLink.disabled = links.size >= 64;
    }
    for (const block of extras.filter(block => block.content.ObjectLink)) addLinkRow(block);
    addLink.addEventListener("click", () => { if (links.size < 64) { addLinkRow(); clearAuthorPreview(); } });
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
        activateTree(layout);
        installEditorActions();
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
          for (const kind of ["person", "organization", "household", "family_house", "definition", "relationship", "event_involvement", "actor_action",
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
    const queryElementId = savedQueries[0]?.id ?? crypto.randomUUID();
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
      for (const [id, row] of links) blocks.push({ id, audience: row.audience.value,
        content: { ObjectLink: { target: { world: worldId, id: row.target.value.trim() }, label: row.caption.value } } });
      // Retain existing component order and identities, append only new blocks.
      const order = new Map(children!.map((block, i) => [block.id, i]));
      blocks.sort((a, b) => (order.get(a.id) ?? children!.length) - (order.get(b.id) ?? children!.length));
      const arranged: Block[] = arrangement.value === "stack" ? blocks : [{
        id: wrapperId, audience: "Public", content: arrangement.value === "panel"
          ? { Panel: blocks } : { Grid: { columns: Number(arrangement.value), children: blocks } },
      }];
      return { ...layout, root: { ...layout.root, content: { Stack: arranged } } };
    }
    currentDraft = draft;
    treeButton.addEventListener("click", () => {
      if (!form.reportValidity()) return;
      activateTree(currentDraft());
    });
    installEditorActions();
  } catch (error) { result.textContent = String(error); }
}
