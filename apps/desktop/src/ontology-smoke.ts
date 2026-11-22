// Explicit test mode. All submissions go through the real author studio and native IPC.
import { invoke } from "@tauri-apps/api/core";

type World = { id: string; revision: number };
type Ref = { world: string; id: string };
type Area = { id: string; reading: { status: string; amount?: number; unit?: string }; corrects: string | null };
type Fact = { input: { kind: string; subject: Ref; target: Ref } };
type Result = { rows: { id: string }[] };

const object = (n: number) => "00000000-0000-0000-0000-" + n.toString(16).padStart(12, "0");
const source = object(1);
const descriptor = object(2);
const driveTag = object(3);
const areaTag = object(4);

function assert(condition: unknown, label: string): asserts condition {
  if (!condition) throw new Error(label);
}

async function world(): Promise<World> {
  const worlds = await invoke<World[]>("workspace_worlds");
  assert(worlds.length === 1, "expected one isolated smoke world");
  return worlds[0];
}

function form(selector: string, fields: Record<string, string>) {
  const studio = document.querySelector<HTMLDetailsElement>("[data-network-studio]");
  assert(studio, "missing native author studio");
  studio.open = true;
  const form = studio.querySelector<HTMLFormElement>(selector);
  assert(form, "missing author form " + selector);
  for (const [name, value] of Object.entries(fields)) {
    const field = form.elements.namedItem(name) as HTMLInputElement | HTMLSelectElement | null;
    assert(field, "missing field " + name);
    field.value = value;
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }
  assert(form.checkValidity(), "incomplete author form " + selector);
  form.requestSubmit();
}

async function waitFor(test: () => boolean, label: string) {
  const deadline = performance.now() + 6000;
  while (!test()) {
    if (performance.now() > deadline) {
      throw new Error(label + ": " + document.querySelector("[data-network-studio] [data-status]")?.textContent);
    }
    await new Promise<void>(resolve => setTimeout(resolve, 25));
  }
}

