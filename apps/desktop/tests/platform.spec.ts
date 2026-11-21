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


test("the browser-only preview does not expose native author history", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("region", { name: "Scrollable diagram" })).toBeVisible();
  await expect(page.locator("#history")).toBeHidden();
});

test("browser preview does not expose the persistent native author workspace", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#author-workspace")).toBeHidden();
});


test("native author form refuses a missing historical day instead of silently saving day zero", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "00000000-0000-4000-8000-000000000001", name: "Audit world", revision: 0 };
    const people = [
      { id: "00000000-0000-4000-8000-000000000002", label: "Alice" },
      { id: "00000000-0000-4000-8000-000000000003", label: "Beatrice" },
    ];
    const writes: string[] = [];
    Object.assign(window, {
      isTauri: true,
      __authorTestWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string) => {
          if (command === "platform_scene") {
            return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          }
          if (command === "northbridge_history") {
            return { day: 0, relationships: [], cause: null };
          }
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") {
            return { world, day: 0, view_revision: 0, people, relationships: [], events: [] };
          }
          if (command === "workspace_create_relationship") {
            writes.push(command);
            return null;
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  await expect(page.getByRole("region", { name: "Persistent author workspace" })).toBeVisible();
  const form = page.locator("#author-create-relationship");
  await expect(form.locator('select[name="subject"]')).toHaveValue("00000000-0000-4000-8000-000000000002");
  await expect(form.locator('select[name="target"]')).toHaveValue("00000000-0000-4000-8000-000000000003");
  const day = page.locator("#author-day");
  await day.fill("");
  await day.blur();
  await expect(form.getByRole("button", { name: "Create relationship at selected day" })).toBeDisabled();
  await expect(page.locator("#author-status")).toContainText("Enter a whole-number historical day.");
  const writes = await page.evaluate(() => (window as unknown as { __authorTestWrites: string[] }).__authorTestWrites);
  expect(writes).toEqual([]);

  await day.fill("0");
  await day.blur();
  const revision = page.locator("#author-revision");
  await revision.fill("0");
  await revision.blur();
  await expect(page.locator("#author-status")).toContainText("Read-only author revision");
  await expect(form.getByRole("button", { name: "Create relationship at selected day" })).toBeDisabled();
  await revision.fill("");
  await revision.blur();
  await expect(form.getByRole("button", { name: "Create relationship at selected day" })).toBeEnabled();
});

