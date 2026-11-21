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
          if (command === "workspace_saved_queries") return [];
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
          if (command === "workspace_saved_queries") return [];
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
          if (command === "workspace_saved_queries") return [];
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
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 5,
            people, relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_query_path") {
            const request = args.request as { day: number; revision: number | null; max_depth: number; max_visited: number };
            calls.push(request);
            const rows = request.day < 10 || request.revision === 3 ? [] :
              people.slice(1, request.max_depth === 1 || request.max_visited === 2 ? 2 : 3).map(person => ({ ...person, kind: "person" }));
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
  await form.getByLabel("Maximum hops").fill("2");
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
    { world_id: "world", day: 10, revision: null, max_depth: 2, max_visited: 2 },
  ]);
});


test("historical full-text option invokes indexed label search without client-side FTS syntax", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Index world", revision: 1 };
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __fulltextCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 1,
            people: [{ id: "alice", label: "Alice Vale" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_search") {
            calls.push(args);
            return { revision: args.revision ?? 1, truncated: false,
              rows: [{ id: "alice", label: "Alice Vale", kind: "person" }] };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-query-form");
  await form.getByLabel("Find name").fill("Alice Vale");
  await form.getByLabel("Match names").selectOption("fulltext");
  await form.getByRole("button", { name: "Search history" }).click();
  await expect(page.locator("#author-query-results").getByRole("button", { name: /^Alice Vale ·/ })).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as { __fulltextCalls: unknown[] }).__fulltextCalls);
  expect(calls).toMatchObject([{ worldId: "world", day: 0, revision: null,
    mode: "fulltext", term: "Alice Vale", limit: 100 }]);
});


