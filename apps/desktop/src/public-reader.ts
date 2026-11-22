import { invoke } from "@tauri-apps/api/core";

export type ObjectRef = { world: string; id: string };
export type ObjectLink = { target: ObjectRef; label: string };
export type PublicDisplay = { content: { Text?: string; Stack?: PublicDisplay[]; ObjectLink?: ObjectLink } };
export type Backlinks = { matches: { source: ObjectRef; element: string; label: string }[]; truncated: boolean };

// Navigation never reads author layouts. History stores IDs, not page contents.
export function publicReader(host: HTMLElement, worldId: string, day: number,
  valid: () => boolean, unavailable: () => void) {
  let sequence = 0;
  let history: string[] = [];
  async function load(entryId: string, back = false) {
    const current = ++sequence;
    host.textContent = "Loading current publication…";
    try {
      const page = await invoke<PublicDisplay | null>("workspace_public_entry", { worldId, day, entryId });
      if (current !== sequence || !valid()) return;
      if (!page) { history = []; host.textContent = "Unpublished at this date."; unavailable(); return; }
      if (back) history.pop();
      else if (history[history.length - 1] !== entryId) {
        history.push(entryId);
        if (history.length > 64) history.shift();
      }
      host.replaceChildren();
      const navigation = document.createElement("nav");
      navigation.setAttribute("aria-label", "Published page navigation");
      const previous = document.createElement("button");
      previous.type = "button";
      previous.textContent = "Back to previous page";
      previous.disabled = history.length < 2;
      previous.addEventListener("click", () => { if (valid()) void load(history[history.length - 2], true); });
      navigation.append(previous);
      host.append(navigation);
      function linkButton(label: string, target: ObjectRef) {
        if (target.world !== worldId) return;
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = label;
        button.addEventListener("click", () => { if (valid()) void load(target.id); });
        host.append(button);
      }
      function render(node: PublicDisplay) {
        if (node.content.Text !== undefined) {
          const p = document.createElement("p");
          p.textContent = node.content.Text;
          host.append(p);
        }
        if (node.content.ObjectLink) linkButton(node.content.ObjectLink.label, node.content.ObjectLink.target);
        node.content.Stack?.forEach(render);
      }
      render(page);
      const incoming = document.createElement("button");
      incoming.type = "button";
      incoming.textContent = "Find published backlinks";
      host.append(incoming);
      incoming.addEventListener("click", async () => {
        if (!valid()) return;
        incoming.disabled = true;
        try {
          const result = await invoke<Backlinks>("workspace_public_entry_backlinks",
            { worldId, day, entryId, requestId: crypto.randomUUID() });
          if (current !== sequence || !valid()) return;
          incoming.remove();
          const heading = document.createElement("p");
          heading.textContent = result.truncated ? "Published backlinks · first 50" :
            result.matches.length + " published backlinks";
          host.append(heading);
          for (const match of result.matches) linkButton("Referenced by: " + match.label, match.source);
        } catch (error) {
          if (current === sequence && valid()) {
            incoming.disabled = false;
            const message = document.createElement("p");
            message.textContent = String(error);
            host.append(message);
          }
        }
      });
    } catch (error) {
      if (current === sequence && valid()) host.textContent = String(error);
    }
  }
  return { open: (entryId: string) => { history = []; return load(entryId); } };
}
