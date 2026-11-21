import { invoke, isTauri } from "@tauri-apps/api/core";
import northbridge from "./northbridge.json";
import "./style.css";
import { benchmarkHeight, benchmarkTriangles, benchmarkWidth, createGpuGraph, makeBenchmark, type GpuGraph } from "./graph-benchmark";

export type Scene = typeof northbridge;
type PlatformScene = { target: string; smoke: boolean; scene: Scene };
type BondSnapshot = { subject: string; target: string; kind: string; trust: number | null; tags: string[] };
type EventSnapshot = { event: string; actor: string; witness: string; action: string; intent: string; outcome: string; source: string };
type HistorySnapshot = { day: number; relationships: BondSnapshot[]; cause: EventSnapshot | null };
type WebGpuStatus = "available" | "api_unavailable" | "adapter_unavailable" | "probe_failed" | "timed_out";

const webGpuLabels: Record<WebGpuStatus, string> = {
  available: "WebGPU adapter detected",
  api_unavailable: "WebGPU unavailable",
  adapter_unavailable: "WebGPU adapter unavailable",
  probe_failed: "WebGPU adapter probe failed",
  timed_out: "WebGPU adapter probe timed out",
};

const host = document.querySelector<HTMLElement>("#host")!;
const renderer = document.querySelector<HTMLElement>("#renderer")!;

async function start() {
  const response = isTauri()
    ? await invoke<PlatformScene>("platform_scene")
    : { target: "browser preview", smoke: false, scene: northbridge };
  host.textContent = `Local scene · ${response.target}`;
  document.querySelector<HTMLElement>("#workspace")!.hidden = false;
  const benchmark = !response.smoke && new URLSearchParams(location.search).has("benchmark");
  if (benchmark) {
    document.body.classList.add("benchmark");
    document.querySelector("h1")!.textContent = "Renderer benchmark";
    document.querySelector(".intro")!.textContent = "Inspect, zoom, and pan the 2,000-node fixture.";
    const link = document.querySelector<HTMLAnchorElement>("#benchmark-link")!;
    link.textContent = "Back to Northbridge";
    link.href = location.pathname;
  }
  const graph = showScene(benchmark ? makeBenchmark() : response.scene, benchmark);
  const historyVerified = await showHistory(response.smoke);
  if (!response.smoke) await showAuthorWorkspace();
  const webgpu = await probeWebGpu();
  if (graph) renderer.textContent = `Canvas2D · ${webGpuLabels[webgpu]}`;
  if (benchmark && graph && webgpu === "available") void graph.tryGpu();
  if (response.smoke) {
    const inspector = document.querySelector<HTMLButtonElement>("#nodes button[data-node='beatrice']");
    inspector?.click();
    const sceneVerified =
      document.querySelectorAll("#nodes button").length === 5 &&
      inspector?.getAttribute("aria-pressed") === "true" &&
      [...document.querySelectorAll("#connections li")].some(
        (item) => item.textContent === "Beatrice Thorne · Member of · Merchant Guild",
      );
    const canvas2dVerified = graph?.visibleNode("beatrice") ?? false;
    const observation = { scene_verified: sceneVerified, canvas2d_verified: canvas2dVerified, history_verified: historyVerified, webgpu };
    await invoke("complete_platform_smoke", { observation });
  }
}

async function showHistory(smoke: boolean): Promise<boolean> {
  if (!isTauri()) return false;
  const section = document.querySelector<HTMLElement>("#history")!;
  const select = document.querySelector<HTMLSelectElement>("#history-year")!;
  const relations = document.querySelector<HTMLElement>("#history-relations")!;
  const cause = document.querySelector<HTMLElement>("#history-cause")!;
  section.hidden = false;
  async function refresh(): Promise<boolean> {
    try {
      const snapshot = await invoke<HistorySnapshot>("northbridge_history", { day: Number(select.value) });
      relations.replaceChildren(...snapshot.relationships.map((bond) => {
        const card = document.createElement("article");
        const heading = document.createElement("h3");
        const state = document.createElement("p");
        const tags = document.createElement("p");
        heading.textContent = `${bond.subject} → ${bond.target}`;
        state.textContent = `${bond.kind} · trust: ${bond.trust ?? "not scored"}`;
        tags.textContent = `Tags: ${bond.tags.join(", ") || "none"}`;
        card.append(heading, state, tags);
        return card;
      }));
      cause.textContent = snapshot.cause
        ? `${snapshot.cause.event}: ${snapshot.cause.actor} (${snapshot.cause.action}; ${snapshot.cause.outcome}). Intent: ${snapshot.cause.intent}. Witness: ${snapshot.cause.witness}. Source: ${snapshot.cause.source}.`
        : "No documented relationship-changing event at this date.";
      return snapshot.relationships.length >= 2;
    } catch (error) {
      cause.textContent = `History unavailable: ${String(error)}`;
      return false;
    }
  }
  select.addEventListener("change", () => void refresh());
  const original = await refresh();
  if (!original) return false;
  // The platform smoke also verifies the event-attributed history path.
  if (smoke) {
    select.value = "3652";
    const changed = await refresh();
    if (!changed || !cause.textContent?.includes("Northbridge folio 14")) return false;
    select.value = "0";
    await refresh();
  }
  return true;
}