test("saved world graph follows the author view and submits a specific causing actor", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "River Archive", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }, { id: "beatrice", label: "Beatrice" }];
    const requests: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __eventRequests: requests,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, payload: { day: number; revision: number | null; request?: unknown; audience?: string }) => {
          if (command === "platform_scene") return { target: "native bridge test", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_evidence") {
            const rumor = { statement: "The rumor says the objection succeeded." };
            const privateAccount = { statement: "Alice knows the clerk was absent." };
            if (payload.audience === "author") return [
              { ...rumor, source: "Private diary", audience: "Public" },
              { ...privateAccount, source: "Clerk's letter", audience: "Character", reader_id: "alice" },
            ];
            return payload.audience === "alice" ? [rumor, privateAccount] : [rumor];
          }
          if (command === "workspace_record_event") {
            requests.push(payload.request);
            return null;
          }
          if (command === "workspace_snapshot") {
            const later = payload.day >= 10 && payload.revision !== 3;
            return {
              world, day: payload.day, view_revision: payload.revision ?? 4, people,
              definitions: [{
                id: "mediator", label: "Mediator", category: "role", description: "Negotiates between parties",
                reverse_label: null, symmetric: false,
              }],
              relationships: [{
                id: "bond", subject_id: "alice", target_id: "beatrice", kind: "friendship",
                trust: later ? 12 : 90, tags: [later ? "estranged" : "friends"], starts_at: later ? 10 : 0,
                cause: later ? {
                  event_id: "letter", day: 10, event: "Disclosed letter", actor: "Beatrice",
                  action: "Disclose", outcome: "succeeded", intent: "Publicize", source: "Folio 14",
                  explanation: "A confidence was broken", before_trust: 90, before_tags: ["friends"],
                  after_trust: 12, after_tags: ["estranged"],
                } : null,
              }],
              events: later ? [{
                id: "letter", label: "Disclosed letter", day: 10, citation: "Folio 14",
                participants: [{ person_id: "alice", role: "actor" }, { person_id: "beatrice", role: "actor" }],
                actions: [
                  { id: "prevent", person_id: "alice", action: "Prevent", outcome: "failed", intent: "Protect the letter" },
                  { id: "disclose", person_id: "beatrice", action: "Disclose", outcome: "succeeded", intent: "Publicize" },
                ],
              }] : [],
            };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  await expect(graph).toBeVisible();
  const day = page.locator("#author-day");
  await day.fill("8");
  await day.blur();
  await expect(page.locator("#author-world-info")).toContainText("historical day 8");

  const editor = page.locator("#author-record-event");
  await editor.getByLabel("Event name").fill("Council dispute");
  await editor.getByLabel("Source citation").fill("Council minutes");
  for (const [i, person] of ["alice", "beatrice"].entries()) {
    await editor.getByRole("button", { name: "Add participant" }).click();
    const row = editor.locator("#event-participants fieldset").nth(i);
    await row.getByLabel("Person").selectOption(person);
    await row.getByLabel("Action", { exact: true }).fill(i === 0 ? "Object" : "Disclose");
    await row.getByLabel("Intent").fill("Settle the dispute");
  }
  await editor.getByRole("button", { name: "Add participant" }).click();
  const mediation = editor.locator("#event-participants fieldset").nth(2);
  await mediation.getByLabel("Role").selectOption({ label: "Mediator" });
  await expect(mediation.getByLabel("Action", { exact: true })).toBeHidden();
  await editor.getByLabel("Record a relationship effect").check();
  await editor.getByLabel("Causing action").selectOption({ label: "Beatrice · Disclose" });
  await editor.getByLabel("Explanation").fill("Disclosure broke confidence");
  await editor.getByRole("button", { name: "Save event" }).click();
  await expect(page.locator("#author-status")).toHaveText("Saved to local world history.");
  expect(await page.evaluate(() => (window as unknown as { __eventRequests: unknown[] }).__eventRequests)).toMatchObject([{
    expected_revision: 4, day: 8,
    participants: [{ person_id: "alice" }, { person_id: "beatrice" }, { role: "definition:mediator", action: null }],
    effect: { relationship_id: "bond", actor_row: 1 },
  }]);

  await day.fill("10");
  await day.blur();
  await graph.locator("#nodes button[data-node=bond]").click();
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await expect(inspector).toContainText("Before Trust 90 · friends");
  await expect(inspector).toContainText("After Trust 12 · estranged");
  await inspector.getByRole("button", { name: "Inspect source event" }).click();
  await expect(inspector).toContainText("Alice · Prevent (failed)");
  await expect(inspector).toContainText("Beatrice · Disclose (succeeded)");
  await expect(inspector).toContainText("Private diary");
  await inspector.getByLabel("Accounts visible to").selectOption("public");
  await expect(inspector).toContainText("The rumor says the objection succeeded.");
  await expect(inspector).not.toContainText("Private diary");
  await expect(inspector).not.toContainText("Alice knows the clerk was absent.");
  await inspector.getByLabel("Accounts visible to").selectOption("alice");
  await expect(inspector).toContainText("Alice knows the clerk was absent.");
  await expect(inspector).toContainText("Alice · Prevent (failed)");
  await inspector.getByRole("button", { name: "Edit this event" }).click();
  await expect(editor.getByLabel("Event name")).toHaveValue("Disclosed letter");
  await editor.getByRole("button", { name: "Correct action: Prevent" }).click();
  await expect(editor.locator("#event-participants").getByLabel("Action", { exact: true })).toHaveValue("Prevent");

  const revision = page.locator("#author-revision");
  await revision.fill("3");
  await revision.blur();
  await expect(graph.locator("#nodes button[data-node=letter]")).toHaveCount(0);
  await expect(editor.getByRole("button", { name: "Save event" })).toBeDisabled();
});


