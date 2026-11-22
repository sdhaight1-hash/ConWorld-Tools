import { invoke } from "@tauri-apps/api/core";
import type { Block, Binding } from "./entry-editor";
import type { SavedQuery, ParameterValue } from "./author-workspace";
import { templates, instantiateTemplate, historicalComponent } from "./canvas-templates";

// This is a draft-only editor. The Rust compiler/store validates the entire tree
// before save and owns publication; no new persistence or authority lives here.
function children(block: Block): Block[] | undefined {
  return block.content.Stack ?? block.content.Panel ?? block.content.Grid?.children ?? block.content.Template?.children ?? block.content.CustomTemplate?.children;
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
  entryId: string, day: number, revision: number, editable: boolean, changed: () => void) {
  let root = structuredClone(original);
  let selected = root.id;
  let customTemplates: { id: string; name: string; revision: number }[] = [];
  const context = { world_id: worldId, day, revision };
  let dragged: string | undefined;
  let last = structuredClone(root);
  const undo: Block[] = [];
  const redo: Block[] = [];
  let undoButton: HTMLButtonElement | undefined;
  let redoButton: HTMLButtonElement | undefined;
  function recordChange() {
    undo.push(last);
    if (undo.length > 32) undo.shift();
    redo.length = 0;
    last = structuredClone(root);
    if (undoButton) undoButton.disabled = false;
    if (redoButton) redoButton.disabled = true;
    changed();
  }
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
    recordChange();
    render();
  }

  function contains(ancestor: Block, id: string): boolean {
    return ancestor.id === id || (children(ancestor)?.some(child => contains(child, id)) ?? false);
  }
  function height(block: Block): number {
    return Math.max(0, ...(children(block) ?? []).map(child => 1 + height(child)));
  }
  function size(block: Block): number {
    return 1 + (children(block) ?? []).reduce((total, child) => total + size(child), 0);
  }
  function move(source: string, destination: string) {
    if (!editable) return;
    const all = rows();
    const item = all.find(row => row.block.id === source);
    const target = all.find(row => row.block.id === destination);
    if (!item?.parent || !target || !children(target.block) || target === item.parent ||
        contains(item.block, target.block.id) || target.depth + 1 + height(item.block) > 16) return;
    const siblings = children(item.parent.block)!;
    item.block.audience = restrict(item.block.audience, item.parent.effective);
    siblings.splice(siblings.indexOf(item.block), 1);
    children(target.block)!.push(item.block);
    selected = item.block.id;
    commit("Moved component; identity and binding preserved.");
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
    if (editable) {
      const history = document.createElement("div");
      undoButton = button("Undo Canvas edit", () => {
        const prior = undo.pop();
        if (!prior) return;
        redo.push(structuredClone(root));
        root = prior; last = structuredClone(root); changed(); render();
      }, !undo.length);
      redoButton = button("Redo Canvas edit", () => {
        const next = redo.pop();
        if (!next) return;
        undo.push(structuredClone(root));
        root = next; last = structuredClone(root); changed(); render();
      }, !redo.length);
      history.append(undoButton, redoButton);
      controls.append(history);
    }
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
      if (editable) {
        line.draggable = !!item.parent;
        line.ondragstart = event => { dragged = item.block.id; event.dataTransfer?.setData("text/plain", item.block.id); };
        line.ondragend = () => { dragged = undefined; };
        if (children(item.block)) {
          line.ondragover = event => { if (dragged) event.preventDefault(); };
          line.ondrop = event => { event.preventDefault(); if (dragged) move(dragged, item.block.id); dragged = undefined; };
        }
      }
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
      text.addEventListener("input", () => { node.content.Text = text.value; recordChange(); });
      field("Component text", text);
    }
    if (node.content.ObjectLink) {
      const link = node.content.ObjectLink;
      const target = document.createElement("input");
      target.value = link.target.id;
      target.required = true;
      target.readOnly = !editable;
      target.addEventListener("input", () => { link.target.id = target.value.trim(); recordChange(); });
      field("Component link target", target);
      const caption = document.createElement("input");
      caption.value = link.label;
      caption.required = true;
      caption.maxLength = 800;
      caption.readOnly = !editable;
      caption.addEventListener("input", () => { link.label = caption.value; recordChange(); });
      field("Component link caption", caption);
    }
    if (node.content.Grid) {
      const columns = document.createElement("select");
      for (const value of [1, 2, 3, 4]) columns.add(new Option(String(value), String(value)));
      columns.value = String(node.content.Grid.columns);
      columns.disabled = !editable;
      columns.addEventListener("change", () => { node.content.Grid!.columns = Number(columns.value); recordChange(); });
      field("Grid columns", columns);
    }
    if (node.content.CustomTemplate) {
      const pin = document.createElement("p");
      pin.textContent = "Custom template revision " + node.content.CustomTemplate.definition_revision + " · " + node.content.CustomTemplate.template_id;
      controls.append(pin);
    }
    if (node.content.Template) {
      const pin = document.createElement("p");
      pin.textContent = "Template: " + node.content.Template.key + " · version " + node.content.Template.version;
      controls.append(pin);
    }
    const liveFocus = node.content.NetworkTable ?? node.content.ImpactTable ?? node.content.PublishedReferences;
    if (liveFocus) {
      const target = document.createElement("input");
      target.value = liveFocus.id;
      target.required = true;
      target.readOnly = !editable;
      target.addEventListener("input", () => { liveFocus.id = target.value.trim(); recordChange(); });
      field("Reference subject identity", target);
      const description = document.createElement("p");
      description.textContent = node.content.NetworkTable ? "Private sourced historical facts." : node.content.ImpactTable ? "Confirmed persisted event impact. Author only." :
        "Only current released backlinks, never private registry data.";
      controls.append(description);
    }
    if (node.content.FamilyTable) {
      const target = document.createElement("input");
      target.value = node.content.FamilyTable.id;
      target.required = true;
      target.readOnly = !editable;
      target.addEventListener("input", () => { node.content.FamilyTable!.id = target.value.trim(); recordChange(); });
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
          for (const kind of ["person", "organization", "household", "family_house", "faction", "conflict", "definition", "relationship", "event_involvement", "actor_action",
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
          recordChange();
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
    audience.disabled = !editable || node === root || !!node.content.SavedQuery || !!node.content.Query || !!node.content.FamilyTable || !!node.content.NetworkTable || !!node.content.ImpactTable;
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
      controls.append(button("Load custom templates", async () => {
        try {
          customTemplates = await invoke<typeof customTemplates>("workspace_custom_templates", { request: context });
          if (host.isConnected) render();
        } catch (error) { message.textContent = String(error); }
      }));
      if (customTemplates.length) {
        const choice = document.createElement("select");
        for (const item of customTemplates) choice.add(new Option(item.name + " · revision " + item.revision, item.id));
        choice.setAttribute("aria-label", "Custom template");
        field("Custom template", choice);
        controls.append(button("Insert custom template", async () => {
          const item = customTemplates.find(item => item.id === choice.value);
          if (!item) return;
          const before = last;
          try {
            const instance = await invoke<Block>("workspace_instantiate_custom_template", { request: context,
              templateId: item.id, definitionRevision: item.revision, entryId });
            if (!host.isConnected || last !== before) return;
            if (rows().length + size(instance) > 512 || row.depth + 1 + height(instance) > 16) {
              message.textContent = "Template exceeds the layout budget."; return;
            }
            slot.push(instance); selected = instance.id; commit("Inserted pinned custom template.");
          } catch (error) { message.textContent = String(error); }
        }));
      }
      const history = document.createElement("select");
      for (const [value, label] of [["person_events", "Person event history"], ["family", "Accepted family links"], ["network", "Sourced historical network"], ["impact", "Confirmed event impact"], ["published_refs", "Published references"], ["actors", "Event actors"],
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
    const targets = all.filter(target => children(target.block) && target !== row.parent &&
      !contains(node, target.block.id) && target.depth + 1 + height(node) <= 16);
    const destination = document.createElement("select");
    for (const target of targets) destination.add(new Option(kind(target.block) + " · " + target.block.id.slice(0, 8), target.block.id));
    field("Move into container", destination);
    controls.append(button("Move component", () => {
      const target = targets.find(target => target.block.id === destination.value);
      if (!target) return;
      move(node.id, target.block.id);
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