test("native author saves, revises and replays historical name searches", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Saved search world", revision: 1 };
    const alice = { id: "alice", label: "Alice Vale" };
    const calls: unknown[] = [];
    const saved: { id: string; name: string; mode: string; term: string; limit: number; recorded_at: number }[] = [];
    Object.assign(window, {
      isTauri: true,
      __savedQueryCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [alice], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") {
            return saved.filter(query => query.recorded_at <= (args.revision ?? world.revision));
          }
          if (command === "workspace_save_query") {
            const request = args.request as { expected_revision: number; query_id: string | null; name: string; mode: string; term: string; limit: number };
            if (request.expected_revision !== world.revision) throw new Error("stale revision");
            calls.push({ operation: "save", ...request });
            world.revision++;
            const query = { id: request.query_id ?? "saved-1", name: request.name, mode: request.mode,
              term: request.term, limit: request.limit, recorded_at: world.revision };
            saved.splice(0, saved.length, query);
            return query.id;
          }
          if (command === "workspace_search") {
            calls.push({ operation: "run", ...args });
            return { revision: args.revision ?? world.revision, truncated: false,
              rows: [{ ...alice, kind: "person" }] };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const searchForm = page.locator("#author-query-form");
  const savedForm = page.locator("#author-save-query-form");
  await searchForm.getByLabel("Find name").fill("Alice Vale");
  await searchForm.getByLabel("Match names").selectOption("fulltext");
  await savedForm.getByLabel("Saved search name").fill("Find Alice");
  await savedForm.getByRole("button", { name: "Save search definition" }).click();
  await expect(page.locator("#author-query-status")).toContainText("Saved search definition");
  await expect(savedForm.locator('select[name="query_id"]')).toHaveValue("saved-1");
  await savedForm.getByRole("button", { name: "Run saved search" }).click();
  await expect(page.locator("#author-query-results").getByRole("button", { name: /^Alice Vale ·/ })).toBeVisible();
  const revision = page.locator("#author-revision");
  await revision.fill("1");
  await revision.blur();
  await expect(savedForm.locator('select[name="query_id"]')).toHaveValue("");
  await expect(savedForm.getByRole("button", { name: "Save search definition" })).toBeDisabled();
  await revision.fill("");
  await revision.blur();
  await savedForm.locator('select[name="query_id"]').selectOption("saved-1");
  await savedForm.getByLabel("Saved search name").fill("Find Alice corrected");
  await savedForm.getByRole("button", { name: "Save search definition" }).click();
  await expect(page.locator("#author-query-status")).toContainText("Saved search definition");
  const calls = await page.evaluate(() => (window as unknown as { __savedQueryCalls: unknown[] }).__savedQueryCalls);
  expect(calls).toMatchObject([
    { operation: "save", expected_revision: 1, query_id: null, name: "Find Alice", mode: "fulltext", term: "Alice Vale" },
    { operation: "run", worldId: "world", mode: "fulltext", term: "Alice Vale", limit: 100 },
    { operation: "save", expected_revision: 2, query_id: "saved-1", name: "Find Alice corrected" },
  ]);
});


test("native mention discovery lists distinct collision identities and never accepts inferred facts", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Discovery world", revision: 2 };
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __discoveryCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 2,
            people: [{ id: "person-a", label: "Eleanor Vale" }, { id: "person-b", label: "Eleanor Vale" }],
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_discover_mentions") {
            calls.push(args);
            return { revision: args.revision ?? 2, truncated: false, rows: [
              { source_id: "claim", source_revision: 2, source_label: "Witness diary",
                statement: "Eleanor Vale met the Guild.", target_id: "person-a",
                target_label: "Eleanor Vale", start_byte: 0, end_byte: 12, collisions: 2,
                source_branch: "main", source_state_revision: 2, disposition: null },
              { source_id: "claim", source_revision: 2, source_label: "Witness diary",
                statement: "Eleanor Vale met the Guild.", target_id: "person-b",
                target_label: "Eleanor Vale", start_byte: 0, end_byte: 12, collisions: 2,
                source_branch: "main", source_state_revision: 2, disposition: null },
            ] };
          }
          throw new Error("Unexpected mock native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Find possible mentions" });
  await trigger.click();
  await expect(page.locator("#author-discovery-status")).toContainText("2 possible name candidate");
  const results = page.locator("#author-discovery-results li");
  await expect(results).toHaveCount(2);
  await expect(results.nth(0)).toContainText("Candidate only");
  await expect(results.nth(1)).toContainText("Candidate only");
  await expect(results.nth(0)).toContainText("person-a");
  await expect(results.nth(1)).toContainText("person-b");
  await results.nth(1).getByRole("button", { name: "Inspect candidate person" }).click();
  const calls = await page.evaluate(() => (window as unknown as { __discoveryCalls: unknown[] }).__discoveryCalls);
  expect(calls).toEqual([{ worldId: "world", day: 0, revision: null, limit: 100 }]);
  const revision = page.locator("#author-revision");
  await revision.fill("1");
  await revision.blur();
  await expect(results).toHaveCount(0);
  await expect(page.locator("#author-discovery-status")).toContainText("Choose Find possible mentions");
});