test("native historical search keeps ambiguous identities and navigates the real graph view", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Search world", revision: 3 };
    const people = [
      { id: "alice-a", label: "Alice Vale" },
      { id: "alice-b", label: "Alice Vale" },
      { id: "beatrice", label: "Beatrice Thorne" },
    ];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __searchCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 3,
            people, relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_search") {
            calls.push(args);
            return { revision: args.revision ?? 3, truncated: false,
              rows: args.mode === "backlinks" ? [{ id: "bond-1", label: "Friendship", kind: "relationship" }] :
                people.slice(0, args.revision === 1 ? 1 : 2).map(person => ({ ...person, kind: "person" })) };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-query-form");
  await form.getByLabel("Find name").fill("Alice Vale");
  await form.getByLabel("Match names").selectOption("exact");
  await form.getByRole("button", { name: "Search history" }).click();
  const results = page.locator("#author-query-results");
  await expect(results.getByRole("button", { name: /^Alice Vale ·/ })).toHaveCount(2);
  await results.getByRole("button", { name: /^Alice Vale ·.*alice-b$/ }).click();
  await expect(page.locator('#nodes button[data-node="alice-b"]')).toHaveAttribute("aria-pressed", "true");
  const revision = page.locator("#author-revision");
  await revision.fill("1");
  await revision.blur();
  await expect(results.getByRole("button", { name: /^Alice Vale ·/ })).toHaveCount(0);
  await form.getByRole("button", { name: "Search history" }).click();
  await expect(results.getByRole("button", { name: /^Alice Vale ·/ })).toHaveCount(1);
  await results.getByRole("button", { name: "Find references to Alice Vale alice-a" }).click();
  await expect(results.getByText(/Friendship · relationship · bond-1/)).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as { __searchCalls: unknown[] }).__searchCalls);
  expect(calls).toMatchObject([
    { worldId: "world", day: 0, revision: null, mode: "exact", term: "Alice Vale", limit: 100 },
    { worldId: "world", day: 0, revision: 1, mode: "exact", term: "Alice Vale", limit: 100 },
    { worldId: "world", day: 0, revision: 1, mode: "backlinks", term: "alice-a", limit: 100 },
  ]);
});


test("native path explorer uses typed direction, hop and visit budgets at the selected history", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Paths", revision: 5 };
    const people = [
      { id: "alice", label: "Alice" }, { id: "beatrice", label: "Beatrice" },
      { id: "cara", label: "Cara" },
    ];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __pathCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 5,
            people, relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_query_path") {
            const request = args.request as { day: number; revision: number | null; max_depth: number; max_visited: number };
            calls.push(request);
            const rows = request.day < 10 || request.revision === 3 ? [] :
              people.slice(1, request.max_depth === 1 ? 2 : 3).map(person => ({ ...person, kind: "person" }));
            return { revision: request.revision ?? 5, truncated: request.max_visited === 2, rows };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const day = page.locator("#author-day");
  await day.fill("10");
  await day.blur();
  const form = page.locator("#author-path-form");
  await form.getByLabel("Start person").selectOption("alice");
  await form.getByLabel("Relationship kind").fill("friendship");
  await form.getByLabel("Direction").selectOption("outgoing");
  await form.getByLabel("Maximum hops").fill("2");
  await form.getByLabel("Visited budget").fill("3");
  await form.getByRole("button", { name: "Explore connections" }).click();
  const results = page.locator("#author-query-results");
  await expect(results.getByRole("button", { name: /^Beatrice ·/ })).toBeVisible();
  await expect(results.getByRole("button", { name: /^Cara ·/ })).toBeVisible();
  await results.getByRole("button", { name: /^Beatrice ·/ }).click();
  await expect(page.locator('#nodes button[data-node="beatrice"]')).toHaveAttribute("aria-pressed", "true");
  await day.fill("9");
  await day.blur();
  await expect(results.getByRole("button", { name: /^Beatrice ·/ })).toHaveCount(0);
  await form.getByRole("button", { name: "Explore connections" }).click();
  await expect(page.locator("#author-query-status")).toContainText("0 match(es)");
  await day.fill("10");
  await day.blur();
  const revision = page.locator("#author-revision");
  await revision.fill("3");
  await revision.blur();
  await form.getByRole("button", { name: "Explore connections" }).click();
  await expect(page.locator("#author-query-status")).toContainText("0 match(es)");
  await revision.fill("");
  await revision.blur();
  await form.getByLabel("Maximum hops").fill("1");
  await form.getByLabel("Visited budget").fill("2");
  await form.getByRole("button", { name: "Explore connections" }).click();
  await expect(results.getByRole("button", { name: /^Beatrice ·/ })).toBeVisible();
  await expect(results.getByRole("button", { name: /^Cara ·/ })).toHaveCount(0);
  await expect(page.locator("#author-query-status")).toContainText("more results exist");
  const recorded = await page.evaluate(() => (window as unknown as { __pathCalls: unknown[] }).__pathCalls);
  expect(recorded).toMatchObject([
    { world_id: "world", day: 10, revision: null, start_id: "alice", kind: "friendship",
      direction: "outgoing", max_depth: 2, max_visited: 3, limit: 100 },
    { world_id: "world", day: 9, revision: null },
    { world_id: "world", day: 10, revision: 3 },
    { world_id: "world", day: 10, revision: null, max_depth: 1, max_visited: 2 },
  ]);
});
