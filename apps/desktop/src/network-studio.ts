import { invoke } from "@tauri-apps/api/core";
type Context = { world_id: string; day: number; revision: number };
type Ref = { world: string; id: string };
type Fact = { id: string; input: { kind: string; role: string }; subject_label: string; target_label: string };
type Ghost = { id: string; claim: Ref; label: string; kind: string; reviewed: boolean; promoted: Ref | null };

export function openNetworkStudio(parent: HTMLElement, request: Context, editable: boolean,
  refresh: () => Promise<void>, valid: () => boolean) {
  parent.querySelector("[data-network-studio]")?.remove();
  const host = document.createElement("details");
  host.dataset.networkStudio = "";
  host.className = "query-panel";
  host.innerHTML = `<summary>Institution, causal and draft identity studio</summary>
    <p class="hint">Author-only. Stable object IDs come from historical search. Source evidence is required for network facts.</p>
    <form data-identity class="editor-form"><fieldset><legend>Create typed identity</legend>
      <label>Identity label <input name="label" required maxlength="800"></label>
      <label>Identity type <select name="kind"><option value="organization">Organization</option><option value="household">Household</option><option value="family_house">Family house</option><option value="faction">Faction</option><option value="conflict">Conflict</option><option value="office">Office</option><option value="title">Hereditary or legal title</option><option value="asset">Asset</option><option value="population">Fictional population</option><option value="conflict_issue">Conflict issue</option><option value="conflict_phase">Conflict phase</option><option value="succession_rule">Succession rule</option><option value="person">Person</option></select></label>
      <button>Create identity</button></fieldset></form>
    <form data-fact class="editor-form"><fieldset><legend>Record sourced network fact</legend>
      <label>Subject identity <input name="subject" required></label><label>Target identity <input name="target" required></label>
      <label>Evidence source identity <input name="source" required></label>
      <label>Fact type <select name="kind"><option value="membership">Membership</option><option value="marriage">Marriage</option><option value="succession">Institutional succession</option><option value="confirmed_cause">Confirmed cause</option><option value="alleged_cause">Alleged cause</option><option value="conflict_position">Faction conflict stance</option><option value="office_holding">Office holding</option><option value="office_of">Office in institution</option><option value="title_holding">Title holding</option><option value="title_of">Title of institution</option><option value="ownership">Ownership</option><option value="issue_of">Issue in conflict</option><option value="phase_of">Phase in conflict</option><option value="rule_of">Succession rule of title</option><option value="population_member">Founder population affiliation</option></select></label>
      <label>Role or causal rationale <input name="role" required maxlength="800"></label>
      <label>Starts at day <input name="start" type="number" required></label><label>Ends before day <input name="end" type="number"></label>
      <button>Record network fact</button></fieldset></form>
    <form data-focus class="editor-form"><label>Network focus identity <input name="focus" required></label><button>Inspect network facts</button><button type="button" data-impact>Explore confirmed impact</button></form><ul data-facts></ul><ul data-impact-results></ul>
    <form data-ghost class="editor-form"><fieldset><legend>Propose identity from a claim</legend>
      <label>Claim identity <input name="claim" required></label>
      <label>Start UTF-8 byte <input name="start" type="number" min="0" required></label>
      <label>End UTF-8 byte (exclusive) <input name="end" type="number" min="1" required></label>
      <label>Proposed type <select name="kind"><option value="person">Person</option><option value="organization">Organization</option><option value="household">Household</option><option value="family_house">Family house</option><option value="faction">Faction</option><option value="conflict">Conflict</option><option value="office">Office</option><option value="title">Hereditary or legal title</option><option value="asset">Asset</option><option value="population">Fictional population</option><option value="conflict_issue">Conflict issue</option><option value="conflict_phase">Conflict phase</option><option value="succession_rule">Succession rule</option></select></label>
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
  host.querySelectorAll("fieldset").forEach(field => field.disabled = !editable);
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
  submit("[data-fact]", f => void write("workspace_record_network_fact", { input: {
    subject:ref(value(f,"subject")),target:ref(value(f,"target")),source:ref(value(f,"source")),
    kind:value(f,"kind"),role:value(f,"role"),starts_at:Number(value(f,"start")),
    ends_before:value(f,"end") ? Number(value(f,"end")) : null,
  } }));
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
