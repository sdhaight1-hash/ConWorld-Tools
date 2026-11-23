import { test, expect } from "@playwright/test";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

test("isolated World Bible prototype: shared identity, relationships and reader", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(pathToFileURL(resolve("../../Design-Docs/Research-Studies/world-bible-prototype/index.html")).href);
  const workspace = page.locator("#workspace");
  await workspace.getByRole("button", { name: /Mara Vale/ }).click();
  await workspace.getByRole("button", { name: "Connections", exact: true }).click();
  await page.locator("#targetSearch").fill("Lady");
  await page.locator("#target").selectOption("crown");
  await expect(page.locator("#predicate")).toHaveValue("Holds title");
  await page.locator("#since").fill("1885");
  await workspace.getByRole("button", { name: "Add connection" }).click();
  await expect(workspace.getByText("Holds title · 1885")).toBeVisible();
  await workspace.getByRole("button", { name: "Hierarchy", exact: true }).click();
  await expect(workspace.getByRole("button", { name: "House Vale" })).toBeVisible();
  await expect(workspace.getByRole("button", { name: "Northbridge" })).toBeVisible();
  await workspace.getByRole("button", { name: "Edit", exact: true }).click();
  await workspace.getByLabel("Article narrative").fill("Revised fictional biography.");
  await workspace.getByRole("button", { name: "Save prototype article" }).click();
  await expect(workspace.getByText("Revised fictional biography.")).toBeVisible();
  await workspace.getByRole("button", { name: "Reader preview" }).click();
  await expect(workspace.getByText("Hidden Council")).toHaveCount(0);
  await expect(workspace.getByText("Lady of Blackthorn")).toHaveCount(0);
  await workspace.getByRole("button", { name: "Return to author" }).click();
  await page.getByRole("button", { name: "＋ New entry" }).click();
  await page.locator("#newName").fill("New witness");
  await page.getByRole("button", { name: "Create identity only" }).click();
  await expect(workspace.getByText("This identity has no article.")).toBeVisible();
  await page.locator("#directory").getByRole("button", { name: "Mara Vale · Person" }).click();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const stamp = new Date().toISOString().slice(0,19).replace("T","_").replaceAll(":","-")+"Z";
    await page.screenshot({path:"../../Development-Screenshots/"+stamp+"_phase-04_canvas_world-bible-prototype_01.png"});
  }
});
