import { canLeaveEntryDrafts } from "./entry-drafts";
import { invoke } from "@tauri-apps/api/core";
type Context = { world_id: string; branch_id?: string; day: number; revision: number };
const selectedBranches = new Map<string, string>();
type Ref = { world: string; id: string };
type Fact = { id: string; input: { kind: string; role: string }; subject_label: string; target_label: string };
type Classification = { id: string; profile: string; descriptor_id: string; descriptor_revision: number;
  descriptor_branch: string; descriptor_state_revision: number; source_id: string; starts_at: number;
  ends_before: number | null; recorded_at: number };
type FactReference = Ref & { family: "network" | "parentage" | "classification" | "site_area" };
type SemanticFact = { reference: FactReference; subject: Ref; target: Ref | null; source: Ref | null;
  descriptor: Ref | null; description: string; rationale: string; recorded_at: number; source_branch: string };
type AreaReading = { status: "unknown" } | { status: "known"; amount: number; unit: "hectare" | "square_metre" };
type SiteArea = { id: string; descriptor_id: string; descriptor_revision: number; source_id: string;
  reading: AreaReading; starts_at: number; ends_before: number | null; recorded_at: number; corrects: string | null };
type Ghost = { id: string; claim: Ref; label: string; kind: string; reviewed: boolean; promoted: Ref | null };

