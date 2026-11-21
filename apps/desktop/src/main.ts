import { invoke, isTauri } from "@tauri-apps/api/core";
import northbridge from "./northbridge.json";
import "./style.css";
import { showAuthorWorkspace } from "./author-workspace";
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
  if (!response.smoke && !benchmark && isTauri() && graph) {
    await showAuthorWorkspace(response.scene, graph.setScene, graph.select);
  }
  const webgpu = await probeWebGpu();
  if (graph?.hasCanvas) renderer.textContent = `Canvas2D · ${webGpuLabels[webgpu]}`;
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


function showScene(scene: Scene, benchmark: boolean) {
  let width = benchmark ? benchmarkWidth : 680;
  let height = benchmark ? benchmarkHeight : 475;
  let savedWorld = false;
  let inspectRecord: ((id: string, panel: HTMLElement) => void) | undefined;
  const plot = document.querySelector<HTMLElement>("#plot")!;
  const list = document.querySelector<HTMLUListElement>("#nodes")!;
  const connections = document.querySelector<HTMLUListElement>("#connections")!;
  const details = document.querySelector<HTMLElement>("#record-details")!;
  const selection = document.querySelector<HTMLElement>("#selection")!;
  const canvas = document.querySelector<HTMLCanvasElement>("#graph")!;
  const context = canvas.getContext("2d");
  const zoomOut = document.querySelector<HTMLButtonElement>("#zoom-out")!;
  const zoomIn = document.querySelector<HTMLButtonElement>("#zoom-in")!;
  const reset = document.querySelector<HTMLButtonElement>("#reset-zoom")!;
  const nodes = new Map<string, Scene["nodes"][number]>();
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
    const fitWidth = benchmark || (savedWorld && width > 680) ? width : Math.min(plot.clientWidth, (plot.offsetHeight * width) / height);
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
      const labelWidth = savedWorld ? 132 : undefined;
      context.strokeText(node.label, node.x, y, labelWidth);
      context.fillText(node.label, node.x, y, labelWidth);
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
      const label = `${from.label} · ${edge.label} · ${to.label}`;
      if (savedWorld) {
        const link = document.createElement("button");
        link.type = "button";
        link.textContent = label;
        link.addEventListener("click", () => {
          const neighbor = edge.source === id ? edge.target : edge.source;
          select(neighbor);
          buttons.get(neighbor)?.focus();
        });
        item.append(link);
      } else {
        item.textContent = label;
      }
      items.push(item);
    }
    connections.replaceChildren(...items);
    details.replaceChildren();
    inspectRecord?.(id, details);
    draw();
  }

  function setScene(next: Scene, inspect?: typeof inspectRecord) {
    const sameWorld = savedWorld && inspect !== undefined;
    const focused = document.activeElement instanceof HTMLButtonElement
      ? document.activeElement.dataset.node
      : undefined;
    scene = next;
    inspectRecord = inspect;
    savedWorld = inspect !== undefined;
    if (!benchmark) {
      width = Math.max(680, ...scene.nodes.map((node) => node.x + 78));
      height = Math.max(475, ...scene.nodes.map((node) => node.y + 62));
    }
    if (!sameWorld) {
      zoom = 100;
      plot.scrollTo(0, 0);
    }
    nodes.clear();
    buttons.clear();
    list.replaceChildren();
    for (const node of scene.nodes) {
      nodes.set(node.id, node);
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
    if (nodes.size) {
      select(sameWorld && nodes.has(active) ? active : scene.nodes[0].id);
      if (focused) buttons.get(focused)?.focus();
    } else {
      active = "";
      selection.textContent = "No records at this date and revision";
      connections.replaceChildren();
      details.replaceChildren();
      draw();
    }
    setZoom(zoom);
  }

  if (!context) {
    plot.hidden = true;
    zoomOut.parentElement!.hidden = true;
    renderer.textContent = "HTML node list · Canvas unavailable";
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
  setScene(scene);
  return {
    hasCanvas: Boolean(context),
    setScene,
    select,
    tryGpu,
    visibleNode(id: string) {
      const node = nodes.get(id);
      if (!node || !context) return false;
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
