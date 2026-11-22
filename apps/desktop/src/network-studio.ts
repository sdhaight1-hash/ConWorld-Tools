import { invoke } from "@tauri-apps/api/core";
type Context = { world_id: string; branch_id?: string; day: number; revision: number };
const selectedBranches = new Map<string, string>();
type Ref = { world: string; id: string };
type Fact = { id: string; input: { kind: string; role: string }; subject_label: string; target_label: string };
type Classification = { id: string; profile: string; descriptor_id: string; descriptor_revision: number;
  descriptor_branch: string; descriptor_state_revision: number; source_id: string; starts_at: number;
  ends_before: number | null; recorded_at: number };
type FactReference = Ref & { family: "network" | "parentage" | "classification" };
type SemanticFact = { reference: FactReference; subject: Ref; target: Ref | null; source: Ref | null;
  descriptor: Ref | null; description: string; rationale: string; recorded_at: number; source_branch: string };
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
    <form data-semantic-fact class="editor-form"><label>Fact family <select name="family"><option value="network">Network</option><option value="parentage">Parentage</option><option value="classification">Classification</option></select></label>
      <label>Stable fact identity <input name="id" required></label><button>Resolve fact</button></form>
    <p data-semantic-detail role="status"></p>
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
  host.querySelectorAll("fieldset").forEach(field => field.disabled = !editable);
  const selectedLabel=host.querySelector<HTMLElement>("[data-selected-branch]")!;
  selectedLabel.textContent = request.branch_id ? "Alternate branch: "+request.branch_id : "Main history";
  const chooser=host.querySelector<HTMLSelectElement>("[data-branch]")!;
  const choose=(id:string | undefined)=>{
    if(id)selectedBranches.set(request.world_id,id);else selectedBranches.delete(request.world_id);
    if(id)request={...request,branch_id:id};else {const {branch_id:_,...main}=request;request=main;}
    selectedLabel.textContent=id?"Alternate branch: "+id:"Main history";
    for(const selector of ["[data-classifications]","[data-semantic-links]","[data-semantic-detail]","[data-facts]","[data-impact-results]","[data-identity-path]","[data-succession-result]","[data-ghosts]","[data-candidate-results]","[data-templates]"])host.querySelector(selector)?.replaceChildren();
    host.querySelector<HTMLButtonElement>("[data-enact]")!.disabled=true;
  };
  host.querySelector<HTMLButtonElement>("[data-load-branches]")!.onclick=async()=>{
    try {
      const branches=await invoke<{id:string;day:number|null;created_revision:number}[]>("workspace_metadata_branches",{request});
      if(!valid())return;
      chooser.replaceChildren();
      for(const branch of branches)chooser.add(new Option(branch.day===null?"Main history":"Fork day "+branch.day+" · "+branch.id,branch.day===null?"":branch.id));
      chooser.value=request.branch_id??"";chooser.hidden=false;
    }catch(error){if(valid())status.textContent=String(error);}
  };
  chooser.onchange=()=>choose(chooser.value||undefined);
  const forkButton=host.querySelector<HTMLButtonElement>("[data-fork]")!;
  forkButton.disabled=!editable;
  forkButton.onclick=async()=>{
    if(!valid()||busy||!editable)return;
    busy=true;
    try{
      const id=await invoke<string>("workspace_fork_metadata",{request});
      if(valid()){choose(id);await refresh();}
    }catch(error){if(valid())status.textContent=String(error);}
    finally{busy=false;}
  };
  let busy = false;
  async function write(command: string, args: Record<string, unknown>) {
    if (!valid() || busy || !editable) return;
    busy = true;
    try { await invoke(command, { request, ...args }); if (valid()) await refresh(); }
    catch (error) { if (valid()) status.textContent = String(error); }
    finally { busy = false; }
  }
  function submit(selector: string, action: (f: HTMLFormElement) => void) {
    const f = form(selector);
    f.onsubmit = event => { event.preventDefault(); if (valid()) action(f); };
  }
  submit("[data-identity]", f => void write("workspace_create_identity", { kind: value(f,"kind"), label: value(f,"label") }));
  submit("[data-classification]", f => void write("workspace_record_classification", { input: {
    subject_id: value(f,"subject"), descriptor_id: value(f,"descriptor"), source_id: value(f,"source"),
    profile: value(f,"profile"), starts_at: Number(value(f,"start")),
    ends_before: value(f,"end") ? Number(value(f,"end")) : null,
  } }));
  submit("[data-classification-focus]", async f => {
    const ctx = request;
    const subjectId = value(f,"subject");
    try {
      const rows = await invoke<Classification[]>("workspace_classifications", { request: ctx, subjectId });
      if (!valid() || ctx !== request || subjectId !== value(f,"subject")) return;
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
    } catch (error) { if (valid() && ctx === request) status.textContent = String(error); }
  });
  function describeFact(fact: SemanticFact) {
    return fact.description + " · " + fact.reference.family + " / " + fact.reference.id +
      " · subject " + fact.subject.id + (fact.target ? " · target " + fact.target.id : "") +
      (fact.source ? " · evidence " + fact.source.id : "") +
      (fact.descriptor ? " · descriptor " + fact.descriptor.id : "") +
      " · revision " + fact.recorded_at + " · " + fact.rationale;
  }
  async function resolveFact(reference: FactReference) {
    const ctx = request;
    try {
      const fact = await invoke<SemanticFact | null>("workspace_semantic_fact", { request: ctx, reference });
      if (!valid() || ctx !== request) return;
      host.querySelector("[data-semantic-detail]")!.textContent = fact ? describeFact(fact) : "Fact is not active in this historical view.";
    } catch (error) { if (valid() && ctx === request) status.textContent = String(error); }
  }
  submit("[data-semantic-fact]", f => void resolveFact({
    ...ref(value(f,"id")), family: value(f,"family") as FactReference["family"],
  }));
  submit("[data-semantic-focus]", async f => {
    const ctx = request;
    const targetId = value(f,"target");
    try {
      const facts = await invoke<SemanticFact[]>("workspace_semantic_fact_backlinks", { request: ctx, targetId });
      if (!valid() || ctx !== request || targetId !== value(f,"target")) return;
      const list = host.querySelector("[data-semantic-links]")!;
      list.replaceChildren();
      for (const fact of facts) {
        const row = document.createElement("li");
        row.textContent = describeFact(fact) + " ";
        const inspect = document.createElement("button");
        inspect.type = "button"; inspect.textContent = "Resolve " + fact.reference.family + " fact";
        inspect.onclick = () => void resolveFact(fact.reference);
        row.append(inspect); list.append(row);
      }
      status.textContent = facts.length + " active semantic backlinks.";
    } catch (error) { if (valid() && ctx === request) status.textContent = String(error); }
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
    try {
      const facts=await invoke<Fact[]>("workspace_network_facts",{request,focus:value(f,"focus")});
      if(!valid())return;
      const list=host.querySelector("[data-facts]")!;list.replaceChildren();
      for(const fact of facts){
        const row=document.createElement("li");
        row.textContent=fact.subject_label+" → "+fact.target_label+" · "+fact.input.kind+" · "+fact.input.role+" ";
        row.append(action("Withdraw fact",()=>void write("workspace_withdraw_network_fact",{factId:fact.id})));list.append(row);
      }
      status.textContent=facts.length+" recorded facts.";
    }catch(error){if(valid())status.textContent=String(error);}
  });
  host.querySelector<HTMLButtonElement>("[data-impact]")!.onclick = async () => {
    try {
      const rows = await invoke<[Ref, string][]>("workspace_persisted_impact", {request, focus: value(form("[data-focus]"), "focus")});
      if (!valid()) return;
      const list = host.querySelector("[data-impact-results]")!;
      list.replaceChildren();
      for (const [object, label] of rows) {
        const row = document.createElement("li"); row.textContent = label + " · " + object.id; list.append(row);
      }
      status.textContent = rows.length + " confirmed downstream events.";
    } catch (error) { if (valid()) status.textContent = String(error); }
  };
  host.querySelector<HTMLButtonElement>("[data-candidates]")!.onclick = async () => {
    try {
      const ghostForm = form("[data-ghost]");
      const rows = await invoke<{label:string;start_byte:number;end_byte:number;existing:Ref[]}[]>("workspace_ghost_candidates", {request,claimId:value(ghostForm,"claim")});
      if(!valid())return;
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
    } catch(error){if(valid())status.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-resolve-identity]")!.onclick=async()=>{
    const out=host.querySelector<HTMLOutputElement>("[data-identity-path]")!;
    try{
      const ctx=request;
      const chain=await invoke<Ref[]>("workspace_canonical_identity",{request:ctx,focus:value(form("[data-focus]"),"focus")});
      if(valid()&&ctx===request)out.textContent=chain.map(x=>x.id).join(" → ");
    }catch(error){if(valid())out.textContent=String(error);}
  };
  const succession=form("[data-succession]");
  const enact=host.querySelector<HTMLButtonElement>("[data-enact]")!;
  let approvedTitle="";
  succession.oninput=()=>{approvedTitle="";enact.disabled=true;};
  host.querySelector<HTMLButtonElement>("[data-preview-succession]")!.onclick=async()=>{
    const ctx=request,title=value(succession,"title"),out=host.querySelector<HTMLOutputElement>("[data-succession-result]")!;
    try{
      const result=await invoke<{eligible:boolean;explanation:string;designated:Ref|null}>("workspace_succession_preview",{request:ctx,title});
      if(!valid()||ctx!==request||title!==value(succession,"title"))return;
      out.textContent=result.explanation+(result.designated?" · "+result.designated.id:"");
      approvedTitle=result.eligible?title:"";enact.disabled=!editable||!result.eligible;
    }catch(error){if(valid())out.textContent=String(error);}
  };
  submit("[data-succession]",f=>{if(approvedTitle===value(f,"title")&&!enact.disabled)void write("workspace_enact_succession",{title:approvedTitle,source:value(f,"source")});});
  submit("[data-ghost]", f=>void write("workspace_propose_ghost",{claimId:value(f,"claim"),startByte:Number(value(f,"start")),
    endByte:Number(value(f,"end")),kind:value(f,"kind")}));
  submit("[data-template]", f => void write("workspace_save_custom_template", { sourceId: value(f,"source"), name:value(f,"name"), templateId:value(f,"id") || null }));
  host.querySelector<HTMLButtonElement>("[data-template-list]")!.onclick = async () => {
    try {
      const templates = await invoke<{id:string;name:string;revision:number}[]>("workspace_custom_templates",{request});
      if(!valid())return;
      const list=host.querySelector("[data-templates]")!;list.replaceChildren();
      for(const template of templates){const row=document.createElement("li");row.textContent=template.name+" · "+template.id+" · revision "+template.revision;list.append(row);}
    } catch(error){if(valid())status.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-review]")!.onclick=async()=>{
    try{
      const ghosts=await invoke<Ghost[]>("workspace_ghost_proposals",{request});
      if(!valid())return;
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
    }catch(error){if(valid())status.textContent=String(error);}
  };
}