type AuthorWorld = { id: string; name: string; revision: number };
type AuthorPerson = { id: string; label: string };
type AuthorCause = {
  day: number; event: string; actor: string | null;
  action: string | null; intent: string | null; outcome: string | null; source: string;
};
type AuthorRelationship = {
  id: string; subject_id: string; target_id: string; kind: string;
  trust: number | null; tags: string[]; starts_at: number | null;
  cause: AuthorCause | null;
};
type AuthorSnapshot = {
  world: AuthorWorld; day: number; people: AuthorPerson[]; relationships: AuthorRelationship[];
};

// Phase Two editor calls native Rust and keeps no independent world database.
async function showAuthorWorkspace() {
  if (!isTauri()) return;
  const section = document.querySelector<HTMLElement>("#author-workspace")!;
  const worldSelect = document.querySelector<HTMLSelectElement>("#author-world")!;
  const dayInput = document.querySelector<HTMLInputElement>("#author-day")!;
  const content = document.querySelector<HTMLElement>("#author-world-content")!;
  const info = document.querySelector<HTMLElement>("#author-world-info")!;
  const status = document.querySelector<HTMLElement>("#author-status")!;
  const peopleList = document.querySelector<HTMLUListElement>("#author-people")!;
  const relationshipList = document.querySelector<HTMLUListElement>("#author-relationships")!;
  const worldForm = document.querySelector<HTMLFormElement>("#author-create-world")!;
  const personForm = document.querySelector<HTMLFormElement>("#author-create-person")!;
  const relationshipForm = document.querySelector<HTMLFormElement>("#author-create-relationship")!;
  const eventForm = document.querySelector<HTMLFormElement>("#author-record-event")!;
  section.hidden = false;
  let requestSequence = 0;
  const value = (form: HTMLFormElement, name: string) =>
    (form.elements.namedItem(name) as HTMLInputElement | HTMLSelectElement).value;
  const score = (raw: string): number | null => raw.trim() === "" ? null : Number(raw);
  const currentDay = () => {
    const raw = dayInput.value.trim();
    if (!/^-?\d+$/.test(raw)) throw new Error("Enter a whole-number historical day.");
    const day = Number(raw);
    if (!Number.isSafeInteger(day)) throw new Error("Historical day is outside the supported range.");
    return day;
  };
  function options(select: HTMLSelectElement, values: Array<{ id: string; label: string }>) {
    const previous = select.value;
    select.replaceChildren(...values.map(({ id, label }) => {
      const option = document.createElement("option");
      option.value = id;
      option.textContent = label;
      return option;
    }));
    if (values.some((item) => item.id === previous)) select.value = previous;
  }
  async function refresh() {
    const sequence = ++requestSequence;
    const worldId = worldSelect.value;
    content.hidden = !worldId;
    if (!worldId) return;
    const snapshot = await invoke<AuthorSnapshot>("workspace_snapshot", {
      worldId, day: currentDay(),
    });
    if (sequence !== requestSequence) return;
    info.textContent = snapshot.world.name + " · revision " + snapshot.world.revision + " · day " + snapshot.day;
    const people = new Map(snapshot.people.map((person) => [person.id, person.label]));
    const personOptions = snapshot.people.map((person) => ({ id: person.id, label: person.label }));
    for (const form of [relationshipForm, eventForm]) {
      for (const field of ["subject", "target", "actor"]) {
        const select = form.elements.namedItem(field);
        if (select instanceof HTMLSelectElement) options(select, personOptions);
      }
    }
    const subjectSelection = relationshipForm.elements.namedItem("subject") as HTMLSelectElement;
    const targetSelection = relationshipForm.elements.namedItem("target") as HTMLSelectElement;
    if (personOptions.length > 1 && subjectSelection.value === targetSelection.value) {
      targetSelection.value = personOptions.find((item) => item.id !== subjectSelection.value)!.id;
    }
    options(eventForm.elements.namedItem("relationship") as HTMLSelectElement,
      snapshot.relationships.map((bond) => ({
        id: bond.id,
        label: (people.get(bond.subject_id) ?? bond.subject_id) + " → " +
          (people.get(bond.target_id) ?? bond.target_id) + " (" + bond.kind + ")",
      })));
    peopleList.replaceChildren(...snapshot.people.map((person) => {
      const item = document.createElement("li");
      item.textContent = person.label;
      return item;
    }));
    relationshipList.replaceChildren(...snapshot.relationships.map((bond) => {
      const item = document.createElement("li");
      const description = document.createElement("p");
      description.textContent = (people.get(bond.subject_id) ?? bond.subject_id) + " → " +
        (people.get(bond.target_id) ?? bond.target_id) + " · " + bond.kind +
        " · trust " + (bond.trust ?? "unscored") + " · " +
        (bond.tags.join(", ") || "no tags") + " · since day " + (bond.starts_at ?? "unknown");
      item.append(description);
      if (bond.cause) {
        const evidence = document.createElement("p");
        evidence.className = "hint";
        evidence.textContent = "Why it changed (day " + bond.cause.day + "): " +
          bond.cause.event + (bond.cause.actor ? " · " + bond.cause.actor : "") +
          (bond.cause.action ? " · " + bond.cause.action : "") +
          (bond.cause.outcome ? " (" + bond.cause.outcome + ")" : "") +
          (bond.cause.intent ? " · intent: " + bond.cause.intent : "") +
          " · source: " + bond.cause.source;
        item.append(evidence);
      }
      return item;
    }));
    relationshipForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = snapshot.people.length < 2;
    eventForm.querySelector<HTMLButtonElement>('button[type="submit"]')!.disabled = snapshot.people.length === 0 || snapshot.relationships.length === 0;
    status.textContent = "World loaded. Edits are saved to the local SQLite database.";
  }
  async function refreshWorlds(preferred?: string) {
    const worlds = await invoke<AuthorWorld[]>("workspace_worlds");
    options(worldSelect, worlds.map((world) => ({ id: world.id, label: world.name })));
    if (preferred && worlds.some((world) => world.id === preferred)) worldSelect.value = preferred;
    await refresh();
  }
  function bind(form: HTMLFormElement, action: () => Promise<void>) {
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      void (async () => {
        const button = form.querySelector<HTMLButtonElement>('button[type="submit"]')!;
        button.disabled = true;
        try {
          await action();
          await refresh();
          form.querySelectorAll<HTMLInputElement>('input[name="name"], input[name="event"], input[name="citation"]').forEach((input) => { input.value = ""; });
          status.textContent = "Saved to local world history.";
        } catch (error) {
          status.textContent = "Unable to save: " + String(error);
        } finally {
          button.disabled = form === relationshipForm
            ? (relationshipForm.elements.namedItem("subject") as HTMLSelectElement).options.length < 2
            : form === eventForm
              ? (eventForm.elements.namedItem("relationship") as HTMLSelectElement).options.length === 0
              : false;
        }
      })();
    });
  }
  bind(worldForm, async () => {
    const world = await invoke<AuthorWorld>("workspace_create_world", { name: value(worldForm, "name") });
    await refreshWorlds(world.id);
  });
  bind(personForm, async () => {
    await invoke("workspace_create_person", { worldId: worldSelect.value, name: value(personForm, "name") });
  });
  bind(relationshipForm, async () => {
    await invoke("workspace_create_relationship", { request: {
      world_id: worldSelect.value,
      subject_id: value(relationshipForm, "subject"),
      target_id: value(relationshipForm, "target"),
      kind: value(relationshipForm, "kind"),
      trust: score(value(relationshipForm, "trust")),
      tags: value(relationshipForm, "tags"),
      day: currentDay(),
    } });
  });
  bind(eventForm, async () => {
    await invoke("workspace_record_event", { request: {
      world_id: worldSelect.value,
      relationship_id: value(eventForm, "relationship"),
      actor_id: value(eventForm, "actor"),
      day: currentDay(),
      event_label: value(eventForm, "event"),
      action: value(eventForm, "action"),
      intent: value(eventForm, "intent"),
      outcome: value(eventForm, "outcome"),
      source_citation: value(eventForm, "citation"),
      trust: score(value(eventForm, "trust")),
      tags: value(eventForm, "tags"),
    } });
  });
  worldSelect.addEventListener("change", () => { void refresh().catch((error) => { status.textContent = String(error); }); });
  dayInput.addEventListener("change", () => { void refresh().catch((error) => { status.textContent = String(error); }); });
  try {
    await refreshWorlds();
  } catch (error) {
    status.textContent = "Author workspace unavailable: " + String(error);
  }
}