export function openNetworkStudio(parent: HTMLElement, request: Context, editable: boolean,
  refresh: () => Promise<void>, valid: () => boolean) {
  parent.querySelector("[data-network-studio]")?.remove();
  const selected = selectedBranches.get(request.world_id);
  if (selected) request = { ...request, branch_id: selected };
  const host = document.createElement("details");
  host.dataset.networkStudio = "";
  host.className = "query-panel";
  host.innerHTML = `<summary>Institution, causal and draft identity studio</summary>
    <p class="hint">Author-only. Stable object IDs come from historical search. Source evidence is required for network facts. Spatial containment uses an exact hierarchy name; Located at roles: birth, death, residence, presence, event_site, headquarters, asset_placement, population_origin.</p>
    <section class="branch-controls"><h3>Alternate historical timelines</h3><p data-selected-branch></p><button type="button" data-load-branches>Load branches</button><select aria-label="Historical branch" data-branch hidden></select><button type="button" data-fork>Create fork at selected day</button><p class="hint">Branch applies only to this network studio; the main graph and page editor remain on main.</p></section>
    <form data-identity class="editor-form"><fieldset><legend>Create typed identity</legend>
      <label>Identity label <input name="label" required maxlength="800"></label>
      <label>Identity type <select name="kind"><option value="organization">Organization</option><option value="household">Household</option><option value="family_house">Family house</option><option value="faction">Faction</option><option value="conflict">Conflict</option><option value="office">Office</option><option value="title">Hereditary or legal title</option><option value="asset">Asset</option><option value="population">Fictional population</option><option value="conflict_issue">Conflict issue</option><option value="conflict_phase">Conflict phase</option><option value="succession_rule">Succession rule</option><option value="concept">Concept</option><option value="place">Place</option><option value="person">Person</option></select></label>
      <button>Create identity</button></fieldset></form>
    <form data-semantic-picker class="editor-form"><fieldset><legend>Find compatible identities</legend>
      <p class="hint">Search by a word or phrase in the selected branch and revision. For semantic network endpoints, choose the fact type and start day first; the search matches names containing the text and requires the corresponding profile on that day. Selecting a result only fills a field.</p>
      <label>Selection destination <select name="destination"></select></label>
      <label>Identity name or phrase <input name="phrase" required maxlength="800"></label>
      <button>Find compatible identities</button><button type="button" data-cancel-picker hidden>Cancel identity search</button>
      <ul data-picker-results aria-label="Compatible identity results"></ul>
    </fieldset></form>
    <form data-classification class="editor-form"><fieldset><legend>Record sourced concept classification</legend>
      <p class="hint">Use a Concept identity, a Tag definition and a source from historical search. Classification is not a claim of membership or ancestry.</p>
      <label>Concept identity <input name="subject" required></label>
      <label>Semantic profile <select name="profile"><option value="culture">Culture</option><option value="language">Language</option><option value="material">Material</option><option value="technology">Technology</option></select></label>
      <label>Tag definition identity <input name="descriptor" required></label>
      <label>Classification evidence source <input name="source" required></label>
      <label>Classification starts at day <input name="start" type="number" required></label>
      <label>Classification ends before day <input name="end" type="number"></label>
      <button>Record classification</button></fieldset></form>
    <form data-classification-focus class="editor-form"><label>Classification focus concept <input name="subject" required></label><button>Inspect classifications</button></form>
    <ul data-classifications aria-label="Historical concept classifications"></ul>
    <form data-semantic-focus class="editor-form"><label>Evidence or identity backlinks <input name="target" required></label><button>Inspect semantic backlinks</button></form>
    <ul data-semantic-links aria-label="Historical semantic backlinks"></ul>
    <form data-semantic-fact class="editor-form"><label>Fact family <select name="family"><option value="network">Network</option><option value="parentage">Parentage</option><option value="classification">Classification</option><option value="site_area">Site area</option></select></label>
      <label>Stable fact identity <input name="id" required></label><button>Resolve fact</button></form>
    <p data-semantic-detail role="status"></p>
    <form data-fact-annotation class="editor-form"><fieldset><legend>Annotate a resolved fact</legend>
      <label>Annotation fact family <select name="family"><option value="network">Network</option><option value="parentage">Parentage</option><option value="classification">Classification</option><option value="site_area">Site area</option></select></label>
      <label>Annotation fact UUID <input name="fact" required></label>
      <label>Sourced claim UUID <input name="claim"></label>
      <button type="submit">Link sourced annotation</button><button type="button" data-read-annotations>Inspect annotations</button>
    </fieldset></form><ul data-annotations aria-label="Sourced fact annotations"></ul>
    <form data-retire-descriptor class="editor-form"><fieldset><legend>Retire semantic descriptor</legend>
      <p class="hint">Blocks new assignments from this day in the selected branch. Existing pinned records are retained.</p>
      <label>Retire Tag definition UUID <input name="descriptor" required></label>
      <label><input type="checkbox" name="confirm" required> Confirm retirement</label>
      <button>Retire descriptor</button></fieldset></form>
    <form data-site-area class="editor-form"><fieldset><legend>Record sourced site area</legend>
      <p class="hint">An area reading does not change containment or map geometry. Missing, unknown and known zero differ; units are never converted automatically.</p>
      <label>Measured Place identity <input name="place" required></label>
      <label>Area Tag definition identity <input name="descriptor" required></label>
      <label>Area evidence source <input name="source" required></label>
      <label>Reading status <select name="status"><option value="known">Known</option><option value="unknown">Explicit unknown</option></select></label>
      <label>Area amount <input name="amount" type="number" min="0" step="1" required></label>
      <label>Area unit <select name="unit"><option value="hectare">Hectare</option><option value="square_metre">Square metre</option></select></label>
      <label>Area starts at day <input name="start" type="number" required></label>
      <label>Area ends before day <input name="end" type="number"></label>
      <label>Correct prior area fact (optional) <input name="corrects" placeholder="Stable fact UUID; same interval"></label>
      <button>Record site area</button></fieldset></form>
    <form data-area-focus class="editor-form"><label>Site area focus Place <input name="place" required></label><button>Inspect site area</button></form><output data-area-result></output>
    <form data-fact class="editor-form"><fieldset><legend>Record sourced network fact</legend>
      <label>Subject identity <input name="subject" required></label><label>Target identity <input name="target" required></label>
      <label>Evidence source identity <input name="source" required></label>
      <label>Fact type <select name="kind"><option value="membership">Membership</option><option value="marriage">Marriage</option><option value="succession">Institutional succession</option><option value="confirmed_cause">Confirmed cause</option><option value="alleged_cause">Alleged cause</option><option value="conflict_position">Faction conflict stance</option><option value="office_holding">Office holding</option><option value="office_of">Office in institution</option><option value="title_holding">Title holding</option><option value="title_of">Title of institution</option><option value="ownership">Ownership</option><option value="issue_of">Issue in conflict</option><option value="phase_of">Phase in conflict</option><option value="rule_of">Succession rule of title</option><option value="population_member">Founder population affiliation</option><option value="heir_designation">Designated heir</option><option value="identity_redirect">Reviewed same identity redirect</option><option value="alliance">Faction alliance</option><option value="settlement">Faction settlement</option><option value="culture_language">Culture documented language</option><option value="working_material">Technology working material</option><option value="proper_part_within">Physical containment</option><option value="admin_part_of">Administrative containment</option><option value="located_at">Located at</option></select></label>
      <label>Role or causal rationale <input name="role" required maxlength="800"></label>
      <label>Starts at day <input name="start" type="number" required></label><label>Ends before day <input name="end" type="number"></label>
      <button>Record network fact</button></fieldset></form>
    <form data-focus class="editor-form"><label>Network focus identity <input name="focus" required></label><button>Inspect network facts</button><button type="button" data-impact>Explore confirmed impact</button></form><ul data-facts></ul><ul data-impact-results></ul><button type="button" data-resolve-identity>Trace reviewed identity</button><output data-identity-path></output>
    <form data-succession class="editor-form"><fieldset><legend>Designated succession</legend><label>Title identity <input name="title" required></label><label>Transfer evidence source <input name="source"></label><button type="button" data-preview-succession>Preview eligibility</button><button data-enact disabled>Confirm transfer</button><output data-succession-result></output></fieldset></form>
    <form data-ghost class="editor-form"><fieldset><legend>Propose identity from a claim</legend>
      <label>Claim identity <input name="claim" required></label>
      <label>Start UTF-8 byte <input name="start" type="number" min="0" required></label>
      <label>End UTF-8 byte (exclusive) <input name="end" type="number" min="1" required></label>
      <label>Proposed type <select name="kind"><option value="person">Person</option><option value="organization">Organization</option><option value="household">Household</option><option value="family_house">Family house</option><option value="faction">Faction</option><option value="conflict">Conflict</option><option value="office">Office</option><option value="title">Hereditary or legal title</option><option value="asset">Asset</option><option value="population">Fictional population</option><option value="conflict_issue">Conflict issue</option><option value="conflict_phase">Conflict phase</option><option value="succession_rule">Succession rule</option><option value="concept">Concept</option><option value="place">Place</option></select></label>
      <button>Propose ghost identity</button></fieldset></form>
    <button type="button" data-candidates>Find possible names in selected claim</button><ul data-candidate-results></ul>
    <button type="button" data-review>Review draft identities</button><ul data-ghosts></ul>
    <form data-template class="editor-form"><fieldset><legend>Save persisted page as custom template</legend>
      <p class="hint">Copies authored presentation, not resolved data. Existing instances never auto-update.</p>
      <label>Template source entry identity <input name="source" required></label>
      <label>Template name <input name="name" required maxlength="200"></label>
      <label>Existing template identity (optional revision) <input name="id"></label>
      <button>Save custom template</button></fieldset></form>
    <button type="button" data-template-list>List custom templates</button><ul data-templates></ul><p data-status role="status"></p>`;
  parent.prepend(host);
  const status = host.querySelector<HTMLElement>("[data-status]")!;
  const form = (selector: string) => host.querySelector<HTMLFormElement>(selector)!;
  const value = (f: HTMLFormElement, key: string) => String(new FormData(f).get(key) ?? "");
  const ref = (id: string) => ({ world: request.world_id, id });
  (form("[data-fact]").elements.namedItem("start") as HTMLInputElement).value = String(request.day);
  (form("[data-classification]").elements.namedItem("start") as HTMLInputElement).value = String(request.day);
  const areaForm = form("[data-site-area]");
  (areaForm.elements.namedItem("start") as HTMLInputElement).value = String(request.day);
  const areaStatus = areaForm.elements.namedItem("status") as HTMLSelectElement;
  const areaAmount = areaForm.elements.namedItem("amount") as HTMLInputElement;
  const areaUnit = areaForm.elements.namedItem("unit") as HTMLSelectElement;
  areaStatus.onchange = () => {
    areaAmount.disabled = areaUnit.disabled = areaStatus.value === "unknown";
    areaAmount.required = areaStatus.value === "known";
  };
  host.querySelectorAll("fieldset").forEach(field => field.disabled = !editable);
  // Inspection remains usable at a pinned revision; only the annotation write is disabled.
  annotationFields();
  function annotationFields() {
    const f = form("[data-fact-annotation]");
    f.querySelector("fieldset")!.disabled = false;
    (f.querySelector('button[type="submit"]') as HTMLButtonElement).disabled = !editable;
    (f.elements.namedItem("claim") as HTMLInputElement).disabled = !editable;
  }
  const selectedLabel=host.querySelector<HTMLElement>("[data-selected-branch]")!;
  selectedLabel.textContent = request.branch_id ? "Alternate branch: "+request.branch_id : "Main history";
  const chooser=host.querySelector<HTMLSelectElement>("[data-branch]")!;
  let inspection = 0;
  let pickerRequest: string | undefined;
  const cancelPicker = () => {
    const id = pickerRequest;
    pickerRequest = undefined;
    host.querySelector<HTMLButtonElement>("[data-cancel-picker]")!.hidden = true;
    if (id) void invoke("workspace_cancel_query", { requestId: id }).catch(() => {});
  };
  const picker = form("[data-semantic-picker]");
  picker.querySelector("fieldset")!.disabled = false;
  const destinations = [
    ["Classification subject", "[data-classification]", "subject", "concept", true],
    ["Classification descriptor", "[data-classification]", "descriptor", "tag", true],
    ["Classification source", "[data-classification]", "source", "source", true],
    ["Classification inspector", "[data-classification-focus]", "subject", "concept", false],
    ["Area subject", "[data-site-area]", "place", "place", true],
    ["Area descriptor", "[data-site-area]", "descriptor", "tag", true],
    ["Area source", "[data-site-area]", "source", "source", true],
    ["Area inspector", "[data-area-focus]", "place", "place", false],
    ["Annotation claim", "[data-fact-annotation]", "claim", "claim", true],
    ["Retirement descriptor", "[data-retire-descriptor]", "descriptor", "tag", true],
    ["Semantic network subject", "[data-fact]", "subject", "relation_subject", true],
    ["Semantic network target", "[data-fact]", "target", "relation_target", true],
  ] as const;
  const destination = picker.elements.namedItem("destination") as HTMLSelectElement;
  destinations.forEach(([label,,,, needsWrite], index) => {
    if (editable || !needsWrite) destination.add(new Option(label, String(index)));
  });
  const clear = (...selectors: string[]) => selectors.forEach(selector => host.querySelector(selector)?.replaceChildren());
  const beginInspection = (...selectors: string[]) => {
    cancelPicker();
    clear(...selectors);
    status.textContent = "";
    const token = ++inspection;
    const ctx = request;
    return { ctx, current: () => valid() && ctx === request && token === inspection };
  };
  // Any focus edit invalidates rows and their write actions immediately.
  for (const [selector, results] of [
    ["[data-semantic-picker]", ["[data-picker-results]"]],
    ["[data-classification-focus]", ["[data-classifications]"]],
    ["[data-semantic-focus]", ["[data-semantic-links]", "[data-semantic-detail]", "[data-annotations]"]],
    ["[data-semantic-fact]", ["[data-semantic-detail]"]],
    ["[data-fact-annotation]", ["[data-annotations]"]],
    ["[data-area-focus]", ["[data-area-result]"]],
    ["[data-focus]", ["[data-facts]", "[data-impact-results]", "[data-identity-path]"]],
    ["[data-ghost]", ["[data-candidate-results]"]],
    ["[data-succession]", ["[data-succession-result]"]],
  ] as const) {
    form(selector).addEventListener("input", () => {
      ++inspection; cancelPicker(); clear(...results); status.textContent = "";
    });
  }
  // A different fact type or effective day changes compatible endpoints.
  // Invalidate the old picker result before any late IPC reply can fill it.
  for (const event of ["input", "change"]) {
    form("[data-fact]").addEventListener(event, () => {
      ++inspection; cancelPicker(); clear("[data-picker-results]"); status.textContent = "";
    });
  }
  const choose=(id:string | undefined)=>{
    ++inspection; cancelPicker(); status.textContent = "";
    if(id)selectedBranches.set(request.world_id,id);else selectedBranches.delete(request.world_id);
    if(id)request={...request,branch_id:id};else {const {branch_id:_,...main}=request;request=main;}
    selectedLabel.textContent=id?"Alternate branch: "+id:"Main history";
    for(const selector of ["[data-picker-results]","[data-annotations]","[data-area-result]","[data-classifications]","[data-semantic-links]","[data-semantic-detail]","[data-facts]","[data-impact-results]","[data-identity-path]","[data-succession-result]","[data-ghosts]","[data-candidate-results]","[data-templates]"])host.querySelector(selector)?.replaceChildren();
    host.querySelector<HTMLButtonElement>("[data-enact]")!.disabled=true;
  };
  host.querySelector<HTMLButtonElement>("[data-load-branches]")!.onclick=async()=>{
    try {
      const {ctx,current}=beginInspection();
      const branches=await invoke<{id:string;day:number|null;created_revision:number}[]>("workspace_metadata_branches",{request:ctx});
      if(!current())return;
      chooser.replaceChildren();
      for(const branch of branches)chooser.add(new Option(branch.day===null?"Main history":"Fork day "+branch.day+" · "+branch.id,branch.day===null?"":branch.id));
      chooser.value=request.branch_id??"";chooser.hidden=false;
    }catch(error){if(valid())status.textContent=String(error);}
  };
  chooser.onchange=()=>{ if (busy || !canLeaveEntryDrafts()) { chooser.value=request.branch_id??""; return; } choose(chooser.value||undefined); };
  const forkButton=host.querySelector<HTMLButtonElement>("[data-fork]")!;
  forkButton.disabled=!editable;
  forkButton.onclick=async()=>{
    if(!valid()||busy||!editable||!canLeaveEntryDrafts())return;
    busy=true;
    try{
      const id=await invoke<string>("workspace_fork_metadata",{request});
      if(valid()){choose(id);await refresh();}
    }catch(error){if(valid())status.textContent=String(error);}
    finally{busy=false;}
  };
  let busy = false;
  async function write(command: string, args: Record<string, unknown>) {
    if (!valid() || busy || !editable || !canLeaveEntryDrafts()) return;
    busy = true;
    ++inspection; cancelPicker();
    try { await invoke(command, { request, ...args }); if (valid()) await refresh(); }
    catch (error) { if (valid()) status.textContent = String(error); }
    finally { busy = false; }
  }
  function submit(selector: string, action: (f: HTMLFormElement) => void) {
    const f = form(selector);
    f.onsubmit = event => { event.preventDefault(); if (valid()) action(f); };
  }
  host.querySelector<HTMLButtonElement>("[data-cancel-picker]")!.onclick = () => {
    ++inspection; cancelPicker(); clear("[data-picker-results]"); status.textContent = "Identity search cancelled.";
  };
  submit("[data-semantic-picker]", async f => {
    const phrase = value(f,"phrase").trim();
    if (!phrase) return;
    const chosen = destinations[Number(destination.value)];
    if (!chosen || (chosen[4] && !editable)) return;
    const {ctx,current}=beginInspection("[data-picker-results]");
    let kind: string = chosen[3];
    let selectionContext = ctx;
    if (kind === "relation_subject" || kind === "relation_target") {
      const relation = form("[data-fact]");
      const factKind = value(relation, "kind");
      const subject = kind === "relation_subject";
      if (factKind === "culture_language") kind = subject ? "culture" : "language";
      else if (factKind === "working_material") kind = subject ? "technology" : "material";
      else {
        status.textContent = "Select Culture documented language or Technology working material as the fact type.";
        return;
      }
      const raw = value(relation, "start").trim();
      const day = Number(raw);
      if (!raw || !Number.isSafeInteger(day)) {
        status.textContent = "Enter a valid integer fact start day.";
        return;
      }
      selectionContext = { ...ctx, day };
    }
    const requestId=crypto.randomUUID();
    pickerRequest=requestId;
    host.querySelector<HTMLButtonElement>("[data-cancel-picker]")!.hidden=false;
    try {
      const result = await invoke<{rows: {id: string; label: string}[]; truncated: boolean}>(
        "workspace_semantic_candidates", {request:selectionContext,requestId,kind,phrase});
      if (!current()) return;
      const list=host.querySelector("[data-picker-results]")!;
      for (const row of result.rows) {
        const li=document.createElement("li"), button=document.createElement("button");
        button.type="button"; button.textContent=row.label+" · "+row.id;
        button.onclick=()=>{
          if (!current()) return;
          const input=form(chosen[1]).elements.namedItem(chosen[2]) as HTMLInputElement;
          input.value=row.id;
          input.dispatchEvent(new Event("input", {bubbles:true}));
          ++inspection;
          list.replaceChildren();
          input.focus();
          status.textContent="Identity selected.";
        };
        li.append(button); list.append(li);
      }
      status.textContent=result.rows.length+" compatible identities."+(result.truncated?" Refine the phrase to see further matches.":"");
    } catch (error) { if(current())status.textContent=String(error); }
    finally { if(pickerRequest===requestId) {pickerRequest=undefined;host.querySelector<HTMLButtonElement>("[data-cancel-picker]")!.hidden=true;} }
  });
  submit("[data-identity]", f => void write("workspace_create_identity", { kind: value(f,"kind"), label: value(f,"label") }));
  submit("[data-classification]", f => void write("workspace_record_classification", { input: {
    subject_id: value(f,"subject"), descriptor_id: value(f,"descriptor"), source_id: value(f,"source"),
    profile: value(f,"profile"), starts_at: Number(value(f,"start")),
    ends_before: value(f,"end") ? Number(value(f,"end")) : null,
  } }));
  submit("[data-classification-focus]", async f => {
    const { ctx, current } = beginInspection("[data-classifications]");
    const subjectId = value(f,"subject");
    try {
      const rows = await invoke<Classification[]>("workspace_classifications", { request: ctx, subjectId });
      if (!current() || subjectId !== value(f,"subject")) return;
      const list = host.querySelector("[data-classifications]")!;
      list.replaceChildren();
      for (const classification of rows) {
        const row = document.createElement("li");
        row.textContent = classification.profile + " · " + classification.starts_at + "–" +
          (classification.ends_before ?? "open") + " · source " + classification.source_id +
          " · definition " + classification.descriptor_id + " @ revision " + classification.descriptor_revision + " ";
        row.append(action("Withdraw classification", () => void write("workspace_withdraw_classification", { factId: classification.id })));
        list.append(row);
      }
      status.textContent = rows.length + " recorded classifications.";
    } catch (error) { if (current()) status.textContent = String(error); }
  });
  function describeFact(fact: SemanticFact) {
    return fact.description + " · " + fact.reference.family + " / " + fact.reference.id +
      " · subject " + fact.subject.id + (fact.target ? " · target " + fact.target.id : "") +
      (fact.source ? " · evidence " + fact.source.id : "") +
      (fact.descriptor ? " · descriptor " + fact.descriptor.id : "") +
      " · revision " + fact.recorded_at + " · " + fact.rationale;
  }
  async function resolveFact(reference: FactReference) {
    const { ctx, current } = beginInspection("[data-semantic-detail]");
    try {
      const fact = await invoke<SemanticFact | null>("workspace_semantic_fact", { request: ctx, reference });
      if (!current()) return;
      host.querySelector("[data-semantic-detail]")!.textContent = fact ? describeFact(fact) : "Fact is not active in this historical view.";
    } catch (error) { if (current()) status.textContent = String(error); }
  }
  submit("[data-semantic-fact]", f => void resolveFact({
    ...ref(value(f,"id")), family: value(f,"family") as FactReference["family"],
  }));
  submit("[data-semantic-focus]", async f => {
    const { ctx, current } = beginInspection("[data-semantic-links]", "[data-semantic-detail]");
    const targetId = value(f,"target");
    try {
      const facts = await invoke<SemanticFact[]>("workspace_semantic_fact_backlinks", { request: ctx, targetId });
      if (!current() || targetId !== value(f,"target")) return;
      const list = host.querySelector("[data-semantic-links]")!;
      list.replaceChildren();
      for (const fact of facts) {
        const row = document.createElement("li");
        row.textContent = describeFact(fact) + " ";
        const inspect = document.createElement("button");
        inspect.type = "button"; inspect.textContent = "Resolve " + fact.reference.family + " fact";
        inspect.onclick = () => void resolveFact(fact.reference);
        const annotate = document.createElement("button");
        annotate.type = "button"; annotate.textContent = "Select fact for annotation";
        annotate.onclick = () => {
          const f = form("[data-fact-annotation]");
          (f.elements.namedItem("family") as HTMLSelectElement).value = fact.reference.family;
          const input = f.elements.namedItem("fact") as HTMLInputElement;
          input.value = fact.reference.id;
          input.dispatchEvent(new Event("input", { bubbles: true }));
          (editable ? f.elements.namedItem("claim") as HTMLInputElement : input).focus();
        };
        row.append(inspect, annotate); list.append(row);
      }
      status.textContent = facts.length + " active semantic backlinks.";
    } catch (error) { if (current()) status.textContent = String(error); }
  });
  const annotationForm = form("[data-fact-annotation]");
  const annotationRef = (f: HTMLFormElement): FactReference => ({
    ...ref(value(f,"fact")), family: value(f,"family") as FactReference["family"],
  });
  submit("[data-fact-annotation]", f => {
    if (!value(f,"claim").trim()) { status.textContent = "Select an existing sourced claim."; return; }
    void write("workspace_annotate_semantic_fact", { fact: annotationRef(f), claimId: value(f,"claim") });
  });
  host.querySelector<HTMLButtonElement>("[data-read-annotations]")!.onclick = async () => {
    const { ctx, current } = beginInspection("[data-annotations]");
    const fact = annotationRef(annotationForm);
    try {
      const rows = await invoke<{ id: string; claim: Ref; recorded_at: number }[]>("workspace_semantic_fact_annotations", { request: ctx, fact });
      if (!current() || fact.id !== annotationRef(annotationForm).id || fact.family !== annotationRef(annotationForm).family) return;
      const list = host.querySelector("[data-annotations]")!;
      list.replaceChildren(...rows.map(annotation => {
        const row = document.createElement("li");
        row.textContent = "Claim " + annotation.claim.id + " · annotation " + annotation.id + " · revision " + annotation.recorded_at;
        return row;
      }));
      status.textContent = rows.length + " sourced annotations.";
    } catch (error) { if (current()) status.textContent = String(error); }
  };
  submit("[data-retire-descriptor]", f => void write("workspace_retire_semantic_descriptor", { descriptorId: value(f,"descriptor") }));
  submit("[data-site-area]", f => {
    const amount = Number(value(f,"amount"));
    if (value(f,"status") === "known" && (!value(f,"amount") || !Number.isSafeInteger(amount) || amount < 0)) {
      status.textContent = "Enter a nonnegative whole area amount.";
      return;
    }
    const reading: AreaReading = value(f,"status") === "unknown" ? { status: "unknown" } :
      { status: "known", amount, unit: value(f,"unit") as "hectare" | "square_metre" };
    void write("workspace_record_site_area", { input: {
      place_id: value(f,"place"), descriptor_id: value(f,"descriptor"), source_id: value(f,"source"),
      reading, starts_at: Number(value(f,"start")), ends_before: value(f,"end") ? Number(value(f,"end")) : null,
      corrects: value(f,"corrects").trim() || null,
    } });
  });
  submit("[data-area-focus]", async f => {
    const { ctx, current } = beginInspection("[data-area-result]");
    const placeId = value(f,"place");
    try {
      const area = await invoke<SiteArea | null>("workspace_site_area", { request: ctx, placeId });
      if (!current() || placeId !== value(f,"place")) return;
      const out = host.querySelector("[data-area-result]")!;
      out.replaceChildren();
      if (!area) { out.textContent = "No recorded area at this date and revision."; return; }
      const reading = area.reading.status === "unknown" ? "Explicitly unknown" :
        area.reading.amount + " " + area.reading.unit.replaceAll("_", " ");
      out.textContent = reading + " · fact " + area.id + " · evidence " + area.source_id +
        " · definition revision " + area.descriptor_revision + " · recorded revision " + area.recorded_at + " ";
      out.append(action("Prepare area correction", () => {
        const set = (name: string, value: string) => (areaForm.elements.namedItem(name) as HTMLInputElement).value = value;
        set("place", placeId); set("source", area.source_id); set("descriptor", area.descriptor_id);
        set("start", String(area.starts_at)); set("end", area.ends_before === null ? "" : String(area.ends_before));
        set("corrects", area.id); areaStatus.value = area.reading.status;
        areaAmount.value = area.reading.status === "known" ? String(area.reading.amount) : "";
        if (area.reading.status === "known") areaUnit.value = area.reading.unit;
        areaStatus.dispatchEvent(new Event("change"));
      }));
    } catch (error) { if (current()) status.textContent = String(error); }
  });
  submit("[data-fact]", f => void write("workspace_record_network_fact", { input: {
    subject:ref(value(f,"subject")),target:ref(value(f,"target")),source:ref(value(f,"source")),
    kind:value(f,"kind"),role:value(f,"role"),starts_at:Number(value(f,"start")),
    ends_before:value(f,"end") ? Number(value(f,"end")) : null,
  } }));
  (form("[data-fact]").elements.namedItem("kind") as HTMLSelectElement).addEventListener("change", () => {
    const f = form("[data-fact]");
    const kind = value(f,"kind");
    if (kind === "culture_language" || kind === "working_material") {
      (f.elements.namedItem("role") as HTMLInputElement).value =
        kind === "culture_language" ? "documented_language" : "working_medium";
    }
  });
  function action(label: string, run: () => void) {
    const button = document.createElement("button"); button.type="button";button.textContent=label;
    button.disabled=!editable;button.onclick=run;return button;
  }
  submit("[data-focus]", async f => {
    const { ctx, current } = beginInspection("[data-facts]");
    try {
      const facts=await invoke<Fact[]>("workspace_network_facts",{request:ctx,focus:value(f,"focus")});
      if(!current())return;
      const list=host.querySelector("[data-facts]")!;list.replaceChildren();
      for(const fact of facts){
        const row=document.createElement("li");
        row.textContent=fact.subject_label+" → "+fact.target_label+" · "+fact.input.kind+" · "+fact.input.role+" ";
        row.append(action("Withdraw fact",()=>void write("workspace_withdraw_network_fact",{factId:fact.id})));list.append(row);
      }
      status.textContent=facts.length+" recorded facts.";
    }catch(error){if(current())status.textContent=String(error);}
  });
  host.querySelector<HTMLButtonElement>("[data-impact]")!.onclick = async () => {
    const { ctx, current } = beginInspection("[data-impact-results]");
    try {
      const rows = await invoke<[Ref, string][]>("workspace_persisted_impact", {request:ctx, focus: value(form("[data-focus]"), "focus")});
      if (!current()) return;
      const list = host.querySelector("[data-impact-results]")!;
      list.replaceChildren();
      for (const [object, label] of rows) {
        const row = document.createElement("li"); row.textContent = label + " · " + object.id; list.append(row);
      }
      status.textContent = rows.length + " confirmed downstream events.";
    } catch (error) { if (current()) status.textContent = String(error); }
  };
  host.querySelector<HTMLButtonElement>("[data-candidates]")!.onclick = async () => {
    const { ctx, current } = beginInspection("[data-candidate-results]");
    try {
      const ghostForm = form("[data-ghost]");
      const rows = await invoke<{label:string;start_byte:number;end_byte:number;existing:Ref[]}[]>("workspace_ghost_candidates", {request:ctx,claimId:value(ghostForm,"claim")});
      if(!current())return;
      const list=host.querySelector("[data-candidate-results]")!;list.replaceChildren();
      for(const candidate of rows) {
        const row=document.createElement("li");
        row.textContent=candidate.label+" · "+candidate.existing.length+" possible existing identities ";
        row.append(action("Select claim span",()=>{
          (ghostForm.elements.namedItem("start") as HTMLInputElement).value=String(candidate.start_byte);
          (ghostForm.elements.namedItem("end") as HTMLInputElement).value=String(candidate.end_byte);
        }));
        for(const match of candidate.existing) row.append(" · "+match.id);
        list.append(row);
      }
      status.textContent="Possible names only; no identities or facts have been created.";
    } catch(error){if(current())status.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-resolve-identity]")!.onclick=async()=>{
    const out=host.querySelector<HTMLOutputElement>("[data-identity-path]")!;
    const { ctx, current }=beginInspection("[data-identity-path]");
    try{
      const chain=await invoke<Ref[]>("workspace_canonical_identity",{request:ctx,focus:value(form("[data-focus]"),"focus")});
      if(current())out.textContent=chain.map(x=>x.id).join(" → ");
    }catch(error){if(current())out.textContent=String(error);}
  };
  const succession=form("[data-succession]");
  const enact=host.querySelector<HTMLButtonElement>("[data-enact]")!;
  let approvedTitle="";
  succession.oninput=()=>{approvedTitle="";enact.disabled=true;};
  host.querySelector<HTMLButtonElement>("[data-preview-succession]")!.onclick=async()=>{
    const {ctx,current}=beginInspection("[data-succession-result]");
    approvedTitle="";enact.disabled=true;
    const title=value(succession,"title"),out=host.querySelector<HTMLOutputElement>("[data-succession-result]")!;
    try{
      const result=await invoke<{eligible:boolean;explanation:string;designated:Ref|null}>("workspace_succession_preview",{request:ctx,title});
      if(!current()||title!==value(succession,"title"))return;
      out.textContent=result.explanation+(result.designated?" · "+result.designated.id:"");
      approvedTitle=result.eligible?title:"";enact.disabled=!editable||!result.eligible;
    }catch(error){if(current())out.textContent=String(error);}
  };
  submit("[data-succession]",f=>{if(approvedTitle===value(f,"title")&&!enact.disabled)void write("workspace_enact_succession",{title:approvedTitle,source:value(f,"source")});});
  submit("[data-ghost]", f=>void write("workspace_propose_ghost",{claimId:value(f,"claim"),startByte:Number(value(f,"start")),
    endByte:Number(value(f,"end")),kind:value(f,"kind")}));
  submit("[data-template]", f => void write("workspace_save_custom_template", { sourceId: value(f,"source"), name:value(f,"name"), templateId:value(f,"id") || null }));
  host.querySelector<HTMLButtonElement>("[data-template-list]")!.onclick = async () => {
    const { ctx, current }=beginInspection("[data-templates]");
    try {
      const templates = await invoke<{id:string;name:string;revision:number}[]>("workspace_custom_templates",{request:ctx});
      if(!current())return;
      const list=host.querySelector("[data-templates]")!;list.replaceChildren();
      for(const template of templates){const row=document.createElement("li");row.textContent=template.name+" · "+template.id+" · revision "+template.revision;list.append(row);}
    } catch(error){if(current())status.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-review]")!.onclick=async()=>{
    const {ctx,current}=beginInspection("[data-ghosts]");
    try{
      const ghosts=await invoke<Ghost[]>("workspace_ghost_proposals",{request:ctx});
      if(!current())return;
      const list=host.querySelector("[data-ghosts]")!;list.replaceChildren();
      for(const ghost of ghosts){
        const row=document.createElement("li");row.textContent=ghost.label+" · "+ghost.kind+" · claim "+ghost.claim.id+" ";
        if(ghost.reviewed)row.append(ghost.promoted?"Promoted: "+ghost.promoted.id:"Dismissed");
        else {
          const target=document.createElement("input");
          target.placeholder="Existing matching identity";
          target.setAttribute("aria-label","Existing identity for "+ghost.label);
          target.disabled=!editable;
          row.append(action("Promote draft",()=>void write("workspace_review_ghost",{proposalId:ghost.id,promote:true})),
            action("Dismiss draft",()=>void write("workspace_review_ghost",{proposalId:ghost.id,promote:false})),
            target, action("Resolve to existing",()=>void write("workspace_resolve_ghost",{proposalId:ghost.id,targetId:target.value.trim()})));
        }
        list.append(row);
      }
      status.textContent=ghosts.length+" draft records.";
    }catch(error){if(current())status.textContent=String(error);}
  };
}
