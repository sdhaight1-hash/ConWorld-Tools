import { expect, test } from "@playwright/test";

test("the bundled scene works with no WebGPU and no external requests", async ({ page }) => {
  const external: string[] = [];
  await page.route("**/*", async (route) => {
    if (new URL(route.request().url()).origin !== "http://localhost:4173") {
      external.push(route.request().url());
      await route.abort();
      return;
    }
    await route.continue();
  });
  await page.addInitScript(() => Object.defineProperty(navigator, "gpu", { value: undefined }));
  await page.goto("/");
  await expect(page.getByRole("status").filter({ hasText: "WebGPU unavailable" })).toBeVisible();
  const beatrice = page.getByRole("button", { name: /Beatrice Thorne/ });
  await beatrice.focus();
  await page.keyboard.press("Enter");
  await expect(beatrice).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByText("Beatrice Thorne · Member of · Merchant Guild")).toBeVisible();
  const plot = page.getByRole("region", { name: "Scrollable diagram" });
  const canvas = plot.locator("canvas");
  const initialWidth = await canvas.getAttribute("width");
  await page.getByRole("button", { name: "Zoom in" }).click();
  await expect(canvas).not.toHaveAttribute("width", initialWidth!);
  await expect.poll(() => plot.evaluate((el) => el.scrollHeight > el.clientHeight)).toBe(true);
  await page.getByRole("button", { name: "Reset zoom" }).click();
  await expect(canvas).toHaveAttribute("width", initialWidth!);
  expect(external).toEqual([]);
});

test("the node inspector works when Canvas2D is unavailable", async ({ page }) => {
  await page.addInitScript(() => { HTMLCanvasElement.prototype.getContext = () => null; });
  await page.goto("/");
  await expect(page.getByRole("status").filter({ hasText: "Canvas unavailable" })).toBeVisible();
  await page.getByRole("button", { name: /Merchant Guild/ }).click();
  await expect(page.getByRole("heading", { name: "Merchant Guild" })).toBeVisible();
  await expect(page.getByText("Merchant Guild · Based in · Northbridge")).toBeVisible();
});

test("the 2,000-node graph remains selectable without WebGPU", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(navigator, "gpu", { value: undefined }));
  await page.goto("/?benchmark=1");
  await expect(page.getByRole("heading", { name: "Renderer benchmark" })).toBeVisible();
  await expect(page.locator("#nodes button")).toHaveCount(2_000);
  await expect(page.getByRole("status").filter({ hasText: "WebGPU unavailable" })).toBeVisible();
  await page.locator("#nodes button[data-node='2000']").click();
  await expect(page.getByRole("heading", { name: "Node 2000" })).toBeVisible();
});

test("wgpu renders, pans, zooms and recovers from device loss", async ({ page }) => {
  test.skip(process.env.ATLAS_SOFTWARE_WEBGPU !== "1", "Requires the software WebGPU test configuration");
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/?benchmark=1");
  const gpuStatus = page.getByRole("status").filter({ hasText: "wgpu WebGPU" });
  await expect(gpuStatus).toContainText("2,000 labeled nodes");
  const plot = page.getByRole("region", { name: "Scrollable diagram" });
  await expect(plot.locator("canvas")).toHaveCount(2);
  const { x, y } = (await plot.locator("#graph").boundingBox())!;
  await page.mouse.click(x + 90, y + 24);
  await expect(page.getByRole("heading", { name: "Node 0002" })).toBeVisible();
  await page.getByRole("button", { name: "Zoom in" }).click();
  await expect(plot.locator("#graph")).toHaveAttribute("width", "3750");
  await plot.evaluate((el) => el.scrollTo(760, 310));
  await expect.poll(() => plot.evaluate((el) => [el.scrollLeft, el.scrollTop])).toEqual([760, 310]);
  await page.getByRole("button", { name: "Simulate device loss" }).click();
  await expect(page.getByRole("status").filter({ hasText: "WebGPU device lost" })).toBeVisible();
  await expect(plot.locator("canvas")).toHaveCount(1);
  await expect(page.getByRole("heading", { name: "Node 0002" })).toBeVisible();
  await page.getByRole("button", { name: "Retry WebGPU" }).click();
  await expect(gpuStatus).toBeVisible();
  await expect(plot.locator("canvas")).toHaveCount(2);
  await expect.poll(() => plot.evaluate((el) => [el.scrollLeft, el.scrollTop])).toEqual([760, 310]);
  expect(errors).toEqual([]);
});