test("native author can approve and dismiss one historical reference without creating a relationship", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Reviewed world", revision: 2 };
    const calls: unknown[] = [];
    let disposition: string | null = null;
    Object.assign(window, {
      isTauri: true,
      __reviewCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "a", label: "Eleanor Vale" }, { id: "b", label: "Eleanor Vale" }],
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_discover_mentions") return {
            revision: args.revision ?? world.revision, truncated: false, rows: [
              { source_id: "claim", source_branch: "main", source_state_revision: 2,
                source_revision: 2, source_label: "Diary", statement: "Eleanor Vale spoke.",
                target_id: "a", target_label: "Eleanor Vale", start_byte: 0, end_byte: 12,
                collisions: 2, disposition: (args.revision ?? world.revision) < 3 ? null : disposition },
              { source_id: "claim", source_branch: "main", source_state_revision: 2,
                source_revision: 2, source_label: "Diary", statement: "Eleanor Vale spoke.",
                target_id: "b", target_label: "Eleanor Vale", start_byte: 0, end_byte: 12,
                collisions: 2, disposition: null },
            ],
          };
          if (command === "workspace_review_mention") {
            const request = args.request as { decision: string; expected_revision: number };
            calls.push({ command, request });
            if (request.expected_revision !== world.revision) throw new Error("stale author revision");
            disposition = request.decision;
            world.revision++;
            return world.revision;
          }
          throw new Error("Unexpected native call " + command);
        },
      },
    });
  });
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Find possible mentions" });
  await trigger.click();
  const results = page.locator("#author-discovery-results li");
  await expect(results).toHaveCount(2);
  await results.nth(0).getByRole("button", { name: "Approve link" }).click();
  await expect(results.nth(0)).toContainText("Approved hyperlink only");
  await expect(results.nth(1)).toContainText("Candidate only");
  await results.nth(0).getByRole("button", { name: "Dismiss candidate" }).click();
  await expect(results.nth(0)).toContainText("Dismissed candidate");
  const calls = await page.evaluate(() => (window as unknown as { __reviewCalls: unknown[] }).__reviewCalls);
  expect(calls).toMatchObject([
    { command: "workspace_review_mention", request: {
      world_id: "world", day: 0, expected_revision: 2, source_id: "claim", source_branch: "main",
      source_state_revision: 2, source_revision: 2, target_id: "a",
      start_byte: 0, end_byte: 12, decision: "approved",
    }},
    { command: "workspace_review_mention", request: {
      world_id: "world", day: 0, expected_revision: 3, target_id: "a", decision: "dismissed",
    }},
  ]);
  const archived = page.locator("#author-revision");
  await archived.fill("2");
  await archived.blur();
  await trigger.click();
  await expect(results.nth(0)).toContainText("Candidate only");
  await expect(results.nth(0).getByRole("button", { name: "Approve link" })).toHaveCount(0);
});


test("native indexed claim search shows author-only statement provenance and replays saved mode", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Document archive", revision: 2 };
    const saved: { id: string; name: string; mode: string; term: string; limit: number; recorded_at: number }[] = [];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __claimSearchCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") {
            return saved.filter((item) => item.recorded_at <= (args.revision ?? world.revision));
          }
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_search") {
            calls.push({ operation: "search", ...args });
            return { revision: args.revision ?? world.revision, truncated: false, rows: [
              { id: "claim-1", label: "Court testimony", kind: "claim",
                statement: "The obsidian ledger was concealed in the Guild archive.",
                source_citation: "Archive folio 24" },
            ] };
          }
          if (command === "workspace_save_query") {
            const request = args.request as { expected_revision: number; name: string; mode: string; term: string; limit: number };
            if (world.revision !== request.expected_revision) throw new Error("stale revision");
            calls.push({ operation: "save", ...request });
            world.revision++;
            saved.push({ id: "saved-claim", name: request.name, mode: request.mode, term: request.term,
              limit: request.limit, recorded_at: world.revision });
            return "saved-claim";
          }
          throw new Error("Unexpected mock-native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const search = page.locator("#author-query-form");
  await search.getByLabel("Find name").fill("obsidian ledger");
  await search.getByLabel("Match names").selectOption("claimtext");
  await search.getByRole("button", { name: "Search history" }).click();
  const result = page.locator("#author-query-results li");
  await expect(result).toHaveCount(1);
  await expect(result.locator("blockquote")).toContainText("obsidian ledger was concealed");
  await expect(result).toContainText("Source citation: Archive folio 24");
  await expect(result.getByRole("button", { name: "Find references" })).toBeVisible();
  const savedForm = page.locator("#author-save-query-form");
  await savedForm.getByLabel("Saved search name").fill("Ledger in statements");
  await savedForm.getByRole("button", { name: "Save search definition" }).click();
  await expect(savedForm.locator('select[name="query_id"]')).toHaveValue("saved-claim");
  await savedForm.getByRole("button", { name: "Run saved search" }).click();
  await expect(result.locator("blockquote")).toContainText("obsidian ledger");
  const calls = await page.evaluate(() => (window as unknown as { __claimSearchCalls: unknown[] }).__claimSearchCalls);
  expect(calls).toMatchObject([
    { operation: "search", worldId: "world", day: 0, revision: null, mode: "claimtext", term: "obsidian ledger", limit: 100 },
    { operation: "save", expected_revision: 2, mode: "claimtext", term: "obsidian ledger" },
    { operation: "search", mode: "claimtext", term: "obsidian ledger" },
  ]);
  const revision = page.locator("#author-revision");
  await revision.fill("2");
  await revision.blur();
  await expect(result).toHaveCount(0);
  await expect(savedForm.getByRole("button", { name: "Save search definition" })).toBeDisabled();
});


