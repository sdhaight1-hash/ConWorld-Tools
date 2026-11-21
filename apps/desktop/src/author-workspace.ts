import { invoke } from "@tauri-apps/api/core";
import type { Scene } from "./main";

type World = { id: string; name: string; revision: number };
type Person = { id: string; label: string };
type Cause = {
  day: number; event_id: string; event: string; explanation: string;
  actor: string | null; action: string | null; intent: string | null;
  outcome: string | null; source: string;
  before_trust: number | null; before_tags: string[];
  after_trust: number | null; after_tags: string[];
};
type Relationship = {
  id: string; subject_id: string; target_id: string; kind: string;
  trust: number | null; tags: string[]; starts_at: number | null; cause: Cause | null;
};
type Event = {
  id: string; label: string; day: number | null; citation: string | null;
  participants: { person_id: string; role: string }[];
  actions: { id: string; person_id: string; target_id: string | null; action: string; intent: string; outcome: string; citation: string | null }[];
};
type Definition = { id: string; label: string; category: "relationship" | "tag" | "role";
  description: string; reverse_label: string | null; symmetric: boolean };
type EvidenceStatement = { statement: string; source: string | null; audience: string | null; reader_id: string | null };
type SearchResult = { revision: number; truncated: boolean; rows: { id: string; label: string; kind: string }[] };
type Snapshot = {
  world: World; day: number; view_revision: number;
  people: Person[]; relationships: Relationship[]; events: Event[]; definitions: Definition[];
};
type Option = { id: string; label: string };

function options(select: HTMLSelectElement, values: Option[], preferred = select.value) {
  select.replaceChildren(...values.map(({ id, label }) => new Option(label, id)));
  if (values.some((item) => item.id === preferred)) select.value = preferred;
}

function line(parent: HTMLElement, label: string, text: string) {
  const paragraph = document.createElement("p");
  const title = document.createElement("strong");
  title.textContent = label;
  paragraph.append(title, " " + text);
  parent.append(paragraph);
}

function state(trust: number | null, tags: string[]) {
  return `Trust ${trust ?? "unscored"} · ${tags.join(", ") || "no tags"}`;
}

function eventDetails(panel: HTMLElement, event: Event, name: (id: string) => string) {
  line(panel, "Occurred", event.day === null ? "Date uncertain" : "Day " + event.day);
  line(panel, "Source", event.citation ?? "No citation");
  if (event.participants.length) {
    line(panel, "Roles", event.participants.map((entry) => `${name(entry.person_id)} · ${entry.role}`).join(", "));
  }
  if (event.actions.length) {
    const list = document.createElement("ul");
    for (const action of event.actions) {
      const item = document.createElement("li");
      item.textContent = `${name(action.person_id)} · ${action.action} (${action.outcome})` +
        (action.target_id ? ` → ${name(action.target_id)}` : "") + `. Intent: ${action.intent}.` +
        (action.citation && action.citation !== event.citation ? ` Source: ${action.citation}.` : "");
      list.append(item);
    }
    panel.append(list);
  }
}

function projectScene(snapshot: Snapshot, bondLabel: (bond: Relationship) => string): Scene {
  const edges: Scene["edges"] = [];
  const nodes = [
    ...snapshot.people.map((person) => ({ ...person, kind: "Person" })),
    ...snapshot.relationships.map((bond) => ({ id: bond.id, label: bondLabel(bond), kind: "Relationship" })),
    ...snapshot.events.map((event) => ({ id: event.id, label: event.label, kind: "Event" })),
  ];
  const ids = new Set(nodes.map((node) => node.id));
  const columns = Math.max(1, Math.ceil(Math.sqrt(nodes.length)));
  const addEdge = (source: string, target: string, label: string) => {
    if (ids.has(source) && ids.has(target)) edges.push({ source, target, label });
  };
  for (const bond of snapshot.relationships) {
    addEdge(bond.subject_id, bond.id, "From");
    addEdge(bond.id, bond.target_id, "To");
    if (bond.cause) addEdge(bond.cause.event_id, bond.id, "Caused change");
  }
  for (const event of snapshot.events) {
    for (const { person_id, role } of event.participants) addEdge(person_id, event.id, role);
  }
  const rows = Math.ceil(nodes.length / columns);
  return {
    nodes: nodes.map((node, i) => ({
      ...node,
      x: Math.round(86 + i % columns * Math.max(164, 508 / columns)),
      y: Math.round(65 + Math.floor(i / columns) * Math.max(104, 345 / rows)),
    })),
    edges,
  };
}

