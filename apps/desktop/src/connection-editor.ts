import { invoke } from "@tauri-apps/api/core";
import { appendReferencePicker, type Candidate } from "./reference-picker";
import { canLeaveEntryDrafts } from "./entry-drafts";

type Context = { world_id: string; day: number; revision: number };
type Predicate = { kind: string; label: string; roles?: string[] };
const institutions = new Set(["organization", "household", "family_house", "faction"]);
const ordinary = (kind: string, label: string): Predicate => ({kind, label});

// A deliberately bounded UI subset of store/network.rs validation. The native
// command owns validity, evidence, endpoint existence, interval and cycle checks.
// Causal, succession, redirect and profile-specific mutations stay in their
// specialist tools until their contextual controls are verified.
export function compatibleConnections(subject: string, target: string): Predicate[] {
  const result: Predicate[] = [];
  if (subject === "person") {
    if (institutions.has(target)) result.push(ordinary("membership", "Member of"));
    if (target === "person") result.push(ordinary("marriage", "Married to"));
    if (target === "office") result.push(ordinary("office_holding", "Holds office"));
    if (target === "title") result.push(ordinary("title_holding", "Holds title"), ordinary("heir_designation", "Designated heir"));
    if (target === "population") result.push(ordinary("population_member", "Member of population"));
  }
  if (subject === "office" && institutions.has(target)) result.push(ordinary("office_of", "Office of"));
  if (subject === "title" && institutions.has(target)) result.push(ordinary("title_of", "Title of"));
  if ((subject === "person" || institutions.has(subject)) && target === "asset") result.push(ordinary("ownership", "Owns asset"));
  if (subject === "conflict_issue" && target === "conflict") result.push(ordinary("issue_of", "Issue of"));
  if (subject === "conflict_phase" && target === "conflict") result.push(ordinary("phase_of", "Phase of"));
  if (subject === "faction" && target === "faction") result.push(ordinary("alliance", "Allied with"));
  if (subject === "faction" && ["conflict", "conflict_issue"].includes(target)) {
    result.push({kind:"conflict_position",label:"Position in conflict",roles:["supports","opposes","mediates","neutral"]});
    result.push(ordinary("settlement", "Settlement"));
  }
  if (target === "place") {
    const roles = subject === "person" ? ["birth","death","presence","residence"] :
      subject === "household" ? ["residence","headquarters"] :
      institutions.has(subject) ? ["headquarters"] :
      subject === "asset" ? ["asset_placement"] :
      subject === "population" ? ["population_origin"] :
      subject === "historical_event" ? ["event_site"] : [];
    if (roles.length) result.push({kind:"located_at",label:"Located at",roles});
    if (subject === "place") result.push(ordinary("proper_part_within", "Physically within"), ordinary("admin_part_of", "Administratively part of"));
  }
  return result;
}

