import { invoke } from "@tauri-apps/api/core";

type FamilyContext = { world_id: string; person_id: string; day: number; revision: number };
type Person = { id: string; label: string };
type FamilyLink = { id: string; parent: {id:string}; kind: string; relative_label: string;
  starts_at: number; ends_before: number | null; rationale: string };

export async function openFamilyHistory(panel: HTMLElement, request: FamilyContext, people: Person[],
  editable: boolean, refresh: () => Promise<void>) {
  const host = document.createElement("section");
  host.innerHTML = `<h3>Family history</h3><ul></ul><details class="add-parent"><summary>Record a parent</summary><form class="editor-form"><fieldset disabled>
    <label>Parent <select name="parent" required></select></label>
    <label>Parentage <select name="kind"><option value="biological">Biological</option>
      <option value="adoptive">Adoptive</option><option value="legal">Legal</option></select></label>
    <label>From day <input type="number" name="start" required></label>
    <label>Before day <input type="number" name="end"></label>
    <label>Basis for the record <input name="rationale" required maxlength="200"></label>
    <button>Record parent</button></fieldset></form></details><p role="status"></p>
    <details class="family-match"><summary>Compare fictional inheritance</summary><form class="editor-form">
    <label>Compare with <select name="other" required></select></label>
    <label>Model seed <input type="number" name="seed" min="0" max="4294967295" value="1" required></label>
    <button>Compare segments</button></form><output></output></details>`;
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
  let busy = false;
  async function save(command: string, payload: Record<string, unknown>) {
    if (busy || !editable) return;
    busy = true;
    host.querySelectorAll<HTMLButtonElement>("button").forEach(b => b.disabled = true);
    try {
      await invoke(command, { request, ...payload });
      if (host.isConnected) await refresh();
    } catch (error) {
      status.textContent = String(error);
      host.querySelectorAll<HTMLButtonElement>("button").forEach(b => b.disabled = false);
    } finally { busy = false; }
  }
  try {
    const links = await invoke<FamilyLink[]>("workspace_family_links", { request });
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
    fields.disabled = !editable || !parent.length;
    host.querySelector<HTMLElement>(".add-parent")!.hidden = !editable;
  } catch (error) { status.textContent = String(error); }
  form.onsubmit = event => {
    event.preventDefault();
    if (fields.disabled) return;
    const data = new FormData(form);
    void save("workspace_record_parentage", {input: {parent_id: parent.value, kind: data.get("kind"),
      starts_at:Number(data.get("start")), ends_before:data.get("end") ? Number(data.get("end")) : null,
      rationale:data.get("rationale")}});
  };
}
