import { invoke } from "@tauri-apps/api/core";

type Block = { id: string; audience: string; content: { Text?: string; Stack?: Block[] } };
type Document = { schema_version: number; entry: { world: string; id: string }; root: Block };
type Display = { content: { Text?: string; Stack?: Display[] } };

export async function openEntryEditor(panel: HTMLElement, worldId: string, entryId: string,
  day: number, revision: number, editable: boolean, afterSave: () => Promise<void>) {
  const host = document.createElement("section");
  panel.append(host);
  host.innerHTML = `<h3>Character page</h3><form class="editor-form"><fieldset disabled>
    <label>Page text <textarea name="body" required></textarea></label>
    <label>Author notes <textarea name="notes"></textarea></label>
    <label><input type="checkbox" name="published"> Publish page text</label>
    <button>Save page</button></fieldset></form>
    <button type="button">View current public page</button><div aria-live="polite"></div>`;
  const form = host.querySelector("form")!;
  const fields = host.querySelector("fieldset")!;
  const body = form.elements.namedItem("body") as HTMLTextAreaElement;
  const notes = form.elements.namedItem("notes") as HTMLTextAreaElement;
  const published = form.elements.namedItem("published") as HTMLInputElement;
  const result = host.querySelector("div")!;
  const args = { worldId, entryId, day };
  host.querySelector("button[type=button]")!.addEventListener("click", async () => {
    try {
      const display = await invoke<Display | null>("workspace_public_entry", args);
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
    } catch (error) { result.textContent = String(error); }
  });
  try {
    const saved = await invoke<[Document, boolean] | null>("workspace_entry_layout", { ...args, revision });
    const block = (audience: string): Block => ({ id: crypto.randomUUID(), audience, content: { Text: "" } });
    const layout: Document = saved?.[0] ?? { schema_version: 1, entry: { world: worldId, id: entryId },
      root: { id: crypto.randomUUID(), audience: "Public", content: { Stack: [block("Public"), block("Author")] } } };
    const children = layout.root.content.Stack;
    if (layout.root.audience !== "Public" || children?.length !== 2 ||
        children[0].audience !== "Public" || children[1].audience !== "Author" ||
        typeof children[0].content.Text !== "string" || typeof children[1].content.Text !== "string") {
      result.textContent = "This layout uses components outside the character page editor.";
      return;
    }
    body.value = children[0].content.Text;
    notes.value = children[1].content.Text;
    published.checked = saved?.[1] ?? false;
    fields.disabled = !editable;
    form.addEventListener("submit", async event => {
      event.preventDefault();
      if (fields.disabled) return;
      fields.disabled = true;
      children[0].content.Text = body.value;
      children[1].content.Text = notes.value;
      try {
        await invoke("workspace_save_layout", { worldId, day, expectedRevision: revision, document: layout, published: published.checked });
        if (host.isConnected) await afterSave();
      } catch (error) { result.textContent = String(error); fields.disabled = !editable; }
    });
  } catch (error) { result.textContent = String(error); }
}
