import { canLeaveEntryDrafts } from "./entry-drafts";
import { appendReferencePicker } from "./reference-picker";
import { invoke } from "@tauri-apps/api/core";

type FamilyContext = { world_id: string; person_id: string; day: number; revision: number };
type Person = { id: string; label: string };
type FamilyLink = { id: string; parent: {id:string}; kind: string; relative_label: string;
  starts_at: number; ends_before: number | null; rationale: string };

export async function openFamilyHistory(panel: HTMLElement, request: FamilyContext, people: Person[],
  editable: boolean, refresh: () => Promise<void>) {
  const host = document.createElement("section");
  host.innerHTML = `<h3>Family history</h3><ul></ul><details class="add-parent"><summary>Record a parent</summary><form class="editor-form"><fieldset disabled>
    <label>Parent <select name="parent" required></select></label><div data-parent-picker></div>
    <label>Parentage <select name="kind"><option value="biological">Biological</option>
      <option value="adoptive">Adoptive</option><option value="legal">Legal</option></select></label>
    <label>From day <input type="number" name="start" required></label>
    <label>Before day <input type="number" name="end"></label>
    <label>Basis for the record <input name="rationale" required maxlength="200"></label>
    <button>Record parent</button></fieldset></form></details><p role="status" data-draft-status tabindex="-1"></p>
    <details class="family-match"><summary>Compare fictional inheritance</summary><form class="editor-form">
    <label>Compare with <select name="other" required></select></label><div data-relative-picker></div>
    <label>Model seed <input type="number" name="seed" min="0" max="4294967295" value="1" required></label>
    <button>Compare segments</button></form><output></output></details>
    <details class="founder-genome"><summary>Simulated multigenerational ancestry</summary><p>Dimensionless toy founder segments, not real DNA. Unrecorded parents are unresolved founders.</p><button type="button">Compute founder segments</button><button type="button" data-compare-founders>Compare selected relative</button><button type="button" data-painter>Paint eight toy chromosomes</button><div data-genome></div></details>`;
  panel.append(host);
  const form = host.querySelector("form")!;
  const fields = form.querySelector("fieldset")!;
  const status = host.querySelector<HTMLElement>('[role=status]')!;
  const parent = form.elements.namedItem("parent") as HTMLSelectElement;
  for (const person of people.filter(p => p.id !== request.person_id)) {
    parent.add(new Option(person.label + " · " + person.id, person.id));
  }
  (form.elements.namedItem("start") as HTMLInputElement).value = String(request.day);
  const compare = host.querySelector<HTMLFormElement>(".family-match form")!;
  const other = compare.elements.namedItem("other") as HTMLSelectElement;
  for (const option of Array.from(parent.options)) other.add(new Option(option.text, option.value));
  const compatible = (candidate: { id: string; kind?: string }) =>
    candidate.kind === "person" && candidate.id !== request.person_id;
  const lookup = (selector: string, select: HTMLSelectElement, label: string) => {
    const container = host.querySelector<HTMLElement>(selector)!;
    const identity = document.createElement("input");
    identity.type = "hidden";
    container.append(identity);
    appendReferencePicker(container, identity,
      { world_id: request.world_id, day: request.day, revision: request.revision },
      "entry", label, candidate => {
        if (!Array.from(select.options).some(option => option.value === candidate.id)) {
          select.add(new Option(candidate.label + " · " + candidate.id, candidate.id));
        }
        select.value = candidate.id;
        select.focus();
      }, compatible);
  };
  // Keep the existing nearby options, but no longer require a complete graph
  // or put every person in a select before a family record can be authored.
  lookup("[data-parent-picker]", parent, "parent identity");
  lookup("[data-relative-picker]", other, "relative identity");
  const output = host.querySelector("output")!;
  compare.onsubmit = async event => {
    event.preventDefault();
    const submit = compare.querySelector("button")!;
    submit.disabled = true;
    try {
      const segments = await invoke<{start:number;end:number;contributor:{id:string}}[]>("workspace_family_match", {
        request, otherId:other.value, seed:Number(new FormData(compare).get("seed"))
      });
      output.textContent = segments.length ? segments.map(segment => {
        const contributor = people.find(p => p.id === segment.contributor.id);
        return `${contributor?.label ?? segment.contributor.id}: [${segment.start}, ${segment.end})`;
      }).join(" · ") : "No shared contributor intervals for this seed.";
    } catch (error) { output.textContent = String(error); }
    finally { submit.disabled = false; }
  };
  host.querySelector<HTMLButtonElement>(".founder-genome button")!.onclick = async () => {
    const result=host.querySelector<HTMLElement>("[data-genome]")!;
    try {
      const genome=await invoke<{model:number;homologues:{start:number;end:number;allele:{founder:{id:string};homologue:number;unresolved_parent:boolean}}[][]}>("workspace_founder_genome",{request,seed:Number(new FormData(compare).get("seed"))});
      if(!host.isConnected)return;
      result.replaceChildren();
      for(const [index,segments] of genome.homologues.entries()){
        const heading=document.createElement("h4");heading.textContent="Toy homologue "+(index+1)+" · model "+genome.model;result.append(heading);
        const list=document.createElement("ul");
        for(const segment of segments){
          const row=document.createElement("li"),person=people.find(p=>p.id===segment.allele.founder.id);
          row.textContent=(person?.label??segment.allele.founder.id)+" / "+segment.allele.homologue+" · ["+segment.start+", "+segment.end+")";
          list.append(row);
        }
        result.append(list);
      }
    }catch(error){result.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-compare-founders]")!.onclick=async()=>{
    const result=host.querySelector<HTMLElement>("[data-genome]")!;
    try{
      const matches=await invoke<{first_homologue:number;second_homologue:number;segment:{start:number;end:number;allele:{founder:{id:string}}}}[]>("workspace_founder_matches",{request,otherId:other.value,seed:Number(new FormData(compare).get("seed"))});
      if(!host.isConnected)return;
      result.replaceChildren();
      const list=document.createElement("ul");
      for(const match of matches){
        const row=document.createElement("li");
        row.textContent="Homologues "+(match.first_homologue+1)+"/"+(match.second_homologue+1)+" · "+match.segment.allele.founder.id+" · ["+match.segment.start+", "+match.segment.end+")";
        list.append(row);
      }
      result.append(matches.length ? list : "No shared founder alleles in this toy model.");
    }catch(error){result.textContent=String(error);}
  };
  host.querySelector<HTMLButtonElement>("[data-painter]")!.onclick=async()=>{
    const out=host.querySelector<HTMLElement>("[data-genome]")!;
    try{
      const painter=await invoke<{model:number;length:number;chromosomes:{chromosome:number;homologues:{start:number;end:number;allele:{founder:{id:string};homologue:number}}[][]}[]}>("workspace_chromosome_painter",{request,seed:Number(new FormData(compare).get("seed")),count:8});
      if(!host.isConnected)return;
      out.replaceChildren();
      const heading=document.createElement("h4");heading.textContent="Toy chromosome painter · model "+painter.model;out.append(heading);
      const legend=new Map<string,string>();
      for(const chromosome of painter.chromosomes){
        const label=document.createElement("p");label.textContent="Chromosome "+chromosome.chromosome;out.append(label);
        for(const [index,segments] of chromosome.homologues.entries()){
          const row=document.createElement("div");row.className="chromosome-track";row.setAttribute("role","group");row.setAttribute("aria-label","Chromosome "+chromosome.chromosome+", homologue "+(index+1));
          for(const segment of segments){
            const key=segment.allele.founder.id+":"+segment.allele.homologue;
            if(!legend.has(key)){const label=people.find(person=>person.id===segment.allele.founder.id)?.label??segment.allele.founder.id;legend.set(key,label+" · "+segment.allele.homologue);}
            const hash=Array.from(key).reduce((acc,char)=>((acc*31+char.charCodeAt(0))>>>0),0);
            const part=document.createElement("span");part.className="chromosome-segment";
            part.style.flexBasis=(100*(segment.end-segment.start)/painter.length)+"%";part.style.backgroundColor="hsl("+(hash%360)+" 55% 48%)";
            part.title=legend.get(key)+" ["+segment.start+", "+segment.end+")";part.setAttribute("role","img");part.setAttribute("aria-label",part.title);row.append(part);
          }out.append(row);
        }
      }
      const key=document.createElement("ul");for(const [id,label] of legend){const item=document.createElement("li");const swatch=document.createElement("span");const hash=Array.from(id).reduce((acc,char)=>((acc*31+char.charCodeAt(0))>>>0),0);swatch.className="chromosome-key";swatch.style.backgroundColor="hsl("+(hash%360)+" 55% 48%)";item.append(swatch,label);key.append(item);}out.append(key);
    }catch(error){out.textContent=String(error);}
  };
  let busy = false;
  async function save(command: string, payload: Record<string, unknown>) {
    if (busy || !editable || !host.isConnected || !canLeaveEntryDrafts()) return;
    busy = true;
    const controls = [...host.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("input,select,button")];
    const states = controls.map(control => control.disabled);
    controls.forEach(control => control.disabled = true);
    host.dataset.entryDraftState = "saving";
    status.textContent = "Saving family record…";
    try {
      await invoke(command, { request, ...payload });
    } catch (error) {
      delete host.dataset.entryDraftState;
      status.textContent = String(error);
      controls.forEach((control, index) => control.disabled = states[index]);
      busy = false;
      return;
    }
    delete host.dataset.entryDraftState;
    status.textContent = "Family record saved.";
    if (host.isConnected) {
      try { await refresh(); }
      catch (error) { status.textContent = "Family record saved, but refresh failed: " + String(error); }
    }
  }
  try {
    const links = await invoke<FamilyLink[]>("workspace_family_links", { request });
    if (!host.isConnected) return;
    for (const link of links) {
      const row = document.createElement("li");
      const role = link.parent.id === request.person_id ? "child" : "parent";
      row.textContent = `${link.relative_label} · ${link.kind} ${role} · from day ${link.starts_at}` +
        (link.ends_before === null ? "" : ` until day ${link.ends_before} (exclusive)`) + ` · ${link.rationale} `;
      if (editable) {
        const withdrawal = document.createElement("form");
        const reason = document.createElement("input");
        reason.required = true;
        reason.maxLength = 200;
        reason.setAttribute("aria-label", "Reason to withdraw " + link.relative_label);
        reason.placeholder = "Reason for withdrawal";
        const button = document.createElement("button");
        button.textContent = "Withdraw record";
        withdrawal.append(reason, button);
        withdrawal.onsubmit = event => {
          event.preventDefault();
          void save("workspace_withdraw_parentage", {linkId: link.id, rationale: reason.value});
        };
        const correction = document.createElement("details");
        const summary = document.createElement("summary");
        summary.textContent = "Correct this record";
        correction.append(summary, withdrawal);
        row.append(correction);
      }
      host.querySelector("ul")!.append(row);
    }
    if (!links.length) status.textContent = "No accepted family links at this date.";
    fields.disabled = !editable;
    host.querySelector<HTMLElement>(".add-parent")!.hidden = !editable;
  } catch (error) { status.textContent = String(error); }
  form.onsubmit = event => {
    event.preventDefault();
    if (fields.disabled || !parent.value) return;
    const data = new FormData(form);
    void save("workspace_record_parentage", {input: {parent_id: parent.value, kind: data.get("kind"),
      starts_at:Number(data.get("start")), ends_before:data.get("end") ? Number(data.get("end")) : null,
      rationale:data.get("rationale")}});
  };
}
