import { invoke } from "@tauri-apps/api/core";
import type { Block, Binding } from "./entry-editor";
import type { SavedQuery, ParameterValue } from "./author-workspace";
import { templates, instantiateTemplate, historicalComponent } from "./canvas-templates";

// This is a draft-only editor. The Rust compiler/store validates the entire tree
// before save and owns publication; no new persistence or authority lives here.
function children(block: Block): Block[] | undefined {
  return block.content.Stack ?? block.content.Panel ?? block.content.Grid?.children ?? block.content.Template?.children;
}

function kind(block: Block): string {
  return Object.keys(block.content)[0] ?? "Unsupported";
}

function restrict(audience: string, enclosing: string): string {
  if (audience === "Public") return enclosing;
  if (enclosing === "Public" || JSON.stringify(audience) === JSON.stringify(enclosing)) return audience;
  return "Author";
}

type Row = { block: Block; parent?: Row; depth: number; effective: string };

export function editCanvasTree(host: HTMLElement, original: Block, worldId: string,
  entryId: string, revision: number, editable: boolean, changed: () => void) {
  const root = structuredClone(original);
  let selected = root.id;
  const definitions = new Map<string, SavedQuery>();
  const loaded = new Set<number>();
  const key = (id: string, rev: number) => id + "@" + rev;
  let queryError = "";
  async function loadQueries(at: number) {
    if (loaded.has(at)) return;
    loaded.add(at);
    try {
      const queries = await invoke<SavedQuery[]>("workspace_saved_queries", { worldId, revision: at });
      for (const query of queries) definitions.set(key(query.id, query.recorded_at), query);
    } catch (error) {
      queryError = String(error);
    }
    if (host.isConnected) render();
  }
  function newBinding(query: SavedQuery): Binding {
    return { world: worldId, query_id: query.id, definition_revision: query.recorded_at, bindings: [] };
  }
  const message = document.createElement("p");
  message.setAttribute("role", "status");
  const heading = document.createElement("h4");
  heading.textContent = "Canvas tree";
  const hint = document.createElement("p");
  hint.className = "hint";
  hint.textContent = "Select a component to edit or move it. Moving a component retains inherited privacy; new components start private.";
  const tree = document.createElement("div");
  tree.className = "canvas-tree";
  const controls = document.createElement("div");
  controls.className = "canvas-tree-controls";
  host.replaceChildren(heading, hint, tree, controls, message);

  function rows(): Row[] {
    const result: Row[] = [];
    function visit(block: Block, depth: number, parent?: Row) {
      const row = { block, depth, parent, effective: restrict(block.audience, parent?.effective ?? "Public") };
      result.push(row);
      children(block)?.forEach(child => visit(child, depth + 1, row));
    }
    visit(root, 0);
    return result;
  }

  function button(label: string, handler: () => void, disabled = false) {
    const control = document.createElement("button");
    control.type = "button";
    control.textContent = label;
    control.disabled = disabled;
    control.addEventListener("click", handler);
    return control;
  }

  function field(label: string, control: HTMLElement) {
    const wrapper = document.createElement("label");
    wrapper.textContent = label;
    wrapper.append(control);
    controls.append(wrapper);
  }

  function commit(note: string) {
    message.textContent = note;
    changed();
    render();
  }

  function render() {
    const all = rows();
    const row = all.find(row => row.block.id === selected) ?? all[0];
    selected = row.block.id;
    const node = row.block;
    const siblings = row.parent && children(row.parent.block);
    const index = siblings?.indexOf(node) ?? -1;
    tree.replaceChildren();
    controls.replaceChildren();
    for (const item of all) {
      const line = button(kind(item.block) + " · " +
        (item.block.content.Text?.slice(0, 38) || item.block.content.ObjectLink?.label
          || item.block.content.Template?.key || item.block.id.slice(0, 8)), () => {
        selected = item.block.id;
        render();
      });
      line.style.marginInlineStart = Math.min(item.depth, 8) * 0.6 + "rem";
      line.setAttribute("aria-pressed", String(item.block.id === selected));
      line.title = item.block.id;
      tree.append(line);
    }

    const identity = document.createElement("p");
    identity.textContent = "Selected: " + kind(node) + " · " + node.id;
    controls.append(identity);
    if (node.content.Text !== undefined) {
      const text = document.createElement("textarea");
      text.value = node.content.Text;
      text.maxLength = 16_384;
      text.readOnly = !editable;
      text.addEventListener("input", () => { node.content.Text = text.value; changed(); });
      field("Component text", text);
    }
    if (node.content.ObjectLink) {
      const link = node.content.ObjectLink;
      const target = document.createElement("input");
      target.value = link.target.id;
      target.required = true;
      target.readOnly = !editable;
      target.addEventListener("input", () => { link.target.id = target.value.trim(); changed(); });
      field("Component link target", target);
      const caption = document.createElement("input");
      caption.value = link.label;
      caption.required = true;
      caption.maxLength = 800;
      caption.readOnly = !editable;
      caption.addEventListener("input", () => { link.label = caption.value; changed(); });
      field("Component link caption", caption);
    }
    if (node.content.Grid) {
      const columns = document.createElement("select");
      for (const value of [1, 2, 3, 4]) columns.add(new Option(String(value), String(value)));
      columns.value = String(node.content.Grid.columns);
      columns.disabled = !editable;
      columns.addEventListener("change", () => { node.content.Grid!.columns = Number(columns.value); changed(); });
      field("Grid columns", columns);
    }
    if (node.content.Template) {
      const pin = document.createElement("p");
      pin.textContent = "Template: " + node.content.Template.key + " · version " + node.content.Template.version;
      controls.append(pin);
    }
    if (node.content.FamilyTable) {
      const target = document.createElement("input");
      target.value = node.content.FamilyTable.id;
      target.required = true;
      target.readOnly = !editable;
      target.addEventListener("input", () => { node.content.FamilyTable!.id = target.value.trim(); changed(); });
      field("Family subject identity", target);
      const description = document.createElement("p");
      description.textContent = "Accepted biological, adoptive and legal links at the selected date. Author only.";
      controls.append(description);
    }
    if (node.content.Query) {
      const description = document.createElement("p");
      description.textContent = "Historical query · follows the selected day and revision. Author only.";
      controls.append(description);
    }
    if (node.content.SavedQuery) {
      const binding = node.content.SavedQuery;
      const pinned = key(binding.query_id, binding.definition_revision);
      if (!definitions.has(pinned)) void loadQueries(binding.definition_revision);
      const choices = document.createElement("select");
      if (!definitions.has(pinned)) choices.add(new Option("Unavailable pin · " + pinned, pinned));
      for (const [value, definition] of definitions) {
        choices.add(new Option(definition.name + " · revision " + definition.recorded_at, value));
      }
      choices.value = pinned;
      choices.disabled = !editable;
      choices.addEventListener("change", () => {
        const definition = definitions.get(choices.value);
        if (!definition) return;
        node.content.SavedQuery = newBinding(definition);
        node.audience = "Author";
        commit("Query definition changed explicitly.");
      });
      field("Saved query definition", choices);
      for (const parameter of definitions.get(pinned)?.parameters ?? []) {
        const value = binding.bindings.find(item => item.name === parameter.name)?.value ?? parameter.default;
        let input: HTMLInputElement | HTMLSelectElement;
        if (parameter.default.type === "object_kind") {
          input = document.createElement("select");
          for (const kind of ["person", "definition", "relationship", "event_involvement", "actor_action",
            "historical_event", "source", "claim"]) input.add(new Option(kind.replaceAll("_", " "), kind));
        } else {
          input = document.createElement("input");
          input.maxLength = parameter.default.type === "source" ? 36 : 800;
        }
        input.value = value.type === "source" ? value.source_id : value.value;
        input.required = true;
        input.disabled = !editable;
        input.addEventListener("input", () => {
          const bound: ParameterValue = parameter.default.type === "source"
            ? { type: "source", world_id: worldId, source_id: input.value }
            : { type: parameter.default.type, value: input.value };
          binding.bindings = binding.bindings.filter(item => item.name !== parameter.name);
          binding.bindings.push({ name: parameter.name, value: bound });
          changed();
        });
        field("Query parameter " + parameter.name, input);
      }
    }
    if (queryError) {
      const warning = document.createElement("p");
      warning.textContent = "Saved query definitions unavailable: " + queryError;
      controls.append(warning);
    }
    const audience = document.createElement("select");
    audience.add(new Option("Private", "Author"));
    audience.add(new Option("Public", "Public"));
    if (node.audience !== "Author" && node.audience !== "Public") {
      audience.add(new Option("Existing character audience", "character"));
      audience.value = "character";
    } else audience.value = node.audience;
    audience.disabled = !editable || node === root || !!node.content.SavedQuery || !!node.content.Query || !!node.content.FamilyTable;
    audience.addEventListener("change", () => {
      if (audience.value === "character") return;
      node.audience = audience.value;
      commit("Audience changed explicitly. Publication is validated on save.");
    });
    field("Component audience", audience);
    if (!editable) return;

    const slot = children(node);
    if (slot) {
      const add = document.createElement("select");
      for (const label of ["Text", "Stack", "Panel", "Grid", "ObjectLink"]) add.add(new Option(label, label));
      if (definitions.size) add.add(new Option("Saved query table", "SavedQuery"));
      field("New component kind", add);
      controls.append(button("Add child", () => {
        const first = definitions.values().next().value;
        const content: Block["content"] = add.value === "SavedQuery" && first ? { SavedQuery: newBinding(first) } :
          add.value === "Text" ? { Text: "" } :
          add.value === "Stack" ? { Stack: [] } : add.value === "Panel" ? { Panel: [] } :
            add.value === "Grid" ? { Grid: { columns: 2, children: [] } } :
              { ObjectLink: { target: { world: worldId, id: "" }, label: "" } };
        const child: Block = { id: crypto.randomUUID(), audience: "Author", content };
        slot.push(child);
        selected = child.id;
        commit("Added private " + add.value + ".");
      }, all.length >= 512 || row.depth >= 16));
      const templateChoice = document.createElement("select");
      for (const item of templates) templateChoice.add(new Option(item.label + " · v1", item.key));
      field("Canvas template", templateChoice);
      controls.append(button("Insert template", () => {
        const template = instantiateTemplate(templateChoice.value, worldId, entryId);
        if (!template) return;
        slot.push(template);
        selected = template.id;
        commit("Inserted version-pinned template; existing components unchanged.");
      }, all.length > 494 || row.depth > 12));
      const history = document.createElement("select");
      for (const [value, label] of [["person_events", "Person event history"], ["family", "Accepted family links"], ["actors", "Event actors"],
        ["witnesses", "Event witnesses"], ["victims", "Event victims"], ["stakeholders", "Event stakeholders"]]) {
        history.add(new Option(label, value));
      }
      field("Historical component", history);
      controls.append(button("Add historical component", () => {
        const child = historicalComponent(history.value, worldId, entryId);
        if (!child) return;
        slot.push(child);
        selected = child.id;
        commit("Added private live historical query.");
      }, all.length >= 512 || row.depth >= 16));
    }
    if (!siblings || !row.parent) return;
    controls.append(
      button("Move up", () => {
        [siblings[index - 1], siblings[index]] = [siblings[index], siblings[index - 1]];
        commit("Moved component up.");
      }, index === 0),
      button("Move down", () => {
        [siblings[index + 1], siblings[index]] = [siblings[index], siblings[index + 1]];
        commit("Moved component down.");
      }, index === siblings.length - 1),
    );
    function contains(ancestor: Block, id: string): boolean {
      return ancestor.id === id || (children(ancestor)?.some(child => contains(child, id)) ?? false);
    }
    function height(block: Block): number {
      return Math.max(0, ...(children(block) ?? []).map(child => 1 + height(child)));
    }
    const targets = all.filter(target => children(target.block) && target !== row.parent &&
      !contains(node, target.block.id) && target.depth + 1 + height(node) <= 16);
    const destination = document.createElement("select");
    for (const target of targets) destination.add(new Option(kind(target.block) + " · " + target.block.id.slice(0, 8), target.block.id));
    field("Move into container", destination);
    controls.append(button("Move component", () => {
      const target = targets.find(target => target.block.id === destination.value);
      if (!target) return;
      // Intersect source ancestry before detaching, matching Rust reparent().
      node.audience = restrict(node.audience, row.parent!.effective);
      siblings.splice(index, 1);
      children(target.block)!.push(node);
      commit("Moved component; identity and binding preserved.");
    }, targets.length === 0));
    controls.append(button("Remove component", () => {
      if ((children(node)?.length ?? 0) > 0) {
        message.textContent = "Move or remove children before removing this container.";
        return;
      }
      siblings.splice(index, 1);
      selected = row.parent!.block.id;
      commit("Removed component.");
    }));
  }

  render();
  void loadQueries(revision);
  return { root: () => structuredClone(root) };
}
