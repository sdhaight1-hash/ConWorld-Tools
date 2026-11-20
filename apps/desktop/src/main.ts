import { invoke, isTauri } from "@tauri-apps/api/core";
import northbridge from "./northbridge.json";
import "./style.css";

type Scene = typeof northbridge;
type PlatformScene = { target: string; scene: Scene };

const width = 680;
const height = 475;
const host = document.querySelector<HTMLElement>("#host")!;
const renderer = document.querySelector<HTMLElement>("#renderer")!;

async function start() {
  const response = isTauri()
    ? await invoke<PlatformScene>("platform_scene")
    : { target: "browser preview", scene: northbridge };
  host.textContent = `Local scene · ${response.target}`;
  document.querySelector<HTMLElement>("#workspace")!.hidden = false;
  showScene(response.scene);
}

function showScene(scene: Scene) {
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

  function draw() {
    if (!context) return;
    const fitWidth = Math.min(plot.clientWidth, (plot.offsetHeight * width) / height);
    canvas.width = Math.floor((Math.max(440, fitWidth) * zoom) / 100);
    canvas.height = Math.round((canvas.width * height) / width);
    context.setTransform(canvas.width / width, 0, 0, canvas.height / height, 0, 0);
    context.strokeStyle = "#273d44";
    context.beginPath();
    for (let x = 20; x < width; x += 24) {
      for (let y = 20; y < height; y += 24) {
        context.rect(x, y, 0.9, 0.9);
      }
    }
    context.stroke();
    for (const edge of scene.edges) {
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
    context.font = "600 15px system-ui";
    context.textAlign = "center";
    for (const node of scene.nodes) {
      context.fillStyle = node.id === active ? "#f3d395" : "#a5cfb6";
      context.beginPath();
      context.arc(node.x, node.y, node.id === active ? 9 : 6, 0, Math.PI * 2);
      context.fill();
      context.fillStyle = "#f1eee0";
      context.strokeStyle = "#13272d";
      context.lineWidth = 4;
      context.strokeText(node.label, node.x, node.y + 28);
      context.fillText(node.label, node.x, node.y + 28);
    }
  }

  function select(id: string) {
    const node = nodes.get(id);
    if (!node) return;
    active = id;
    for (const [key, button] of buttons) {
      button.setAttribute("aria-pressed", String(id === key));
    }
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
    zoomIn.disabled = zoom >= 200;
    zoomOut.disabled = zoom <= 100;
    reset.textContent = `${zoom}%`;
    draw();
  }
  zoomIn.addEventListener("click", () => setZoom(Math.min(200, zoom + 25)));
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
  void reportWebGpu();
}

async function reportWebGpu() {
  const { gpu } = navigator as Navigator & { gpu?: { requestAdapter(): Promise<object | null> } };
  if (!gpu) {
    renderer.textContent = "Canvas2D · WebGPU unavailable";
    return;
  }
  try {
    const adapter = await gpu.requestAdapter();
    renderer.textContent = adapter
      ? "Canvas2D · WebGPU adapter detected"
      : "Canvas2D · WebGPU adapter unavailable";
  } catch {
    renderer.textContent = "Canvas2D · WebGPU adapter probe failed";
  }
}

void start().catch(() => {
  host.textContent = "Could not load the local scene.";
});