export function appendConnectionEditor(parent: HTMLElement, subject: Candidate,
  context: Context, valid: () => boolean, saved: () => Promise<void>) {
  if (!subject.kind) return;
  const subjectKind = subject.kind;
  const host = document.createElement("form");
  host.className = "editor-form";
  host.dataset.connectionEditor = "";
  host.innerHTML = `<h3>Add documented connection</h3><p class="hint">Outgoing connection from the selected identity. Start is the selected world day. Choose another day in the world context for an earlier or later fact. Existing facts are not overwritten.</p>
    <label>Connection target identity<input name="target" required></label><div data-target-picker></div>
    <label>Connection type<select name="predicate" required></select></label>
    <label data-role-text>Role or relationship rationale<input name="role" required maxlength="800"></label>
    <label data-role-select hidden>Location role or conflict stance<select name="role_choice"></select></label>
    <label>Connection evidence source<input name="source" required></label><div data-source-picker></div>
    <label>Ends before day (optional)<input name="end" type="number" step="1"></label>
    <p data-connection-preview class="hint"></p><button type="submit" disabled>Record connection</button><p data-draft-status tabindex="-1" role="status"></p>`;
  parent.append(host);
  const target = host.elements.namedItem("target") as HTMLInputElement;
  const predicate = host.elements.namedItem("predicate") as HTMLSelectElement;
  const role = host.elements.namedItem("role") as HTMLInputElement;
  const choice = host.elements.namedItem("role_choice") as HTMLSelectElement;
  const source = host.elements.namedItem("source") as HTMLInputElement;
  const end = host.elements.namedItem("end") as HTMLInputElement;
  const status = host.querySelector<HTMLElement>("[data-draft-status]")!;
  const preview = host.querySelector<HTMLElement>("[data-connection-preview]")!;
  const submit = host.querySelector<HTMLButtonElement>('button[type="submit"]')!;
  let selected: Candidate | undefined;
  let writing = false;
  let predicates: Predicate[] = [];
  function syncRole() {
    const p = predicates.find(p => p.kind === predicate.value);
    const constrained = Boolean(p?.roles?.length);
    role.disabled = constrained;
    role.required = !constrained;
    host.querySelector<HTMLElement>("[data-role-text]")!.hidden = constrained;
    choice.disabled = !constrained;
    choice.required = constrained;
    host.querySelector<HTMLElement>("[data-role-select]")!.hidden = !constrained;
    choice.replaceChildren(...(p?.roles ?? []).map(r => new Option(r.replaceAll("_"," "),r)));
    role.maxLength = ["proper_part_within","admin_part_of"].includes(predicate.value) ? 80 : 800;
    preview.textContent = selected && p ? subject.label + " → " + p.label + " → " + selected.label + " · day " + context.day : "Choose a compatible target from the search results.";
    submit.disabled = writing || !p || selected?.id !== target.value;
  }
  function syncTarget() {
    predicates = selected?.kind && selected.id === target.value
      ? compatibleConnections(subjectKind, selected.kind) : [];
    predicate.replaceChildren(...predicates.map(p => new Option(p.label,p.kind)));
    syncRole();
  }
  target.addEventListener("input", () => { selected = undefined; syncTarget(); });
  predicate.addEventListener("change", syncRole);
  appendReferencePicker(host.querySelector<HTMLElement>("[data-target-picker]")!, target, context, "entry", "connection target",
    candidate => { selected = candidate; syncTarget(); },
    candidate => candidate.id !== subject.id && Boolean(candidate.kind && compatibleConnections(subjectKind,candidate.kind).length));
  appendReferencePicker(host.querySelector<HTMLElement>("[data-source-picker]")!, source, context, "source", "connection source");
  syncTarget();
  host.addEventListener("submit", async event => {
    event.preventDefault();
    if (writing || !valid() || !host.isConnected || !canLeaveEntryDrafts()) return;
    const p = predicates.find(p=>p.kind===predicate.value);
    if (!selected || selected.id !== target.value || !p) return;
    const until = end.value ? Number(end.value) : null;
    if (until !== null && (!Number.isSafeInteger(until) || until <= context.day)) {
      status.textContent = "The end must be a whole day later than the start."; return;
    }
    const roleValue = (p.roles ? choice.value : role.value).trim();
    if (!roleValue || !source.value.trim()) { status.textContent = "A role and evidence source are required."; return; }
    const ref = (id: string) => ({world:context.world_id,id});
    writing = true;
    const controls = [...host.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("input,select,button")];
    const states = controls.map(control => control.disabled);
    controls.forEach(control => control.disabled = true);
    host.dataset.entryDraftState = "saving";
    status.textContent = "Recording connection…";
    try {
      await invoke("workspace_record_network_fact",{request:context,input:{
        subject:ref(subject.id),target:ref(selected.id),source:ref(source.value.trim()),
        kind:p.kind,role:roleValue,starts_at:context.day,ends_before:until,
      }});
    } catch (error) {
      delete host.dataset.entryDraftState;
      writing = false; controls.forEach((control,index) => control.disabled = states[index]);
      syncRole(); status.textContent = String(error); return;
    }
    delete host.dataset.entryDraftState;
    status.textContent = "Connection recorded.";
    if (valid() && host.isConnected) {
      try { await saved(); } catch (error) { status.textContent = "Connection recorded, but refresh failed: " + String(error); }
    }
  });
}