test("native author queries historical events by person and recorded role", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Event roles", revision: 3 };
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __participantQueries: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice Vale" }, { id: "bob", label: "Bob Thorne" }],
            relationships: [],
            events: Number(args.day) >= 10 ?
              [{ id: "meeting", label: "Guild meeting", day: 10, citation: null,
                 participants: [], actions: [] }] : [],
            definitions: [{ id: "role-uuid", label: "Mediator", category: "role",
              description: "", reverse_label: null, symmetric: false }],
          };
          if (command === "workspace_query_participant_events") {
            const request = args.request as { day: number; revision: number | null };
            calls.push(request);
            return { revision: request.revision ?? world.revision, truncated: false,
              rows: request.day >= 10 ? [{ id: "meeting", label: "Guild meeting", kind: "historical_event" }] : [] };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-participant-events-form");
  await expect(form.getByRole("button", { name: "Find participant events" })).toBeEnabled();
  await page.locator("#author-day").fill("10");
  await page.locator("#author-day").blur();
  await form.getByLabel("Participant person").selectOption("alice");
  await form.getByLabel("Recorded event role").selectOption("actor");
  await form.getByRole("button", { name: "Find participant events" }).click();
  await expect(page.locator("#author-query-results").getByRole("button", { name: /^Guild meeting ·/ })).toBeVisible();
  await form.getByLabel("Recorded event role").selectOption("definition:role-uuid");
  await form.getByRole("button", { name: "Find participant events" }).click();
  const calls = await page.evaluate(() => (window as unknown as { __participantQueries: unknown[] }).__participantQueries);
  expect(calls).toEqual([
    { world_id: "world", day: 10, revision: null, participant_id: "alice", role: "actor", limit: 100 },
    { world_id: "world", day: 10, revision: null, participant_id: "alice",
      role: "definition:role-uuid", limit: 100 },
  ]);
  await page.locator("#author-revision").fill("1");
  await page.locator("#author-revision").blur();
  await expect(page.locator("#author-query-results li")).toHaveCount(0);
  await form.getByRole("button", { name: "Find participant events" }).click();
  await expect(page.locator("#author-query-status")).toContainText("author revision 1");
});


test("native author composes bounded kind/source/name predicates and clears stale results", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Composite search", revision: 4 };
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __compositeCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? world.revision,
            people: [], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_composite_query") {
            const request = args.request as Record<string, unknown>;
            calls.push(request);
            return { revision: request.revision ?? world.revision, truncated: false,
              rows: [{ id: "claim-1", kind: "claim", label: "Diary testimony",
                statement: "The ledger was missing.", source_citation: "Archive 7" }] };
          }
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-composite-form");
  await form.getByRole("button", { name: "Run combined query" }).click();
  await expect(page.locator("#author-query-status")).toContainText("Select at least one");
  await form.getByLabel("Label contains").fill("Diary");
  await form.getByLabel("Object kind").selectOption("claim");
  await form.getByLabel("Has cited source ID").fill("source-uuid");
  await form.getByLabel("Combine selected filters").selectOption("or");
  await form.getByLabel("Exclude matches (NOT)").check();
  await form.getByRole("button", { name: "Run combined query" }).click();
  const results = page.locator("#author-query-results li");
  await expect(results).toHaveCount(1);
  await expect(results.first()).toContainText("Archive 7");
  await expect(results.first()).toContainText("ledger was missing");
  const calls = await page.evaluate(() => (window as unknown as { __compositeCalls: unknown[] }).__compositeCalls);
  expect(calls).toEqual([{
    world_id: "world", day: 0, revision: null, limit: 100,
    predicate: { op: "not", filter: { op: "or", filters: [
      { op: "contains", text: "Diary" },
      { op: "object_kind", kind: "claim" },
      { op: "has_source", source_id: "source-uuid" },
    ] } },
  }]);
  await page.locator("#author-revision").fill("2");
  await page.locator("#author-revision").blur();
  await expect(results).toHaveCount(0);
  await form.getByRole("button", { name: "Run combined query" }).click();
  await expect(page.locator("#author-query-status")).toContainText("author revision 2");
});


