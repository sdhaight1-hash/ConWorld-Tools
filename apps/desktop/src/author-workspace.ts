import { openFamilyHistory } from "./family-history";
import { openEntryEditor } from "./entry-editor";
import { publicReader, type Backlinks } from "./public-reader";
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
type SearchResult = { revision: number; truncated: boolean; next_cursor?: string | null; rows: {
  id: string; label: string; kind: string;
  statement?: string | null; source_citation?: string | null;
}[] };
type Predicate = { op: "contains" | "exact" | "exact_alias" | "full_text" | "claim_full_text"; text: string } |
  { op: "object_kind"; kind: string } |
  { op: "has_source"; source_id: string } |
  { op: "and" | "or"; filters: Predicate[] } |
  { op: "not"; filter: Predicate };
export type ParameterValue = { type: "text" | "object_kind"; value: string } |
  { type: "source"; world_id: string; source_id: string };
type QueryParameter = { name: string; default: ParameterValue };
export type SavedQuery = { id: string; name: string; mode: "contains" | "exact" | "alias" | "fulltext" | "claimtext" | "composite";
  term: string; predicate?: Predicate | null; limit: number; recorded_at: number; parameters?: QueryParameter[] };
type MentionDiscovery = { revision: number; truncated: boolean; rows: {
  source_id: string; source_branch: string; source_state_revision: number; source_revision: number;
  source_label: string; statement: string; target_id: string; target_label: string; matched_label?: string;
  start_byte: number; end_byte: number; collisions: number; disposition: "approved" | "dismissed" | null;
}[] };
type PersonAlias = {
  id: string; person_id: string; label: string; starts_at: number | null; ends_before: number | null;
  active: boolean; recorded_at: number;
};
type Snapshot = {
  world: World; day: number; view_revision: number;
  people: Person[]; relationships: Relationship[]; events: Event[]; definitions: Definition[];
  aliases?: PersonAlias[];
  scene_limited?: boolean;
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
  const refreshView = document.querySelector<HTMLButtonElement>("#author-refresh-view")!;
  const editorForms = document.querySelector<HTMLElement>(".editor-forms")!;
  const discoveryPanel = document.querySelector<HTMLElement>("#author-mention-discovery")!;
  const status = document.querySelector<HTMLElement>("#author-status")!;
  const queryForm = document.querySelector<HTMLFormElement>("#author-query-form")!;
  const publicSearch = document.querySelector<HTMLFormElement>("#public-page-search-form")!;
  const publicStatus = document.querySelector<HTMLElement>("#public-page-search-status")!;
  const publicResults = document.querySelector<HTMLUListElement>("#public-page-search-results")!;
  const publicPreview = document.querySelector<HTMLElement>("#public-page-preview")!;
  const publicCancel = document.querySelector<HTMLButtonElement>("#public-page-search-cancel")!;
  const saveQueryForm = document.querySelector<HTMLFormElement>("#author-save-query-form")!;
  const runSavedQuery = document.querySelector<HTMLButtonElement>("#author-run-saved-query")!;
  const saveComposite = document.querySelector<HTMLButtonElement>("#author-save-composite-query")!;
  const savedSelect = saveQueryForm.elements.namedItem("query_id") as HTMLSelectElement;
  const parameterFields = document.querySelector<HTMLElement>("#author-saved-parameters")!;
  const parameterInputs = new Map<string, HTMLInputElement | HTMLSelectElement>();
  const focusHint = document.querySelector<HTMLElement>("#author-search-focus")!;
  const pathForm = document.querySelector<HTMLFormElement>("#author-path-form")!;
  const participantEventsForm = document.querySelector<HTMLFormElement>("#author-participant-events-form")!;
  const compositeForm = document.querySelector<HTMLFormElement>("#author-composite-form")!;
  const queryResults = document.querySelector<HTMLUListElement>("#author-query-results")!;
  const nextSearchPage = document.querySelector<HTMLButtonElement>("#author-next-search-page")!;
  const cancelSearch = document.querySelector<HTMLButtonElement>("#author-cancel-query")!;
  const discoverButton = document.querySelector<HTMLButtonElement>("#author-discover-mentions")!;
  const cancelDiscovery = document.querySelector<HTMLButtonElement>("#author-cancel-discovery")!;
  const discoveryStatus = document.querySelector<HTMLElement>("#author-discovery-status")!;
  const discoveryResults = document.querySelector<HTMLUListElement>("#author-discovery-results")!;
  const queryStatus = document.querySelector<HTMLElement>("#author-query-status")!;
  const graph = document.querySelector<HTMLElement>("#workspace")!;
  const history = document.querySelector<HTMLElement>("#history")!;
  const heading = document.querySelector("h1")!;
  const intro = document.querySelector<HTMLElement>(".intro")!;
  const fixtureIntro = intro.textContent;
  const authorLink = document.querySelector<HTMLElement>("#author-link")!;
  const worldForm = document.querySelector<HTMLFormElement>("#author-create-world")!;
  const personForm = document.querySelector<HTMLFormElement>("#author-create-person")!;
  const aliasForm = document.querySelector<HTMLFormElement>("#author-person-alias")!;
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
  const forms = [personForm, relationshipForm, stateForm, eventForm, definitionForm, evidenceForm, aliasForm];
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
  let publicSequence = 0;
  let publicPreviewSequence = 0;
  let activePublicSearch: string | undefined;
  let activeQuery: string | undefined;
  let pageCursor: string | undefined;
  let pageNumber = 1;
  let discoverySequence = 0;
  let activeDiscovery: string | undefined;
  let discoveryFocus: Option | undefined;
  let saving = false;
  let selectedEvent: Event | undefined;
  let people: Option[] = [];
  let focusedPerson: Option | undefined;
  let savedQueries: SavedQuery[] = [];
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
    const limited = Boolean(snapshot?.scene_limited);
    editorForms.hidden = limited;
    discoveryPanel.hidden = !snapshot;
    pathForm.hidden = limited && !focusedPerson;
    participantEventsForm.hidden = limited && !focusedPerson;
    focusHint.hidden = !limited;
    focusHint.textContent = focusedPerson
      ? "Exploring " + focusedPerson.label + " · " + focusedPerson.id
      : "Choose a person from search results to explore their connections and recorded events.";
    refreshView.disabled = saving || !worldSelect.value;
    nextSearchPage.disabled = !snapshot || saving || !pageCursor;
    cancelSearch.hidden = !activeQuery;
    cancelSearch.disabled = !activeQuery;
    const day = snapshot?.day;
    const availability = [
      true,
      people.length >= 2,
      Boolean(day !== undefined && stateSelect.value && snapshot?.relationships.some((bond) => bond.id === stateSelect.value &&
        (bond.starts_at === null || bond.starts_at < day))),
      !selectedEvent || selectedEvent.day !== null,
      true,
      Boolean(evidenceEvent.value),
      people.length > 0,
    ];
    for (const [index, form] of forms.entries()) {
      form.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = blocked || limited || !availability[index];
    }
    for (const control of [worldSelect, dayInput, revisionInput]) control.disabled = saving;
    worldForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = saving;
    queryForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = !snapshot || saving;
    compositeForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = !snapshot || saving;
    discoverButton.disabled = !snapshot || saving;
    cancelDiscovery.hidden = !activeDiscovery;
    cancelDiscovery.disabled = !activeDiscovery;
    saveQueryForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = blocked;
    saveComposite.disabled = blocked;
    runSavedQuery.disabled = !snapshot || !savedSelect.value || saving;
    pathForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled =
      !snapshot || !(limited ? focusedPerson : people.length) || saving;
    participantEventsForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled =
      !snapshot || !(limited ? focusedPerson : people.length) || saving;
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
          revision: view.view_revision, audience: select.value,
        });
        if (latest !== sequence || snapshot !== view || !results.isConnected) return;
        results.replaceChildren();
        if (!accounts.length) results.textContent = "No visible accounts.";
        for (const account of accounts) {
          line(results, "Reported", account.statement);
          if (account.source) line(results, "Citation", account.source);
          if (account.audience) line(results, "Audience", account.reader_id ? name(account.reader_id) : account.audience);
        }
      } catch (error) {
        if (latest === sequence && snapshot === view && results.isConnected) results.textContent = "Accounts unavailable: " + String(error);
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
    const current = snapshot;
    if (current && people.some(person => person.id === id)) {
      button("Family history", () => {
        panel.replaceChildren();
        void openFamilyHistory(panel, { world_id: current.world.id, person_id: id, day: current.day, revision: current.view_revision },
          people, !revisionInput.value.trim(), async () => { await refresh(); selectNode(id); });
      });
      button("Open character page", () => {
        panel.replaceChildren();
        void openEntryEditor(panel, current.world.id, id, current.day, current.view_revision,
          !revisionInput.value.trim(), async () => { await refresh(); selectNode(id); }, people,
          target => {
            panel.replaceChildren();
            void openEntryEditor(panel, current.world.id, target, current.day, current.view_revision,
              !revisionInput.value.trim(), async () => { await refresh(); selectNode(target); }, people);
          });
      });
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

  function cancelActiveQuery() {
    const requestId = activeQuery;
    activeQuery = undefined;
    cancelSearch.hidden = true;
    cancelSearch.disabled = true;
    if (requestId) {
      void invoke("workspace_cancel_query", { requestId }).catch(() => {
        // The generation guard still prevents stale replies from being displayed.
      });
    }
  }

  function clearPublicSearch() {
    ++publicSequence;
    ++publicPreviewSequence;
    const requestId = activePublicSearch;
    activePublicSearch = undefined;
    publicCancel.hidden = true;
    publicResults.replaceChildren();
    publicPreview.replaceChildren();
    publicStatus.textContent = "";
    if (requestId) void invoke("workspace_cancel_query", { requestId }).catch(() => {});
  }

  publicSearch.addEventListener("input", clearPublicSearch);
  publicCancel.addEventListener("click", () => {
    clearPublicSearch();
    publicStatus.textContent = "Public search cancelled.";
  });
  publicSearch.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (!snapshot || saving) return;
    clearPublicSearch();
    const text = field<HTMLInputElement>(publicSearch, "text").value.trim();
    if (!text) return;
    const sequence = publicSequence;
    const requestId = crypto.randomUUID();
    const { world: { id: worldId }, day } = snapshot;
    activePublicSearch = requestId;
    publicCancel.hidden = false;
    publicStatus.textContent = "Searching current published pages…";
    type Hit = { entry: { world: string; id: string }; element: string; text: string };
    try {
      const found = await invoke<{ matches: Hit[]; truncated: boolean }>(
        "workspace_search_public_pages", { worldId, day, text, requestId });
      if (sequence !== publicSequence) return;
      publicStatus.textContent = found.truncated ? "Showing first 50 matching blocks." :
        found.matches.length + " matching public blocks.";
      for (const hit of found.matches) {
        const item = document.createElement("li");
        const passage = document.createElement("p");
        passage.textContent = hit.text;
        const open = document.createElement("button");
        open.type = "button";
        open.textContent = "Read published page";
        open.addEventListener("click", () => {
          const preview = ++publicPreviewSequence;
          void publicReader(publicPreview, worldId, day,
            () => sequence === publicSequence && preview === publicPreviewSequence,
            () => { clearPublicSearch(); publicStatus.textContent = "Publication changed. Search again."; }).open(hit.entry.id);
        });
        item.append(passage, open);
        publicResults.append(item);
      }
    } catch (error) {
      if (sequence === publicSequence) publicStatus.textContent = "Public search failed: " + String(error);
    } finally {
      if (sequence === publicSequence) {
        activePublicSearch = undefined;
        publicCancel.hidden = true;
      }
    }
  });

  async function refresh() {
    clearPublicSearch();
    const sequence = ++requestSequence;
    ++querySequence;
    cancelActiveQuery();
    pageCursor = undefined;
    pageNumber = 1;
    nextSearchPage.hidden = true;
    ++discoverySequence;
    cancelActiveDiscovery();
    discoveryFocus = undefined;
    discoveryResults.replaceChildren();
    discoveryStatus.textContent = "Choose Find possible mentions to scan this historical view.";
    queryResults.replaceChildren();
    queryStatus.textContent = "Search the selected world, historical day, and author revision.";
    const worldId = worldSelect.value;
    const previous = snapshot;
    snapshot = undefined;
    parameterFields.hidden = true;
    parameterFields.replaceChildren();
    parameterInputs.clear();
    focusedPerson = undefined;
    syncButtons();
    content.hidden = !worldId;
    if (!worldId) {
      showFixture();
      status.textContent = "Select or create a world to begin.";
      return;
    }
    const worldChanged = previous?.world.id !== worldId;
    graph.hidden = true;
    info.textContent = "Loading historical view…";
    const day = currentDay();
    const revision = selectedRevision();
    let next: Snapshot;
    let saved: SavedQuery[];
    try {
      next = await invoke<Snapshot>("workspace_snapshot", { worldId, day, revision });
      saved = await invoke<SavedQuery[]>("workspace_saved_queries", { worldId, revision: next.view_revision });
    } catch (error) {
      if (sequence === requestSequence) throw error;
      return;
    }
    if (sequence !== requestSequence) return;
    snapshot = next;
    savedQueries = saved;
    options(savedSelect, [{ id: "", label: "New saved search" },
      ...savedQueries.map((query) => ({ id: query.id, label: query.name + " · " + query.id.slice(0, 8) }))],
      worldChanged ? "" : savedSelect.value);
    renderSavedParameters();
    people = next.people;
    personNames = new Map(people.map((person) => [person.id, person.label]));
    if (worldChanged) {
      field<HTMLInputElement>(saveQueryForm, "name").value = "";
      evidenceForm.reset();
      definitionForm.reset();
      syncDefinition();
      rows.replaceChildren();
      stateSelect.value = "";
      existing.value = "";
      aliasForm.reset();
      field<HTMLSelectElement>(aliasForm, "alias_id").value = "";
    }
    info.textContent = `Author revision ${next.view_revision} / ${next.world.revision} · historical day ${next.day}` +
      (next.scene_limited ? " · Search-only mode: the complete scene exceeds its budget. Indexed search remains available." : "");
    const subject = field<HTMLSelectElement>(relationshipForm, "subject");
    const target = field<HTMLSelectElement>(relationshipForm, "target");
    options(subject, people);
    options(target, people);
    options(field<HTMLSelectElement>(pathForm, "start"), people);
    options(field<HTMLSelectElement>(aliasForm, "person_id"), people);
    options(field<HTMLSelectElement>(aliasForm, "alias_id"), [
      { id: "", label: "New alias" },
      ...(next.aliases ?? []).map((alias) => ({
        id: alias.id,
        label: alias.label + " · " + (personNames.get(alias.person_id) ?? alias.person_id) +
          (alias.active ? "" : " (disabled)"),
      })),
    ], worldChanged ? "" : value(aliasForm, "alias_id"));
    options(field<HTMLSelectElement>(participantEventsForm, "participant"), people);
    options(field<HTMLSelectElement>(participantEventsForm, "role"),
      [{ id: "any", label: "Any recorded role" }, ...roleOptions()]);
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
    graph.hidden = Boolean(next.scene_limited);
    heading.textContent = next.world.name;
    intro.textContent = next.scene_limited
      ? "Search the selected historical world without loading a complete graph."
      : "Explore saved people, relationships, and events.";
    graph.setAttribute("aria-label", "Saved world graph");
    history.hidden = true;
    authorLink.hidden = true;
    setScene(projectScene(next, bondLabel), inspector);
    status.textContent = next.scene_limited
      ? "Search-only mode. Graph editing and snapshot-wide discovery are unavailable for this view."
      : revision !== null
        ? "Read-only author revision. Clear the author revision field to resume editing."
        : "World loaded. Edits are saved locally.";
  }

  function runQuery(message: string, execute: (requestId: string) => Promise<SearchResult>, page = 1) {
    if (!snapshot || saving) return;
    const sequence = ++querySequence;
    cancelActiveQuery();
    const requestId = crypto.randomUUID();
    activeQuery = requestId;
    cancelSearch.hidden = false;
    cancelSearch.disabled = false;
    pageCursor = undefined;
    nextSearchPage.hidden = true;
    nextSearchPage.disabled = true;
    queryResults.replaceChildren();
    queryStatus.textContent = message;
    void (async () => {
      try {
        const result = await execute(requestId);
        if (sequence !== querySequence || !snapshot) return;
        if (result.revision !== snapshot.view_revision) throw new Error("Search response belongs to a different author revision. Refresh the view.");
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
          if (snapshot.scene_limited && row.kind === "person") {
            const focus = document.createElement("button");
            focus.type = "button";
            focus.textContent = "Explore this person";
            focus.setAttribute("aria-label", "Explore " + row.label + " " + row.id);
            focus.addEventListener("click", () => {
              if (sequence !== querySequence || !snapshot?.scene_limited || saving) return;
              focusedPerson = { id: row.id, label: row.label };
              options(field<HTMLSelectElement>(pathForm, "start"), [focusedPerson]);
              options(field<HTMLSelectElement>(participantEventsForm, "participant"), [focusedPerson]);
              syncButtons();
              field(pathForm, "kind").focus();
            });
            const pageButton = document.createElement("button");
            pageButton.type = "button";
            pageButton.textContent = "Open character page";
            pageButton.addEventListener("click", () => {
              const current = snapshot;
              if (!current || sequence !== querySequence) return;
              pageButton.disabled = true;
              void openEntryEditor(item, current.world.id, row.id, current.day, current.view_revision,
                !revisionInput.value.trim(), refresh, current.people, target => {
                  if (sequence !== querySequence) return;
                  item.replaceChildren();
                  void openEntryEditor(item, current.world.id, target, current.day, current.view_revision,
                    !revisionInput.value.trim(), refresh, current.people);
                });
            });
            item.append(pageButton);
            item.append(focus);
          }
          const references = document.createElement("button");
          references.type = "button";
          references.className = "query-references";
          references.textContent = "Find references";
          references.setAttribute("aria-label", "Find references to " + row.label + " " + row.id);
          references.addEventListener("click", () => runSearch("backlinks", row.id));
          item.append(references);
          const authored = document.createElement("button");
          authored.type = "button";
          authored.textContent = "Find authored page links";
          authored.addEventListener("click", async () => {
            const current = snapshot;
            if (!current || sequence !== querySequence) return;
            authored.disabled = true;
            try {
              const found = await invoke<Backlinks>("workspace_author_entry_backlinks", {
                worldId: current.world.id, entryId: row.id, day: current.day,
                revision: current.view_revision, requestId: crypto.randomUUID(),
              });
              if (sequence !== querySequence || !item.isConnected) return;
              const list = document.createElement("ul");
              for (const match of found.matches) {
                const li = document.createElement("li");
                const open = document.createElement("button");
                open.type = "button";
                open.textContent = match.label + " · " + match.source.id;
                open.addEventListener("click", () => {
                  if (sequence !== querySequence) return;
                  item.replaceChildren();
                  void openEntryEditor(item, current.world.id, match.source.id, current.day,
                    current.view_revision, !revisionInput.value.trim(), refresh, current.people);
                });
                li.append(open);
                list.append(li);
              }
              const status = document.createElement("p");
              status.textContent = found.truncated ? "First 50 authored references." : found.matches.length + " authored references.";
              item.append(status, list);
            } catch (error) {
              if (sequence === querySequence && item.isConnected) {
                authored.disabled = false;
                const status = document.createElement("p");
                status.textContent = String(error);
                item.append(status);
              }
            }
          });
          item.append(authored);
          if (row.kind === "claim") {
            const mentions = document.createElement("button");
            mentions.type = "button";
            mentions.textContent = "Find mentions in this claim";
            mentions.setAttribute("aria-label", "Find mentions in " + row.label + " " + row.id);
            mentions.addEventListener("click", () => {
              if (sequence !== querySequence || !snapshot || saving) return;
              runDiscovery({ id: row.id, label: row.label });
            });
            item.append(mentions);
          }
          if (row.kind === "claim" && row.statement !== undefined && row.statement !== null) {
            const statement = document.createElement("blockquote");
            statement.textContent = row.statement;
            item.append(statement);
            const citation = document.createElement("p");
            citation.className = "hint";
            citation.textContent = "Source citation: " + (row.source_citation || "Unavailable at selected view");
            item.append(citation);
          }
          queryResults.append(item);
        }
        pageNumber = page;
        pageCursor = result.next_cursor || undefined;
        nextSearchPage.hidden = !pageCursor;
        nextSearchPage.disabled = !pageCursor || saving;
        queryStatus.textContent = result.rows.length + " match(es) at author revision " + result.revision +
          (page > 1 || pageCursor ? " · page " + page : "") +
          (pageCursor ? " · more results available." : result.truncated ? " · more results exist; narrow the search." : "");
      } catch (error) {
        if (sequence === querySequence) queryStatus.textContent = "Search failed: " + String(error);
      } finally {
        if (sequence === querySequence) {
          activeQuery = undefined;
          cancelSearch.hidden = true;
          cancelSearch.disabled = true;
        }
      }
    })();
  }

  function reviewCandidate(candidate: MentionDiscovery["rows"][number], decision: "approved" | "dismissed") {
    if (!snapshot || saving || selectedRevision() !== null) return;
    const focus = discoveryFocus;
    const request = {
      world_id: snapshot.world.id,
      day: snapshot.day,
      expected_revision: snapshot.world.revision,
      source_id: candidate.source_id,
      source_branch: candidate.source_branch,
      source_state_revision: candidate.source_state_revision,
      source_revision: candidate.source_revision,
      target_id: candidate.target_id,
      start_byte: candidate.start_byte,
      end_byte: candidate.end_byte,
      decision,
    };
    void (async () => {
      try {
        saving = true;
        syncButtons();
        await invoke<number>("workspace_review_mention", { request });
        saving = false;
        await refresh();
        runDiscovery(focus);
      } catch (error) {
        discoveryStatus.textContent = "Unable to review mention: " + String(error);
      } finally {
        saving = false;
        syncButtons();
      }
    })();
  }

  function cancelActiveDiscovery() {
    const requestId = activeDiscovery;
    activeDiscovery = undefined;
    cancelDiscovery.hidden = true;
    cancelDiscovery.disabled = true;
    if (requestId) void invoke("workspace_cancel_query", { requestId }).catch(() => {});
  }

  cancelDiscovery.addEventListener("click", () => {
    ++discoverySequence;
    cancelActiveDiscovery();
    discoveryResults.replaceChildren();
    discoveryStatus.textContent = "Mention scan cancellation requested.";
  });

  discoverButton.addEventListener("click", () => runDiscovery());

  function runDiscovery(focus?: Option) {
    if (!snapshot || saving) return;
    const worldId = snapshot.world.id;
    const day = snapshot.day;
    const revision = snapshot.view_revision;
    const sequence = ++discoverySequence;
    cancelActiveDiscovery();
    const requestId = crypto.randomUUID();
    activeDiscovery = requestId;
    discoveryFocus = focus;
    cancelDiscovery.hidden = false;
    cancelDiscovery.disabled = false;
    discoveryResults.replaceChildren();
    discoveryStatus.textContent = focus
      ? "Checking only claim: " + focus.label + "…"
      : "Checking source statements for possible names…";
    void (async () => {
      try {
        const result = await invoke<MentionDiscovery>("workspace_discover_mentions", {
          worldId, day, revision, limit: 100, requestId,
          ...(focus ? { sourceId: focus.id } : {}),
        });
        if (sequence !== discoverySequence || !snapshot) return;
        if (result.revision !== snapshot.view_revision) throw new Error("Discovery response belongs to a different author revision. Refresh the view.");
        const visiblePeople = new Set(snapshot.people.map((person) => person.id));
        for (const candidate of result.rows) {
          const item = document.createElement("li");
          const name = document.createElement("p");
          name.textContent = "Candidate: " + candidate.target_label +
            (candidate.matched_label && candidate.matched_label !== candidate.target_label
              ? " (matched alias: " + candidate.matched_label + ")" : "") +
            " · " + candidate.target_id +
            " · " + candidate.collisions + " possible identity/identities for this occurrence";
          const provenance = document.createElement("p");
          provenance.className = "hint";
          provenance.textContent = "From claim " + candidate.source_label + " · " + candidate.source_id +
            " · author revision " + candidate.source_revision +
            " · bytes " + candidate.start_byte + "–" + candidate.end_byte;
          const excerpt = document.createElement("blockquote");
          excerpt.textContent = candidate.statement;
          const caution = document.createElement("p");
          caution.className = "hint";
          caution.textContent = candidate.disposition === "approved"
            ? "Approved hyperlink only — no membership or relationship inferred."
            : candidate.disposition === "dismissed"
              ? "Dismissed candidate — no hyperlink recorded."
              : "Candidate only — no hyperlink, membership, or relationship recorded.";
          item.append(name, provenance, excerpt, caution);
          if (selectedRevision() === null) {
            for (const decision of ["approved", "dismissed"] as const) {
              const review = document.createElement("button");
              review.type = "button";
              review.textContent = decision === "approved" ? "Approve link" : "Dismiss candidate";
              review.disabled = candidate.disposition === decision || saving;
              review.addEventListener("click", () => reviewCandidate(candidate, decision));
              item.append(review);
            }
          }
          if (visiblePeople.has(candidate.target_id)) {
            const inspect = document.createElement("button");
            inspect.type = "button";
            inspect.textContent = "Inspect candidate person";
            inspect.addEventListener("click", () => selectNode(candidate.target_id));
            item.append(inspect);
          }
          discoveryResults.append(item);
        }
        discoveryStatus.textContent = result.rows.length + " possible name candidate(s) at author revision " +
          result.revision + (focus ? " · claim: " + focus.label : "") +
          (result.truncated ? " · more candidates exist; results are limited." : "");
      } catch (error) {
        if (sequence === discoverySequence) {
          discoveryStatus.textContent = "Discovery failed: " + String(error);
        }
      } finally {
        if (sequence === discoverySequence) {
          activeDiscovery = undefined;
          cancelDiscovery.hidden = true;
          cancelDiscovery.disabled = true;
        }
      }
    })();
  }

  cancelSearch.addEventListener("click", () => {
    if (!activeQuery) return;
    ++querySequence;
    cancelActiveQuery();
    pageCursor = undefined;
    nextSearchPage.hidden = true;
    nextSearchPage.disabled = true;
    queryResults.replaceChildren();
    queryStatus.textContent = "Search cancellation requested.";
  });

  nextSearchPage.addEventListener("click", () => {
    if (!snapshot || saving || !pageCursor) return;
    const worldId = snapshot.world.id;
    const day = snapshot.day;
    const revision = snapshot.view_revision;
    const token = pageCursor;
    runQuery("Loading the next historical results…",
      (requestId) => invoke<SearchResult>("workspace_next_search_page", { worldId, day, revision, token, requestId }), pageNumber + 1);
  });

  function runSearch(mode: string, term: string, limit = 100) {
    if (!snapshot || saving) return;
    const worldId = snapshot.world.id;
    const day = snapshot.day;
    const revision = snapshot.view_revision;
    runQuery(mode === "backlinks" ? "Finding incoming references…" : "Searching historical records…",
      (requestId) => invoke<SearchResult>("workspace_search", { worldId, day, revision, mode, term, limit, requestId }));
  }

  queryForm.addEventListener("submit", (event) => {
    event.preventDefault();
    runSearch(field<HTMLSelectElement>(queryForm, "mode").value,
      field<HTMLInputElement>(queryForm, "term").value.trim());
  });

  function renderSavedParameters() {
    const query = savedQueries.find((item) => item.id === savedSelect.value);
    const parameters = query?.parameters ?? [];
    parameterInputs.clear();
    parameterFields.replaceChildren();
    parameterFields.hidden = parameters.length === 0;
    field<HTMLInputElement>(saveQueryForm, "parameterize").checked = parameters.length > 0;
    if (!parameters.length) return;
    const legend = document.createElement("legend");
    legend.textContent = "Run-time parameters (defaults from saved definition)";
    parameterFields.append(legend);
    for (const parameter of parameters) {
      const label = document.createElement("label");
      label.textContent = parameter.name.replaceAll("_", " ");
      let control: HTMLInputElement | HTMLSelectElement;
      if (parameter.default.type === "object_kind") {
        control = document.createElement("select");
        for (const kind of ["person", "definition", "relationship", "event_involvement", "actor_action", "historical_event", "source", "claim"]) {
          control.add(new Option(kind.replaceAll("_", " "), kind));
        }
      } else {
        control = document.createElement("input");
        control.maxLength = parameter.default.type === "source" ? 36 : 800;
      }
      control.value = parameter.default.type === "source" ? parameter.default.source_id : parameter.default.value;
      parameterInputs.set(parameter.name, control);
      label.append(control);
      parameterFields.append(label);
    }
  }

  savedSelect.addEventListener("change", () => {
    const query = savedQueries.find((item) => item.id === savedSelect.value);
    if (query) {
      field<HTMLInputElement>(saveQueryForm, "name").value = query.name;
      if (query.mode !== "composite") {
        field<HTMLInputElement>(queryForm, "term").value = query.term;
        field<HTMLSelectElement>(queryForm, "mode").value = query.mode;
      }
    }
    renderSavedParameters();
    syncButtons();
  });

  runSavedQuery.addEventListener("click", () => {
    const query = savedQueries.find((item) => item.id === savedSelect.value);
    if (!query || !snapshot) return;
    if (query.parameters?.length) {
      const bindings = query.parameters.map((parameter) => {
        const value = parameterInputs.get(parameter.name)?.value ?? "";
        return { name: parameter.name, value: parameter.default.type === "source"
          ? { type: "source", world_id: snapshot!.world.id, source_id: value }
          : { type: parameter.default.type, value } };
      });
      const request = {
        world_id: snapshot.world.id,
        day: snapshot.day,
        revision: snapshot.view_revision,
        query_id: query.id,
        bindings,
      };
      runQuery("Replaying saved query with typed parameters…",
        (requestId) => invoke<SearchResult>("workspace_replay_saved_query", { request, requestId }));
      return;
    }
    if (query.mode === "composite") {
      if (!query.predicate || !snapshot) {
        queryStatus.textContent = "Saved composite predicate is unavailable.";
        return;
      }
      const request = {
        world_id: snapshot.world.id,
        day: snapshot.day,
        revision: snapshot.view_revision,
        predicate: query.predicate,
        limit: query.limit,
      };
      runQuery("Replaying saved historical predicates…",
        (requestId) => invoke<SearchResult>("workspace_composite_query", { request, requestId }));
    } else {
      field<HTMLInputElement>(queryForm, "term").value = query.term;
      field<HTMLSelectElement>(queryForm, "mode").value = query.mode;
      runSearch(query.mode, query.term, query.limit);
    }
  });

  saveQueryForm.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot || saving) return;
    if (selectedRevision() !== null) {
      queryStatus.textContent = "Historical author revisions are read-only.";
      return;
    }
    const term = field<HTMLInputElement>(queryForm, "term").value.trim();
    if (!term) {
      queryStatus.textContent = "Enter a search phrase before saving.";
      return;
    }
    const request = {
      world_id: snapshot.world.id,
      expected_revision: snapshot.world.revision,
      query_id: savedSelect.value || null,
      name: value(saveQueryForm, "name"),
      ...(field<HTMLInputElement>(saveQueryForm, "parameterize").checked ? { parameterize: true } : {}),
      mode: value(queryForm, "mode"),
      term,
      limit: 100,
    };
    void (async () => {
      try {
        saving = true;
        syncButtons();
        const savedId = await invoke<string>("workspace_save_query", { request });
        await refresh();
        savedSelect.value = savedId;
        renderSavedParameters();
        queryStatus.textContent = "Saved search definition to world history.";
      } catch (error) {
        queryStatus.textContent = "Unable to save search: " + String(error);
      } finally {
        saving = false;
        syncButtons();
      }
    })();
  });

  function compositePredicate(): Predicate | null {
    const filters: Predicate[] = [];
    const text = value(compositeForm, "name").trim();
    const alias = value(compositeForm, "alias").trim();
    const labelPhrase = value(compositeForm, "label_phrase").trim();
    const claimPhrase = value(compositeForm, "claim_phrase").trim();
    const kind = value(compositeForm, "kind");
    const source = value(compositeForm, "source").trim();
    if (text) filters.push({ op: "contains", text });
    if (alias) filters.push({ op: "exact_alias", text: alias });
    if (labelPhrase) filters.push({ op: "full_text", text: labelPhrase });
    if (claimPhrase) filters.push({ op: "claim_full_text", text: claimPhrase });
    if (kind) filters.push({ op: "object_kind", kind });
    if (source) filters.push({ op: "has_source", source_id: source });
    if (!filters.length) return null;
    let predicate: Predicate = filters.length === 1 ? filters[0] : {
      op: value(compositeForm, "combine") as "and" | "or", filters,
    };
    if (field<HTMLInputElement>(compositeForm, "negate").checked) {
      predicate = { op: "not", filter: predicate };
    }
    return predicate;
  }

  compositeForm.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot || saving) return;
    const predicate = compositePredicate();
    if (!predicate) {
      queryStatus.textContent = "Select at least one historical object filter.";
      return;
    }
    const request = {
      world_id: snapshot.world.id,
      day: snapshot.day,
      revision: snapshot.view_revision,
      predicate,
      limit: 100,
    };
    runQuery("Evaluating bounded historical predicates…",
      (requestId) => invoke<SearchResult>("workspace_composite_query", { request, requestId }));
  });

  saveComposite.addEventListener("click", () => {
    if (!snapshot || saving) return;
    if (selectedRevision() !== null) {
      queryStatus.textContent = "Historical author revisions are read-only.";
      return;
    }
    const predicate = compositePredicate();
    const queryName = value(saveQueryForm, "name").trim();
    if (!predicate || !queryName) {
      queryStatus.textContent = "Enter a saved search name and select a combined filter.";
      return;
    }
    const request = {
      world_id: snapshot.world.id,
      expected_revision: snapshot.world.revision,
      query_id: savedSelect.value || null,
      name: queryName,
      ...(field<HTMLInputElement>(saveQueryForm, "parameterize").checked ? { parameterize: true } : {}),
      mode: "composite",
      term: "",
      predicate,
      limit: 100,
    };
    void (async () => {
      try {
        saving = true;
        syncButtons();
        const savedId = await invoke<string>("workspace_save_query", { request });
        await refresh();
        savedSelect.value = savedId;
        renderSavedParameters();
        queryStatus.textContent = "Saved combined query definition to author history.";
      } catch (error) {
        queryStatus.textContent = "Unable to save combined query: " + String(error);
      } finally {
        saving = false;
        syncButtons();
      }
    })();
  });

  participantEventsForm.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot || (snapshot.scene_limited && !focusedPerson) || saving) return;
    const request = {
      world_id: snapshot.world.id,
      day: snapshot.day,
      revision: snapshot.view_revision,
      participant_id: value(participantEventsForm, "participant"),
      role: value(participantEventsForm, "role"),
      limit: 100,
    };
    runQuery("Finding explicitly recorded historical event participation…",
      (requestId) => invoke<SearchResult>("workspace_query_participant_events", { request, requestId }));
  });

  pathForm.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!snapshot || (snapshot.scene_limited && !focusedPerson) || saving) return;
    const request = {
      world_id: snapshot.world.id,
      day: snapshot.day,
      revision: snapshot.view_revision,
      start_id: value(pathForm, "start"),
      kind: value(pathForm, "kind").trim(),
      direction: value(pathForm, "direction"),
      max_depth: Number(value(pathForm, "max_depth")),
      max_visited: Number(value(pathForm, "max_visited")),
      limit: 100,
    };
    runQuery("Exploring historical relationship paths…",
      (requestId) => invoke<SearchResult>("workspace_query_path", { request, requestId }));
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
            if (snapshot.scene_limited) throw new Error("Graph editing requires a complete scene.");
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
  field<HTMLSelectElement>(aliasForm, "alias_id").addEventListener("change", () => {
    const alias = snapshot?.aliases?.find((entry) => entry.id === value(aliasForm, "alias_id"));
    if (alias) {
      field<HTMLSelectElement>(aliasForm, "person_id").value = alias.person_id;
      field<HTMLInputElement>(aliasForm, "label").value = alias.label;
      field<HTMLInputElement>(aliasForm, "starts_at").value = alias.starts_at?.toString() ?? "";
      field<HTMLInputElement>(aliasForm, "ends_before").value = alias.ends_before?.toString() ?? "";
      field<HTMLInputElement>(aliasForm, "active").checked = alias.active;
    } else {
      field<HTMLInputElement>(aliasForm, "label").value = "";
      field<HTMLInputElement>(aliasForm, "starts_at").value = "";
      field<HTMLInputElement>(aliasForm, "ends_before").value = "";
      field<HTMLInputElement>(aliasForm, "active").checked = true;
    }
  });
  bind(aliasForm, async ({ world_id, expected_revision }) => {
    const optionalDay = (name: string): number | null => {
      const raw = value(aliasForm, name).trim();
      if (!raw) return null;
      const parsed = Number(raw);
      if (!/^-?\d+$/.test(raw) || !Number.isSafeInteger(parsed)) {
        throw new Error("Alias dates must be whole-number historical days.");
      }
      return parsed;
    };
    await invoke("workspace_save_person_alias", {
      request: {
        world_id,
        expected_revision,
        day: currentDay(),
        person_id: value(aliasForm, "person_id"),
        alias_id: value(aliasForm, "alias_id") || null,
        label: value(aliasForm, "label"),
        starts_at: optionalDay("starts_at"),
        ends_before: optionalDay("ends_before"),
        active: field<HTMLInputElement>(aliasForm, "active").checked,
      },
    });
    const personId = value(aliasForm, "person_id");
    aliasForm.reset();
    field<HTMLSelectElement>(aliasForm, "person_id").value = personId;
    field<HTMLSelectElement>(aliasForm, "alias_id").value = "";
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
  refreshView.addEventListener("click", reload);
  dayInput.addEventListener("change", reload);
  revisionInput.addEventListener("change", reload);
  try {
    await refreshWorlds();
  } catch (error) {
    status.textContent = "Author workspace unavailable: " + String(error);
  }
}