export async function showAuthorWorkspace(
  fixture: Scene,
  setScene: (scene: Scene, inspect?: (id: string, panel: HTMLElement) => void) => void,
  selectNode: (id: string) => void,
) {
  const section = document.querySelector<HTMLElement>("#author-workspace")!;
  const worldSelect = document.querySelector<HTMLSelectElement>("#author-world")!;
  const dayInput = document.querySelector<HTMLInputElement>("#author-day")!;
  const revisionInput = document.querySelector<HTMLInputElement>("#author-revision")!;
  const content = document.querySelector<HTMLElement>("#author-world-content")!;
  const info = document.querySelector<HTMLElement>("#author-world-info")!;
  const status = document.querySelector<HTMLElement>("#author-status")!;
  const queryForm = document.querySelector<HTMLFormElement>("#author-query-form")!;
  const pathForm = document.querySelector<HTMLFormElement>("#author-path-form")!;
  const queryResults = document.querySelector<HTMLUListElement>("#author-query-results")!;
  const queryStatus = document.querySelector<HTMLElement>("#author-query-status")!;
  const graph = document.querySelector<HTMLElement>("#workspace")!;
  const history = document.querySelector<HTMLElement>("#history")!;
  const heading = document.querySelector("h1")!;
  const intro = document.querySelector<HTMLElement>(".intro")!;
  const fixtureIntro = intro.textContent;
  const authorLink = document.querySelector<HTMLElement>("#author-link")!;
  const worldForm = document.querySelector<HTMLFormElement>("#author-create-world")!;
  const personForm = document.querySelector<HTMLFormElement>("#author-create-person")!;
  const relationshipForm = document.querySelector<HTMLFormElement>("#author-create-relationship")!;
  const stateForm = document.querySelector<HTMLFormElement>("#author-relationship-state")!;
  const eventForm = document.querySelector<HTMLFormElement>("#author-record-event")!;
  const eventDate = document.querySelector<HTMLElement>("#event-date")!;
  const existingRoles = document.querySelector<HTMLElement>("#existing-event-roles")!;
  const rows = document.querySelector<HTMLElement>("#event-participants")!;
  const effectFields = document.querySelector<HTMLFieldSetElement>("#event-effect")!;
  const addParticipant = document.querySelector<HTMLButtonElement>("#add-participant")!;
  const definitionForm = document.querySelector<HTMLFormElement>("#author-definition")!;
  const definitionDetails = document.querySelector<HTMLFieldSetElement>("#relationship-definition")!;
  const definitionsList = document.querySelector<HTMLUListElement>("#world-definitions")!;
  const evidenceForm = document.querySelector<HTMLFormElement>("#author-record-evidence")!;
  const forms = [personForm, relationshipForm, stateForm, eventForm, definitionForm, evidenceForm];
  const field = <T extends HTMLInputElement | HTMLSelectElement>(form: HTMLFormElement, name: string) =>
    form.elements.namedItem(name) as T;
  const evidenceEvent = field<HTMLSelectElement>(evidenceForm, "event");
  const evidenceAudience = field<HTMLSelectElement>(evidenceForm, "audience");
  const definitionCategory = field<HTMLSelectElement>(definitionForm, "category");
  const symmetric = field<HTMLInputElement>(definitionForm, "symmetric");
  const inverse = field<HTMLInputElement>(definitionForm, "reverse_label");
  const checkbox = field<HTMLInputElement>(eventForm, "add_effect");
  const existing = field<HTMLSelectElement>(eventForm, "existing");
  const stateSelect = field<HTMLSelectElement>(stateForm, "relationship");
  const effectRelation = field<HTMLSelectElement>(eventForm, "relationship");
  const causingAction = field<HTMLSelectElement>(eventForm, "causing_action");
  const value = (form: HTMLFormElement, name: string) => field(form, name).value;
  const score = (form: HTMLFormElement) => value(form, "trust") === "" ? null : Number(value(form, "trust"));
  let snapshot: Snapshot | undefined;
  let requestSequence = 0;
  let querySequence = 0;
  let saving = false;
  let selectedEvent: Event | undefined;
  let people: Option[] = [];
  let personNames = new Map<string, string>();
  const name = (id: string) => personNames.get(id) ?? "Unavailable person";
  section.hidden = false;

  function currentDay() {
    const raw = dayInput.value.trim();
    const day = Number(raw);
    if (!/^-?\d+$/.test(raw) || !Number.isSafeInteger(day)) throw new Error("Enter a whole-number historical day.");
    return day;
  }

  function selectedRevision() {
    const raw = revisionInput.value.trim();
    if (!raw) return null;
    const revision = Number(raw);
    if (!/^\d+$/.test(raw) || !Number.isSafeInteger(revision)) throw new Error("Enter a non-negative, whole-number author revision.");
    return revision;
  }

  function syncButtons() {
    const blocked = !snapshot || revisionInput.value.trim() !== "" || saving;
    const day = snapshot?.day;
    const availability = [
      true,
      people.length >= 2,
      Boolean(day !== undefined && stateSelect.value && snapshot?.relationships.some((bond) => bond.id === stateSelect.value &&
        (bond.starts_at === null || bond.starts_at < day))),
      !selectedEvent || selectedEvent.day !== null,
      true,
      Boolean(evidenceEvent.value),
    ];
    for (const [index, form] of forms.entries()) {
      form.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = blocked || !availability[index];
    }
    for (const control of [worldSelect, dayInput, revisionInput]) control.disabled = saving;
    worldForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = saving;
    queryForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = !snapshot || saving;
    pathForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = !snapshot || !people.length || saving;
  }

  const definitionFor = (category: Definition["category"], label: string) =>
    snapshot?.definitions?.find((item) => item.category === category && item.label.toLowerCase() === label.toLowerCase());
  const bondLabel = (bond: Relationship) => {
    const arrow = definitionFor("relationship", bond.kind)?.symmetric ? "↔" : "→";
    return `${name(bond.subject_id)} ${arrow} ${name(bond.target_id)} · ${bond.kind}`;
  };
  const audienceOptions = () => [
    { id: "author", label: "Author only" },
    { id: "public", label: "Public" },
    ...people.map((person) => ({ id: person.id, label: `Character · ${person.label}` })),
  ];
  const roleOptions = () => [
    ...["Actor", "Witness", "Victim", "Stakeholder", "Sponsor"].map((label) => ({ id: label.toLowerCase(), label })),
    ...(snapshot?.definitions ?? []).filter((item) => item.category === "role")
      .map((item) => ({ id: "definition:" + item.id, label: item.label })),
  ];
  const rowField = (row: Element, key: string) => row.querySelector<HTMLInputElement | HTMLSelectElement>(`[data-field="${key}"]`)!;
  const participantRows = () => Array.from(rows.querySelectorAll<HTMLFieldSetElement>(":scope > fieldset"));

  function syncEffect() {
    const acting = participantRows().flatMap((row, index) => rowField(row, "role").value === "actor"
      ? [{ id: String(index), label: `${name(rowField(row, "person").value)} · ${rowField(row, "action").value.trim() || "Action " + (index + 1)}` }]
      : []);
    options(causingAction, [{ id: "", label: "Select the causing action" }, ...acting]);
    const eventDay = selectedEvent?.day ?? snapshot?.day;
    const relationships = snapshot?.relationships.filter((bond) => eventDay !== undefined &&
      (bond.starts_at === null || bond.starts_at < eventDay)) ?? [];
    options(effectRelation, relationships.map((bond) => ({ id: bond.id, label: bondLabel(bond) })));
    checkbox.disabled = !relationships.length || !acting.length;
    effectFields.disabled = !checkbox.checked || checkbox.disabled;
    effectFields.hidden = effectFields.disabled;
  }

  function addRow(action?: Event["actions"][number]) {
    const row = document.createElement("fieldset");
    row.innerHTML = `
      <legend>Participant</legend>
      <div class="participant-fields">
        <label>Person <select data-field="person" required></select></label>
        <label>Role <select data-field="role"></select></label>
      </div>
      <div class="participant-fields" data-action>
        <label>Action <input data-field="action" required maxlength="200" /></label>
        <label>Intent <input data-field="intent" required maxlength="200" /></label>
        <label>Outcome <select data-field="outcome">
          <option value="attempted">Attempted</option><option value="succeeded">Succeeded</option><option value="failed">Failed</option>
        </select></label>
        <label>Action target <select data-field="target"></select></label>
      </div>
      <button type="button">Remove participant</button>`;
    options(rowField(row, "person") as HTMLSelectElement, people, action?.person_id);
    options(rowField(row, "role") as HTMLSelectElement, roleOptions());
    options(rowField(row, "target") as HTMLSelectElement, [{ id: "", label: "None" }, ...people], action?.target_id ?? "");
    if (action) {
      row.dataset.actionId = action.id;
      rowField(row, "person").disabled = true;
      rowField(row, "role").disabled = true;
      rowField(row, "action").value = action.action;
      rowField(row, "intent").value = action.intent;
      rowField(row, "outcome").value = action.outcome;
      row.querySelector("legend")!.textContent = "Correct saved action";
      row.querySelector("button")!.textContent = "Cancel action correction";
    }
    row.addEventListener("input", () => {
      const isActor = rowField(row, "role").value === "actor";
      row.querySelector<HTMLElement>("[data-action]")!.hidden = !isActor;
      for (const input of row.querySelectorAll<HTMLInputElement | HTMLSelectElement>("[data-action] input, [data-action] select")) {
        input.disabled = !isActor;
      }
      syncEffect();
    });
    row.querySelector("button")!.addEventListener("click", () => {
      // Clear the index-based choice before row indices shift.
      causingAction.value = "";
      row.remove();
      syncEffect();
    });
    rows.append(row);
    syncEffect();
  }

  function editEvent(id: string) {
    selectedEvent = snapshot?.events.find((event) => event.id === id);
    existing.value = selectedEvent?.id ?? "";
    field(eventForm, "event").value = selectedEvent?.label ?? "";
    field(eventForm, "citation").value = selectedEvent?.citation ?? "";
    eventDate.textContent = selectedEvent
      ? (selectedEvent.day === null ? "Event date uncertain" : `Occurred on day ${selectedEvent.day}`)
      : "New events use the selected historical day.";
    checkbox.checked = false;
    field(eventForm, "explanation").value = "";
    field(eventForm, "trust").value = "";
    field(eventForm, "tags").value = "";
    causingAction.value = "";
    existingRoles.replaceChildren();
    rows.replaceChildren();
    if (selectedEvent) {
      eventDetails(existingRoles, selectedEvent, name);
      if (selectedEvent.day !== null && !revisionInput.value.trim()) {
        for (const action of selectedEvent.actions) {
          const button = document.createElement("button");
          button.type = "button";
          button.textContent = "Correct action: " + action.action;
          button.addEventListener("click", () => {
            const row = participantRows().find((item) => item.dataset.actionId === action.id);
            if (row) rowField(row, "action").focus();
            else addRow(action);
          });
          existingRoles.append(button);
        }
      }
    }
    addParticipant.disabled = !people.length || selectedEvent?.day === null;
    syncEffect();
    syncButtons();
  }

  function fillState(bond = snapshot?.relationships.find((record) => record.id === stateSelect.value)) {
    stateSelect.value = bond?.id ?? "";
    field(stateForm, "trust").value = bond?.trust?.toString() ?? "";
    field(stateForm, "tags").value = bond?.tags.join(", ") ?? "";
    syncButtons();
  }

  function evidencePanel(panel: HTMLElement, event: Event, view: Snapshot) {
    const label = document.createElement("label");
    label.textContent = "Accounts visible to";
    const select = document.createElement("select");
    options(select, revisionInput.value.trim() ? audienceOptions().slice(0, 1) : audienceOptions());
    label.append(select);
    const results = document.createElement("div");
    results.setAttribute("aria-live", "polite");
    panel.append(label, results);
    let latest = 0;
    async function refreshAccounts() {
      const sequence = ++latest;
      results.textContent = "Loading accounts…";
      try {
        const accounts = await invoke<EvidenceStatement[]>("workspace_evidence", {
          worldId: view.world.id, eventId: event.id, day: view.day,
          revision: revisionInput.value.trim() ? view.view_revision : null, audience: select.value,
        });
        if (latest !== sequence || !results.isConnected) return;
        results.replaceChildren();
        if (!accounts.length) results.textContent = "No visible accounts.";
        for (const account of accounts) {
          line(results, "Reported", account.statement);
          if (account.source) line(results, "Citation", account.source);
          if (account.audience) line(results, "Audience", account.reader_id ? name(account.reader_id) : account.audience);
        }
      } catch (error) {
        if (latest === sequence && results.isConnected) results.textContent = "Accounts unavailable: " + String(error);
      }
    }
    select.addEventListener("change", () => void refreshAccounts());
    void refreshAccounts();
  }

  function inspector(id: string, panel: HTMLElement) {
    const bond = snapshot?.relationships.find((record) => record.id === id);
    const event = snapshot?.events.find((record) => record.id === id);
    function button(label: string, action: () => void) {
      const control = document.createElement("button");
      control.type = "button";
      control.textContent = label;
      control.addEventListener("click", action);
      panel.append(control);
    }
    if (event) {
      eventDetails(panel, event, name);
      if (snapshot) evidencePanel(panel, event, snapshot);
      if (!revisionInput.value.trim() && event.day !== null) button("Edit this event", () => {
        editEvent(id);
        field(eventForm, "event").focus();
      });
      if (!revisionInput.value.trim()) button("Add sourced account", () => {
        evidenceEvent.value = id;
        field(evidenceForm, "statement").focus();
      });
    }
    if (bond) {
      line(panel, "Current state", state(bond.trust, bond.tags));
      line(panel, "Effective", bond.starts_at === null ? "Start unknown" : "Day " + bond.starts_at);
      const definition = definitionFor("relationship", bond.kind);
      if (definition) line(panel, "Definition", definition.description);
      if (definition?.reverse_label) line(panel, "Inverse label", definition.reverse_label);
      if (definition?.symmetric) line(panel, "Trust direction", `${name(bond.subject_id)} → ${name(bond.target_id)}`);
      for (const tag of bond.tags) {
        const definition = definitionFor("tag", tag);
        if (definition) line(panel, tag, definition.description);
      }
      if (bond.cause) {
        const cause = bond.cause;
        line(panel, "Why it changed", cause.explanation);
        line(panel, "Before", state(cause.before_trust, cause.before_tags));
        line(panel, "After", state(cause.after_trust, cause.after_tags));
        line(panel, "Attribution", [cause.event, cause.actor, cause.action, cause.outcome].filter(Boolean).join(" · "));
        if (cause.intent) line(panel, "Intent", cause.intent);
        line(panel, "Source", cause.source);
        button("Inspect source event", () => selectNode(cause.event_id));
      }
      if (!revisionInput.value.trim()) button("Change this relationship", () => {
        fillState(bond);
        dayInput.focus();
      });
    }
  }

  function showFixture() {
    history.before(graph);
    heading.textContent = "Northbridge";
    intro.textContent = fixtureIntro;
    graph.setAttribute("aria-label", "Northbridge scene");
    graph.hidden = false;
    authorLink.hidden = false;
    history.hidden = false;
    setScene(fixture);
  }

  async function refresh() {
    const sequence = ++requestSequence;
    ++querySequence;
    queryResults.replaceChildren();
    queryStatus.textContent = "Search the selected world, historical day, and author revision.";
    const worldId = worldSelect.value;
    const previous = snapshot;
    snapshot = undefined;
    syncButtons();
    content.hidden = !worldId;
    if (!worldId) {
      showFixture();
      status.textContent = "Select or create a world to begin.";
      return;
    }
    const worldChanged = previous?.world.id !== worldId;
    if (worldChanged) graph.hidden = true;
    const day = currentDay();
    const revision = selectedRevision();
    let next: Snapshot;
    try {
      next = await invoke<Snapshot>("workspace_snapshot", { worldId, day, revision });
    } catch (error) {
      if (sequence === requestSequence) throw error;
      return;
    }
    if (sequence !== requestSequence) return;
    snapshot = next;
    people = next.people;
    personNames = new Map(people.map((person) => [person.id, person.label]));
    if (worldChanged) {
      evidenceForm.reset();
      definitionForm.reset();
      syncDefinition();
      rows.replaceChildren();
      stateSelect.value = "";
      existing.value = "";
    }
    info.textContent = `Author revision ${next.view_revision} / ${next.world.revision} · historical day ${next.day}`;
    const subject = field<HTMLSelectElement>(relationshipForm, "subject");
    const target = field<HTMLSelectElement>(relationshipForm, "target");
    options(subject, people);
    options(target, people);
    options(field<HTMLSelectElement>(pathForm, "start"), people);
    if (people.length > 1 && subject.value === target.value) {
      target.value = people.find((person) => person.id !== subject.value)!.id;
    }
    const previousBond = previous?.relationships.find((bond) => bond.id === stateSelect.value);
    options(stateSelect, next.relationships.map((bond) => ({ id: bond.id, label: bondLabel(bond) })));
    const nextBond = next.relationships.find((bond) => bond.id === stateSelect.value);
    if (worldChanged || previousBond?.id !== nextBond?.id || previousBond?.starts_at !== nextBond?.starts_at ||
      previous?.view_revision !== next.view_revision) fillState(nextBond);
    const eventOptions = next.events.map((event) => ({ id: event.id, label: event.label }));
    const previousEvent = existing.value;
    options(existing, [{ id: "", label: "New event" }, ...eventOptions]);
    options(evidenceEvent, eventOptions);
    options(evidenceAudience, audienceOptions());
    const definitions = next.definitions ?? [];
    document.querySelector("#relationship-kinds")!.replaceChildren(...definitions
      .filter((item) => item.category === "relationship").map((item) => new Option(item.label)));
    definitionsList.replaceChildren(...definitions.map((item) => {
      const entry = document.createElement("li");
      const control = document.createElement("button");
      control.type = "button";
      control.textContent = item.label;
      control.disabled = revision !== null;
      control.addEventListener("click", () => {
        definitionCategory.value = item.category;
        field(definitionForm, "label").value = item.label;
        field(definitionForm, "description").value = item.description;
        inverse.value = item.reverse_label ?? "";
        symmetric.checked = item.symmetric;
        syncDefinition();
        field(definitionForm, "description").focus();
      });
      entry.append(control, " " + item.description);
      return entry;
    }));
    const eventRevisionChanged = existing.value && previous?.view_revision !== next.view_revision;
    if (worldChanged || previousEvent !== existing.value || eventRevisionChanged || revision !== null) {
      editEvent(existing.value);
    }
    for (const row of participantRows()) {
      options(rowField(row, "person") as HTMLSelectElement, people);
      options(rowField(row, "role") as HTMLSelectElement, roleOptions());
      options(rowField(row, "target") as HTMLSelectElement, [{ id: "", label: "None" }, ...people]);
    }
    addParticipant.disabled = !people.length || selectedEvent?.day === null;
    syncEffect();
    syncButtons();
    if (info.nextElementSibling !== graph) info.after(graph);
    graph.hidden = false;
    heading.textContent = next.world.name;
    intro.textContent = "Explore saved people, relationships, and events.";
    graph.setAttribute("aria-label", "Saved world graph");
    history.hidden = true;
    authorLink.hidden = true;
    setScene(projectScene(next, bondLabel), inspector);
    status.textContent = revision !== null
      ? "Read-only author revision. Clear the author revision field to resume editing."
      : "World loaded. Edits are saved locally.";
  }

  function runQuery(message: string, execute: () => Promise<SearchResult>) {
    if (!snapshot || saving) return;
    const sequence = ++querySequence;
    queryResults.replaceChildren();
    queryStatus.textContent = message;
    void (async () => {
      try {
        const result = await execute();
        if (sequence !== querySequence || !snapshot) return;
        const graphIds = new Set([
          ...snapshot.people.map((item) => item.id),
          ...snapshot.relationships.map((item) => item.id),
          ...snapshot.events.map((item) => item.id),
        ]);
        for (const row of result.rows) {
          const item = document.createElement("li");
          const description = row.label + " · " + row.kind.replaceAll("_", " ") + " · " + row.id;
          if (graphIds.has(row.id)) {
            const button = document.createElement("button");
            button.type = "button";
            button.textContent = description;
            button.addEventListener("click", () => selectNode(row.id));
            item.append(button);
          } else {
            const text = document.createElement("span");
            text.textContent = description;
            item.append(text);
          }
          const references = document.createElement("button");
          references.type = "button";
          references.className = "query-references";
          references.textContent = "Find references";
          references.setAttribute("aria-label", "Find references to " + row.label + " " + row.id);
          references.addEventListener("click", () => runSearch("backlinks", row.id));
          item.append(references);
          queryResults.append(item);
        }
        queryStatus.textContent = result.rows.length + " match(es) at author revision " + result.revision +
          (result.truncated ? " · more results exist; narrow the search." : "");
      } catch (error) {
        if (sequence === querySequence) queryStatus.textContent = "Search failed: " + String(error);
      }
    })();
  }

  function runSearch(mode: string, term: string) {
    if (!snapshot || saving) return;
    const worldId = snapshot.world.id;
    const day = snapshot.day;
    const revision = selectedRevision();
    runQuery(mode === "backlinks" ? "Finding incoming references…" : "Searching historical records…",
      () => invoke<SearchResult>("workspace_search", { worldId, day, revision, mode, term, limit: 100 }));
  }

  queryForm.addEventListener("submit", (event) => {
    event.preventDefault();
    runSearch(field<HTMLSelectElement>(queryForm, "mode").value,
      field<HTMLInputElement>(queryForm, "term").value.trim());
  });

  pathForm.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot || saving) return;
    const request = {
      world_id: snapshot.world.id,
      day: snapshot.day,
      revision: selectedRevision(),
      start_id: value(pathForm, "start"),
      kind: value(pathForm, "kind").trim(),
      direction: value(pathForm, "direction"),
      max_depth: Number(value(pathForm, "max_depth")),
      max_visited: Number(value(pathForm, "max_visited")),
      limit: 100,
    };
    runQuery("Exploring historical relationship paths…",
      () => invoke<SearchResult>("workspace_query_path", { request }));
  });

  async function refreshWorlds(preferred = worldSelect.value) {
    const worlds = await invoke<World[]>("workspace_worlds");
    options(worldSelect, [
      ...worlds.map((world) => ({ id: world.id, label: world.name })),
      { id: "", label: "Northbridge demo" },
    ], preferred || worlds[0]?.id || "");
    await refresh();
  }

  function bind(form: HTMLFormElement, action: (context: { world_id: string; expected_revision: number }) => Promise<void | string>) {
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      if (saving) return;
      void (async () => {
        try {
          if (form !== worldForm) {
            currentDay();
            if (selectedRevision() !== null) throw new Error("Archived author revisions are read-only.");
            if (!snapshot || snapshot.world.id !== worldSelect.value) throw new Error("Wait for the world to load.");
          }
          saving = true;
          syncButtons();
          const savedNode = await action({ world_id: worldSelect.value, expected_revision: snapshot?.world.revision ?? 0 });
          if (form !== worldForm) await refresh();
          if (form === worldForm || form === personForm) field(form, "name").value = "";
          if (form === evidenceForm) {
            field(form, "statement").value = "";
            field(form, "citation").value = "";
            selectNode(evidenceEvent.value);
          }
          if (form === eventForm) {
            editEvent(savedNode || existing.value);
            if (savedNode) selectNode(savedNode);
          }
          status.textContent = "Saved to local world history.";
        } catch (error) {
          status.textContent = "Unable to save: " + String(error);
        } finally {
          saving = false;
          syncButtons();
        }
      })();
    });
  }

  bind(worldForm, async () => {
    const world = await invoke<World>("workspace_create_world", { name: value(worldForm, "name") });
    revisionInput.value = "";
    await refreshWorlds(world.id);
  });
  bind(personForm, async ({ world_id }) => {
    await invoke("workspace_create_person", { worldId: world_id, name: value(personForm, "name") });
  });
  bind(relationshipForm, async (context) => {
    await invoke("workspace_create_relationship", { request: {
      ...context, subject_id: subjectValue(), target_id: value(relationshipForm, "target"),
      kind: value(relationshipForm, "kind"), trust: score(relationshipForm),
      tags: value(relationshipForm, "tags"), day: currentDay(),
    } });
  });
  bind(stateForm, async (context) => {
    await invoke("workspace_record_relationship_state", { request: {
      ...context, relationship_id: stateSelect.value, trust: score(stateForm),
      tags: value(stateForm, "tags"), day: currentDay(),
    } });
  });
  bind(eventForm, async (context) => {
    const participants = participantRows().map((row) => ({
      person_id: rowField(row, "person").value,
      role: rowField(row, "role").value,
      action: rowField(row, "role").value === "actor" ? {
        id: row.dataset.actionId ?? null,
        action: rowField(row, "action").value,
        intent: rowField(row, "intent").value,
        outcome: rowField(row, "outcome").value,
        target_id: rowField(row, "target").value || null,
      } : null,
    }));
    return invoke<string>("workspace_record_event", { request: {
      ...context, event_id: existing.value || null, day: currentDay(),
      event_label: value(eventForm, "event"), source_citation: value(eventForm, "citation"),
      participants, effect: effectFields.disabled ? null : {
        relationship_id: effectRelation.value,
        actor_row: Number(causingAction.value),
        explanation: value(eventForm, "explanation"),
        trust: score(eventForm), tags: value(eventForm, "tags"),
      },
    } });
  });
  function syncDefinition() {
    const directional = definitionCategory.value === "relationship";
    definitionDetails.hidden = !directional;
    definitionDetails.disabled = !directional;
    inverse.disabled = symmetric.checked;
  }
  symmetric.addEventListener("change", syncDefinition);
  definitionCategory.addEventListener("change", syncDefinition);
  bind(definitionForm, async (context) => {
    await invoke("workspace_save_definition", { request: {
      ...context, category: definitionCategory.value, label: value(definitionForm, "label"),
      description: value(definitionForm, "description"),
      reverse_label: definitionDetails.disabled || inverse.disabled ? null : inverse.value.trim() || null,
      symmetric: !definitionDetails.disabled && symmetric.checked,
    } });
  });
  bind(evidenceForm, async (context) => {
    await invoke("workspace_record_evidence", { request: {
      ...context, event_id: evidenceEvent.value, day: currentDay(),
      statement: value(evidenceForm, "statement"), citation: value(evidenceForm, "citation"),
      audience: evidenceAudience.value,
    } });
  });
  const subjectValue = () => value(relationshipForm, "subject");
  field(relationshipForm, "subject").addEventListener("change", () => {
    const target = field(relationshipForm, "target");
    if (target.value === subjectValue()) target.value = people.find((person) => person.id !== subjectValue())?.id ?? "";
  });
  existing.addEventListener("change", () => editEvent(existing.value));
  stateSelect.addEventListener("change", () => fillState());
  checkbox.addEventListener("change", syncEffect);
  addParticipant.addEventListener("click", () => addRow());
  function reload() {
    void refresh().catch((error) => {
      status.textContent = String(error);
    });
  }
  worldSelect.addEventListener("change", () => {
    revisionInput.value = "";
    reload();
  });
  dayInput.addEventListener("change", reload);
  revisionInput.addEventListener("change", reload);
  try {
    await refreshWorlds();
  } catch (error) {
    status.textContent = "Author workspace unavailable: " + String(error);
  }
}