test("native author saves and replays the exact versioned composite AST", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Typed query history", revision: 2 };
    const saved: { id: string; name: string; mode: string; term: string; predicate?: unknown; limit: number; recorded_at: number }[] = [];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __savedCompositeCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") {
            return saved.filter(item => item.recorded_at <= (args.revision ?? world.revision));
          }
          if (command === "workspace_save_query") {
            const request = args.request as { expected_revision: number; name: string; query_id: string | null;
              mode: string; term: string; predicate?: unknown; limit: number };
            if (request.expected_revision !== world.revision) throw new Error("stale author revision");
            calls.push({ operation: "save", ...request });
            world.revision++;
            const result = { id: request.query_id ?? "saved-ast", name: request.name, mode: request.mode,
              term: request.term, predicate: request.predicate, limit: request.limit,
              recorded_at: world.revision };
            saved.splice(0, saved.length, result);
            return result.id;
          }
          if (command === "workspace_composite_query") {
            calls.push({ operation: "replay", ...args });
            return { revision: (args.request as { revision: number | null }).revision ?? world.revision,
              truncated: false,
              rows: [{ id: "claim", label: "Diary entry", kind: "claim",
                statement: "The ledger was moved.", source_citation: "Source 88" }] };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const filter = page.locator("#author-composite-form");
  const saved = page.locator("#author-save-query-form");
  const sourceId = "00000000-0000-4000-8000-000000000001";
  await filter.getByLabel("Label contains").fill("Diary");
  await filter.getByLabel("Object kind").selectOption("claim");
  await filter.getByLabel("Has cited source ID").fill(sourceId);
  await filter.getByLabel("Combine selected filters").selectOption("and");
  await saved.getByLabel("Saved search name").fill("Evidence diary");
  await saved.getByRole("button", { name: "Save combined query" }).click();
  await expect(page.locator("#author-query-status")).toContainText("Saved combined query definition");
  await expect(saved.locator('select[name="query_id"]')).toHaveValue("saved-ast");
  await filter.getByLabel("Label contains").fill("CHANGED UNSAVED FORM");
  await saved.getByRole("button", { name: "Run saved search" }).click();
  const matches = page.locator("#author-query-results li");
  await expect(matches).toHaveCount(1);
  await expect(matches).toContainText("Source 88");
  const calls = await page.evaluate(() => (window as unknown as { __savedCompositeCalls: unknown[] }).__savedCompositeCalls);
  const predicate = { op: "and", filters: [
    { op: "contains", text: "Diary" },
    { op: "object_kind", kind: "claim" },
    { op: "has_source", source_id: sourceId },
  ] };
  expect(calls).toMatchObject([
    { operation: "save", expected_revision: 2, query_id: null,
      mode: "composite", predicate, term: "", name: "Evidence diary" },
    { operation: "replay", request: {
      world_id: "world", day: 0, revision: null, predicate, limit: 100,
    } },
  ]);
  const revision = page.locator("#author-revision");
  await revision.fill("2");
  await revision.blur();
  await expect(matches).toHaveCount(0);
  await expect(saved.locator('select[name="query_id"]')).toHaveValue("");
  await expect(saved.getByRole("button", { name: "Save combined query" })).toBeDisabled();
  await revision.fill("");
  await revision.blur();
  await saved.locator('select[name="query_id"]').selectOption("saved-ast");
  await saved.getByRole("button", { name: "Run saved search" }).click();
  await expect(matches).toHaveCount(1);
});


