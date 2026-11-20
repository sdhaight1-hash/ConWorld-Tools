import type { Scene } from "./main";

export const benchmarkWidth = 3_000;
export const benchmarkHeight = 1_480;
export const benchmarkNodeCount = 2_000;

export function makeBenchmark(): Scene {
  const nodes: Scene["nodes"] = [];
  const edges: Scene["edges"] = [];
  for (let index = 0; index < benchmarkNodeCount; index += 1) {
    const id = String(index + 1).padStart(4, "0");
    const column = index % 50;
    nodes.push({
      id,
      label: `Node ${id}`,
      kind: "Benchmark",
      x: column * 60 + 30,
      y: Math.floor(index / 50) * 36 + 24,
    });
    if (column > 0) edges.push({ source: nodes[index - 1].id, target: id, label: "Adjacent" });
    if (index >= 50) edges.push({ source: nodes[index - 50].id, target: id, label: "Adjacent" });
  }
  return { nodes, edges };
}

export interface GpuGraph {
  draw(width: number, height: number, vertices: Float32Array): void;
  simulate_device_loss(): void;
  free(): void;
}

interface RendererModule {
  default(): Promise<unknown>;
  GraphRenderer: {
    create(canvas: HTMLCanvasElement, onLost: (reason: string) => void): Promise<GpuGraph>;
  };
}

let rendererModule: Promise<RendererModule> | undefined;

export async function createGpuGraph(canvas: HTMLCanvasElement, onLost: (reason: string) => void) {
  rendererModule ??= loadModule().catch((error: unknown) => {
    rendererModule = undefined;
    throw error;
  });
  return (await rendererModule).GraphRenderer.create(canvas, onLost);
}

async function loadModule(): Promise<RendererModule> {
  const url = new URL("wgpu/atlas_renderer.js", document.baseURI);
  const module: RendererModule = await import(/* @vite-ignore */ url.href);
  await module.default();
  return module;
}

export function benchmarkTriangles(scene: Scene, nodes: Map<string, Scene["nodes"][number]>, active: string) {
  const selected = [243 / 255, 211 / 255, 149 / 255];
  const nodeColor = [165 / 255, 207 / 255, 182 / 255];
  const edgeColor = [99 / 255, 130 / 255, 137 / 255];
  const vertices = new Float32Array((scene.edges.length + scene.nodes.length) * 30);
  let cursor = 0;

  function vertex(x: number, y: number, color: number[]) {
    vertices[cursor++] = (x * 2) / benchmarkWidth - 1;
    vertices[cursor++] = 1 - (y * 2) / benchmarkHeight;
    vertices.set(color, cursor);
    cursor += 3;
  }

  function quad(ax: number, ay: number, bx: number, by: number, cx: number, cy: number, dx: number, dy: number, color: number[]) {
    vertex(ax, ay, color);
    vertex(bx, by, color);
    vertex(cx, cy, color);
    vertex(ax, ay, color);
    vertex(cx, cy, color);
    vertex(dx, dy, color);
  }

  for (const edge of scene.edges) {
    const from = nodes.get(edge.source)!;
    const to = nodes.get(edge.target)!;
    const width = from.id === active || to.id === active ? 1.1 : 0.55;
    const length = Math.hypot(to.x - from.x, to.y - from.y);
    const nx = ((from.y - to.y) * width) / length;
    const ny = ((to.x - from.x) * width) / length;
    quad(from.x + nx, from.y + ny, to.x + nx, to.y + ny, to.x - nx, to.y - ny, from.x - nx, from.y - ny, width > 1 ? selected : edgeColor);
  }

  for (const node of scene.nodes) {
    const r = node.id === active ? 5 : 3;
    quad(node.x - r, node.y - r, node.x + r, node.y - r, node.x + r, node.y + r, node.x - r, node.y + r, node.id === active ? selected : nodeColor);
  }
  return vertices;
}