export type OntologyObservation = { passed: boolean; tasks: { name: string; elapsed_ms: number; submissions: number }[]; error: string | null };
export async function verifyOntologySmoke(): Promise<OntologyObservation> {
  const tasks: OntologyObservation["tasks"] = [];
  try {
    let current = await world();
    let submissions = 0;
    let started = performance.now(), previousSubmissions = 0;
    const finishTask = (name: string) => {
      tasks.push({ name, elapsed_ms: performance.now() - started, submissions: submissions - previousSubmissions });
      started = performance.now(); previousSubmissions = submissions;
    };
    const request = (revision = current.revision, day = 0) => ({ world_id: current.id, revision, day });
    const write = async (selector: string, fields: Record<string, string>) => {
      const next = current.revision + 1;
      form(selector, fields);
      await waitFor(() => document.querySelector("#author-world-info")?.textContent?.includes("Author revision " + next + " / " + next) === true,
        "native write did not commit");
      current = await world();
      assert(current.revision === next, "unexpected author revision");
      submissions++;
    };
    const classify = (subject: number, profile: string, tag: string) => write("[data-classification]", {
      subject: object(subject), profile, descriptor: tag, source, start: "0", end: "",
    });
    const relation = (subject: number, target: number, kind: string, role: string) => write("[data-fact]", {
      subject: object(subject), target: object(target), source, kind, role, start: "0", end: "",
    });
    const query = (predicate: unknown, revision = current.revision, day = 0) => invoke<Result>("workspace_composite_query", {
      requestId: crypto.randomUUID(), request: { ...request(revision, day), predicate, limit: 20 },
    });
    // Exercise the actual compatible, branch-aware selector, not a mocked reply.
    const studio = document.querySelector<HTMLDetailsElement>("[data-network-studio]")!;
    const picker = studio.querySelector<HTMLFormElement>("[data-semantic-picker]")!;
    const destination = picker.elements.namedItem("destination") as HTMLSelectElement;
    const choice = Array.from(destination.options).find(option => option.text === "Classification inspector");
    assert(choice, "classification inspector selection missing");
    form("[data-semantic-picker]", { destination: choice.value, phrase: "Estuary speech" });
    await waitFor(() => picker.querySelector("[data-picker-results] button") !== null, "native compatible selection did not render");
    const candidate = picker.querySelector<HTMLButtonElement>("[data-picker-results] button")!;
    assert(candidate.textContent?.includes(object(11)), "native compatible identity mismatch");
    candidate.focus();
    candidate.click();
    assert(document.activeElement === studio.querySelector('[data-classification-focus] input[name="subject"]'),
      "compatible selection did not focus the original field");
    assert((document.activeElement as HTMLInputElement).value === object(11), "identity selection lost stable ID");
    await classify(10, "culture", descriptor);
    await classify(11, "language", descriptor);
    await classify(12, "material", descriptor);
    await relation(10, 11, "culture_language", "documented_language");
    const language = await invoke<Fact[]>("workspace_network_facts", { request: request(), focus: object(10) });
    assert(language.some(f => f.input.kind === "culture_language" && f.input.target.id === object(11)),
      "culture/language stable relation missing");
    assert((await query({ op: "classified_as", profile: "language" })).rows[0]?.id === object(11),
      "language search mismatch");
    finishTask("culture_language");

    await classify(13, "technology", driveTag);
    await classify(14, "technology", driveTag);
    await relation(13, 12, "working_material", "working_medium");
    await relation(14, 12, "working_material", "working_medium");
    const drives = await query({ op: "classified_by", descriptor_id: driveTag });
    assert(drives.rows.length === 2 && drives.rows.every(row => [object(13), object(14)].includes(row.id)),
      "reusable setting-specific descriptor mismatch");
    const material = await invoke<Fact[]>("workspace_network_facts", { request: request(), focus: object(12) });
    assert(material.filter(f => f.input.kind === "working_material").length === 2,
      "two drives did not share the original material identity");
    finishTask("novel_category_shared_material");

    const area = (revision = current.revision, day = 0) => invoke<Area | null>("workspace_site_area",
      { request: request(revision, day), placeId: object(15) });
    assert(await area() === null, "unrecorded area must be missing");
    const areaFields = { place: object(15), descriptor: areaTag, source, start: "0", end: "20", corrects: "" };
    await write("[data-site-area]", { ...areaFields, status: "known", amount: "0", unit: "hectare" });
    const zeroRevision = current.revision;
    const zero = await area();
    assert(zero?.reading.status === "known" && zero.reading.amount === 0, "known zero was lost");
    await write("[data-site-area]", { ...areaFields, status: "known", amount: "120000", unit: "square_metre", corrects: zero.id });
    const corrected = await area();
    assert(corrected?.corrects === zero.id && corrected.reading.unit === "square_metre" &&
      corrected.reading.amount === 120000, "area correction lost source history");
    assert((await area(zeroRevision))?.reading.amount === 0, "historical zero no longer resolves");
    await write("[data-site-area]", { ...areaFields, start: "20", end: "", status: "unknown" });
    assert((await area(current.revision, 20))?.reading.status === "unknown", "explicit unknown was lost");
    const unknown = await query({ op: "site_area", test: { test: "unknown" } }, current.revision, 20);
    assert(unknown.rows.length === 1 && unknown.rows[0].id === object(15), "unknown search mismatch");
    form("[data-area-focus]", { place: object(15) });
    await waitFor(() => document.querySelector("[data-area-result]")?.textContent?.includes("120000 square metre") === true,
      "native area inspector did not render");
    form("[data-classification-focus]", { subject: object(11) });
    await waitFor(() => document.querySelector("[data-classifications]")?.textContent?.includes("language") === true,
      "native language inspector did not render");
    form("[data-semantic-focus]", { target: source });
    await waitFor(() => document.querySelector("[data-semantic-links]")?.textContent?.includes("Site area") === true,
      "native source backlink inspector did not render");
    finishTask("site_area_history_and_evidence");
    console.info("Ontology native smoke passed", { submissions, revision: current.revision, tasks });
    document.querySelector<HTMLElement>("#host")!.textContent = "Native ontology smoke passed: " + submissions + " authored submissions";
    // Put the verified native result in view before asking the launcher to capture.
    document.querySelector("[data-semantic-focus]")?.scrollIntoView({ block: "start" });
    await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    return { passed: true, tasks, error: null };
  } catch (error) {
    const message = String(error);
    console.error(message);
    document.querySelector<HTMLElement>("#host")!.textContent = "Native ontology smoke failed: " + message;
    return { passed: false, tasks, error: message };
  }
}