test("native author creates and disables a historical alias without inferring a relationship", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Alias world", revision: 2 };
    const aliases: { id: string; person_id: string; label: string; starts_at: number | null;
      ends_before: number | null; active: boolean; recorded_at: number }[] = [];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __aliasCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice Rowan" }, { id: "bob", label: "Bob Vale" }],
            aliases: aliases.filter((item) => item.recorded_at <= (args.revision ?? world.revision)),
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_save_person_alias") {
            const request = args.request as { expected_revision: number; alias_id: string | null;
              person_id: string; label: string; starts_at: number | null; ends_before: number | null; active: boolean };
            if (request.expected_revision !== world.revision) throw new Error("stale author revision");
            calls.push({ operation: "save", ...request });
            world.revision++;
            const id = request.alias_id ?? "alias-one";
            const row = { id, person_id: request.person_id, label: request.label, starts_at: request.starts_at,
              ends_before: request.ends_before, active: request.active, recorded_at: world.revision };
            aliases.splice(0, aliases.length, row);
            return id;
          }
          if (command === "workspace_discover_mentions") {
            const matching = aliases.filter((item) => item.active && (args.day as number) >= (item.starts_at ?? -9999));
            return { revision: args.revision ?? world.revision, truncated: false,
              rows: matching.map((item) => ({
                source_id: "claim", source_branch: "main", source_state_revision: 2, source_revision: 2,
                source_label: "Court notes", statement: "The Magpie arrived.",
                target_id: item.person_id, target_label: "Alice Rowan", matched_label: "Magpie",
                start_byte: 4, end_byte: 10, collisions: 1, disposition: null,
              })) };
          }
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-person-alias");
  await page.locator("#author-day").fill("20");
  await page.locator("#author-day").blur();
  await form.getByLabel("Person", { exact: true }).selectOption("alice");
  await form.getByLabel("Alternative name").fill("Magpie");
  await form.getByLabel("Valid from day (optional)").fill("10");
  await form.getByRole("button", { name: "Save person alias" }).click();
  await expect(page.locator("#author-status")).toContainText("Saved to local world history");
  await form.getByLabel("Saved alias").selectOption("alias-one");
  await expect(form.getByLabel("Alternative name")).toHaveValue("Magpie");
  await page.getByRole("button", { name: "Find possible mentions" }).click();
  await expect(page.locator("#author-discovery-results")).toContainText("matched alias: Magpie");
  await expect(page.locator("#author-discovery-results")).toContainText("Candidate only");
  await form.getByLabel("Enable this alias").uncheck();
  await form.getByRole("button", { name: "Save person alias" }).click();
  await page.getByRole("button", { name: "Find possible mentions" }).click();
  await expect(page.locator("#author-discovery-results li")).toHaveCount(0);
  const calls = await page.evaluate(() => (window as unknown as { __aliasCalls: unknown[] }).__aliasCalls);
  expect(calls).toMatchObject([
    { operation: "save", expected_revision: 2, alias_id: null, person_id: "alice",
      label: "Magpie", starts_at: 10, ends_before: null, active: true },
    { operation: "save", expected_revision: 3, alias_id: "alias-one", person_id: "alice",
      label: "Magpie", starts_at: 10, ends_before: null, active: false },
  ]);
  await page.locator("#author-revision").fill("2");
  await page.locator("#author-revision").blur();
  await expect(form.getByRole("button", { name: "Save person alias" })).toBeDisabled();
});
