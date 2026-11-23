import { invoke } from "@tauri-apps/api/core";

export type ObjectRef = { world: string; id: string };
export type ObjectLink = { target: ObjectRef; label: string };
export type ProjectedImage = {width:number;height:number;png_base64:string;alt:string;hotspots:{id:string;x:number;y:number;link:ObjectLink}[]};
export type PublicDisplay = { id?: string; content: { Text?: string; Stack?: PublicDisplay[]; Panel?: PublicDisplay[];
  Grid?: { columns: number; children: PublicDisplay[] }; ObjectLink?: ObjectLink;
  Image?: ProjectedImage;
  StatementRows?: { statements: string[]; truncated: boolean }; ReferenceRows?: { links: ObjectLink[]; truncated: boolean } } };

// Both reader and author preview render only the projected children, in layout order.
export function layoutContainer(parent: HTMLElement, kind: "panel" | "grid", columns = 1) {
  const container = document.createElement("div");
  container.className = "entry-" + kind;
  if (kind === "grid") container.style.setProperty("--entry-columns", String(Math.max(1, Math.min(4, columns))));
  parent.append(container);
  return container;
}
// Only the Rust projection supplies bytes and hotspot labels.
export function renderProjectedImage(parent:HTMLElement,image:ProjectedImage,open:(link:ObjectLink)=>void){
  const figure=document.createElement("figure");
  figure.className="entry-image";
  const surface=document.createElement("div");
  surface.className="entry-image-surface";
  const img=document.createElement("img");
  img.src="data:image/png;base64,"+image.png_base64;
  img.alt=image.alt;img.width=image.width;img.height=image.height;surface.append(img);
  const links=document.createElement("ol");links.className="image-hotspot-list";
  for(const [index,hotspot] of image.hotspots.entries()){
    const marker=document.createElement("button");
    marker.type="button";marker.textContent=String(index+1);
    marker.setAttribute("aria-label",hotspot.link.label);
    marker.className="image-hotspot";
    marker.style.left=hotspot.x/100+"%";marker.style.top=hotspot.y/100+"%";
    marker.onclick=()=>open(hotspot.link);surface.append(marker);
    const row=document.createElement("li"),link=document.createElement("button");link.type="button";
    link.textContent=hotspot.link.label;link.onclick=()=>open(hotspot.link);row.append(link);links.append(row);
  }
  figure.append(surface);
  const caption=document.createElement("figcaption");caption.textContent=image.alt;figure.append(caption);
  if(image.hotspots.length)figure.append(links);
  parent.append(figure);
}
export type Backlinks = { matches: { source: ObjectRef; element: string; label: string }[]; truncated: boolean };

// Navigation never reads author layouts. History stores IDs, not page contents.
export function publicReader(host: HTMLElement, worldId: string, day: number,
  valid: () => boolean, unavailable: () => void) {
  let sequence = 0;
  let history: string[] = [];
  async function load(entryId: string, back = false, anchor?: string) {
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
      function linkButton(label: string, target: ObjectRef, parent = host) {
        if (target.world !== worldId) return;
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = label;
        button.addEventListener("click", () => { if (valid()) void load(target.id); });
        parent.append(button);
      }
      const anchors = new Map<string, HTMLElement>();
      function render(node: PublicDisplay, parent = host) {
        const previous = parent.lastElementChild;
        if (node.content.Text !== undefined) {
          const p = document.createElement("p");
          p.textContent = node.content.Text;
          parent.append(p);
        }
        if (node.content.ObjectLink) linkButton(node.content.ObjectLink.label, node.content.ObjectLink.target, parent);
        if (node.content.Image) renderProjectedImage(parent,node.content.Image,link=>{if(valid()&&link.target.world===worldId)void load(link.target.id);});
        if (node.content.StatementRows) {
          const group = layoutContainer(parent, "panel");
          const title = document.createElement("p");
          title.textContent = "Released historical statements" + (node.content.StatementRows.truncated ? " · limited results" : "");
          group.append(title);
          for (const statement of node.content.StatementRows.statements) {
            const row = document.createElement("p"); row.textContent = statement; group.append(row);
          }
          if (!node.content.StatementRows.statements.length) group.append("No released statements.");
        }
        if (node.content.ReferenceRows) {
          const group = layoutContainer(parent, "panel");
          const title = document.createElement("p");
          title.textContent = "Published references" + (node.content.ReferenceRows.truncated ? " · limited results" : "");
          group.append(title);
          for (const link of node.content.ReferenceRows.links) linkButton("Referenced by: " + link.label, link.target, group);
          if (!node.content.ReferenceRows.links.length) group.append("No published references.");
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
        const rendered = previous ? previous.nextElementSibling : parent.firstElementChild;
        if (node.id && rendered instanceof HTMLElement) anchors.set(node.id, rendered);
      }
      render(page);
      if (anchor) {
        const destination = anchors.get(anchor);
        if (destination) {
          destination.tabIndex = -1;
          destination.focus();
          destination.scrollIntoView({ block: "nearest" });
        } else {
          const notice = document.createElement("p");
          notice.setAttribute("role", "status");
          notice.textContent = "Matching section changed or is no longer visible. Search again.";
          host.append(notice);
        }
      }
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
  return { open: (entryId: string, anchor?: string) => { history = []; return load(entryId, false, anchor); } };
}