function showScene(scene: Scene, benchmark: boolean) {
  const width = benchmark ? benchmarkWidth : 680;
  const height = benchmark ? benchmarkHeight : 475;
  const plot = document.querySelector<HTMLElement>("#plot")!;
  const list = document.querySelector<HTMLUListElement>("#nodes")!;
  const connections = document.querySelector<HTMLUListElement>("#connections")!;
  const selection = document.querySelector<HTMLElement>("#selection")!;
  const canvas = document.querySelector<HTMLCanvasElement>("#graph")!;
  const context = canvas.getContext("2d");
  const zoomOut = document.querySelector<HTMLButtonElement>("#zoom-out")!;
  const zoomIn = document.querySelector<HTMLButtonElement>("#zoom-in")!;
  const reset = document.querySelector<HTMLButtonElement>("#reset-zoom")!;
  const nodes = new Map(scene.nodes.map((node) => [node.id, node]));
  const buttons = new Map<string, HTMLButtonElement>();
  let active = "";
  let zoom = 100;
  const layers = document.querySelector<HTMLElement>("#graph-layers")!;
  const retry = document.querySelector<HTMLButtonElement>("#retry-gpu")!;
  const lose = document.querySelector<HTMLButtonElement>("#lose-gpu")!;
  let gpu: { canvas: HTMLCanvasElement; graph: GpuGraph } | undefined;

  function fallback(message: string) {
    const previous = gpu;
    gpu = undefined;
    previous?.canvas.remove();
    previous?.graph.free();
    lose.disabled = true;
    retry.disabled = false;
    renderer.textContent = `Canvas2D · ${message}`;
    draw();
  }

  async function tryGpu() {
    if (gpu) return;
    document.querySelector<HTMLElement>("#gpu-controls")!.hidden = false;
    retry.disabled = true;
    const gpuCanvas = document.createElement("canvas");
    gpuCanvas.setAttribute("aria-hidden", "true");
    renderer.textContent = "Canvas2D · starting wgpu";
    let lost = false;
    try {
      const graph = await createGpuGraph(gpuCanvas, (reason) => {
        lost = true;
        console.info("wgpu device lost", reason);
        if (gpu?.canvas === gpuCanvas) fallback("WebGPU device lost");
      });
      if (lost) {
        graph.free();
        fallback("WebGPU device lost during setup");
        return;
      }
      gpu = { canvas: gpuCanvas, graph };
      layers.prepend(gpuCanvas);
      draw();
      if (gpu) {
        renderer.textContent = "wgpu WebGPU · 2,000 labeled nodes";
        lose.disabled = false;
      }
    } catch (error) {
      console.warn("wgpu initialization failed", error);
      fallback("WebGPU renderer could not start");
    }
  }
  retry.addEventListener("click", () => void tryGpu());
  lose.addEventListener("click", () => gpu?.graph.simulate_device_loss());


  function draw() {
    if (!context) return;
    const fitWidth = benchmark ? width : Math.min(plot.clientWidth, (plot.offsetHeight * width) / height);
    canvas.width = Math.floor((Math.max(440, fitWidth) * zoom) / 100);
    canvas.height = Math.round((canvas.width * height) / width);
    context.setTransform(canvas.width / width, 0, 0, canvas.height / height, 0, 0);
    if (gpu) {
      const gpuCanvas = gpu.canvas;
      if (gpuCanvas.width !== canvas.width) gpuCanvas.width = canvas.width;
      if (gpuCanvas.height !== canvas.height) gpuCanvas.height = canvas.height;
      try {
        gpu.graph.draw(canvas.width, canvas.height, benchmarkTriangles(scene, nodes, active));
      } catch (error) {
        console.warn("wgpu draw failed", error);
        fallback("WebGPU draw failed");
        return;
      }
    } else if (!benchmark) {
      context.strokeStyle = "#273d44";
      context.beginPath();
      for (let x = 20; x < width; x += 24) {
        for (let y = 20; y < height; y += 24) {
          context.rect(x, y, 0.9, 0.9);
        }
      }
      context.stroke();
    }
    for (const edge of gpu ? [] : scene.edges) {
      const from = nodes.get(edge.source);
      const to = nodes.get(edge.target);
      if (!from || !to) continue;
      const selected = from.id === active || to.id === active;
      context.strokeStyle = selected ? "#e9cc8a" : "#638289";
      context.lineWidth = selected ? 2.4 : 1.5;
      context.beginPath();
      context.moveTo(from.x, from.y);
      context.lineTo(to.x, to.y);
      context.stroke();
    }
    context.font = benchmark ? "11px system-ui" : "600 15px system-ui";
    context.textAlign = "center";
    for (const node of scene.nodes) {
      if (!gpu) {
        context.fillStyle = node.id === active ? "#f3d395" : "#a5cfb6";
        context.beginPath();
        const radius = benchmark ? (node.id === active ? 5 : 3) : (node.id === active ? 9 : 6);
        context.arc(node.x, node.y, radius, 0, Math.PI * 2);
        context.fill();
      }
      context.fillStyle = "#f1eee0";
      context.strokeStyle = "#13272d";
      context.lineWidth = 4;
      const y = node.y + (benchmark ? 16 : 28);
      context.strokeText(node.label, node.x, y);
      context.fillText(node.label, node.x, y);
    }
  }

  function select(id: string) {
    const node = nodes.get(id);
    if (!node) return;
    buttons.get(active)?.setAttribute("aria-pressed", "false");
    active = id;
    buttons.get(id)?.setAttribute("aria-pressed", "true");
    selection.textContent = node.label;
    const items: HTMLLIElement[] = [];
    for (const edge of scene.edges) {
      if (edge.source !== id && edge.target !== id) continue;
      const from = nodes.get(edge.source);
      const to = nodes.get(edge.target);
      if (!from || !to) continue;
      const item = document.createElement("li");
      item.textContent = `${from.label} · ${edge.label} · ${to.label}`;
      items.push(item);
    }
    connections.replaceChildren(...items);
    draw();
  }

  for (const node of scene.nodes) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    const kind = document.createElement("span");
    button.type = "button";
    button.dataset.node = node.id;
    button.setAttribute("aria-pressed", "false");
    button.textContent = node.label;
    kind.textContent = node.kind;
    button.append(kind);
    button.addEventListener("click", () => select(node.id));
    buttons.set(node.id, button);
    item.append(button);
    list.append(item);
  }
  select(scene.nodes[0]?.id ?? "");

  if (!context) {
    plot.hidden = true;
    zoomOut.parentElement!.hidden = true;
    renderer.textContent = "HTML node list · Canvas unavailable";
    return;
  }

  function setZoom(level: number) {
    zoom = level;
    zoomIn.disabled = zoom >= (benchmark ? 150 : 200);
    zoomOut.disabled = zoom <= 100;
    reset.textContent = `${zoom}%`;
    draw();
  }
  zoomIn.addEventListener("click", () => setZoom(Math.min(benchmark ? 150 : 200, zoom + 25)));
  zoomOut.addEventListener("click", () => setZoom(Math.max(100, zoom - 25)));
  reset.addEventListener("click", () => {
    setZoom(100);
    plot.scrollTo(0, 0);
  });
  canvas.addEventListener("click", (event) => {
    const bounds = canvas.getBoundingClientRect();
    const x = ((event.clientX - bounds.left) * width) / bounds.width;
    const y = ((event.clientY - bounds.top) * height) / bounds.height;
    const node = scene.nodes.find((node) => Math.hypot(node.x - x, node.y - y) <= 16);
    if (node) select(node.id);
  });
  new ResizeObserver(draw).observe(plot);
  setZoom(100);
  return {
    tryGpu,
    visibleNode(id: string) {
      const node = nodes.get(id);
      if (!node) return false;
      const x = Math.round((node.x * canvas.width) / width);
      const y = Math.round((node.y * canvas.height) / height);
      const [red, green, blue, alpha] = context.getImageData(x, y, 1, 1).data;
      return red === 243 && green === 211 && blue === 149 && alpha === 255;
    },
  };
}

async function probeWebGpu(): Promise<WebGpuStatus> {
  const { gpu } = navigator as Navigator & { gpu?: { requestAdapter(): Promise<object | null> } };
  if (!gpu) return "api_unavailable";

  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const timeout = new Promise<"timed_out">((resolve) => {
      timer = setTimeout(() => resolve("timed_out"), 5_000);
    });
    const adapter = gpu.requestAdapter().then((value) => value ? "available" : "adapter_unavailable");
    return await Promise.race([adapter, timeout]);
  } catch {
    return "probe_failed";
  } finally {
    clearTimeout(timer);
  }
}

void start().catch(() => {
  host.textContent = "Could not load the local scene.";
});
