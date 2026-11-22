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
    { worldId: "world", day: 0, revision: 3, mode: "exact", term: "Alice Vale", limit: 100 },
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
    { world_id: "world", day: 10, revision: 5, start_id: "alice", kind: "friendship",
      direction: "outgoing", max_depth: 2, max_visited: 3, limit: 100 },
    { world_id: "world", day: 9, revision: 5 },
    { world_id: "world", day: 10, revision: 3 },
    { world_id: "world", day: 10, revision: 5, max_depth: 2, max_visited: 2 },
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
  expect(calls).toMatchObject([{ worldId: "world", day: 0, revision: 1,
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
  expect(calls).toMatchObject([{ worldId: "world", day: 0, revision: 2, limit: 100 }]);
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
    { operation: "search", worldId: "world", day: 0, revision: 2, mode: "claimtext", term: "obsidian ledger", limit: 100 },
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
    { world_id: "world", day: 10, revision: 3, participant_id: "alice", role: "actor", limit: 100 },
    { world_id: "world", day: 10, revision: 3, participant_id: "alice",
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
    world_id: "world", day: 0, revision: 4, limit: 100,
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
  await filter.getByLabel("Indexed claim phrase").fill("ledger was moved");
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
    { op: "claim_full_text", text: "ledger was moved" },
    { op: "object_kind", kind: "claim" },
    { op: "has_source", source_id: sourceId },
  ] };
  expect(calls).toMatchObject([
    { operation: "save", expected_revision: 2, query_id: null,
      mode: "composite", predicate, term: "", name: "Evidence diary" },
    { operation: "replay", request: {
      world_id: "world", day: 0, revision: 3, predicate, limit: 100,
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
          if (command === "workspace_search") {
            if (args.mode !== "alias" || args.term !== "Magpie") throw new Error("Unexpected alias query");
            return { revision: args.revision ?? world.revision, truncated: false,
              rows: aliases.filter((item) => item.active).map((item) => ({
                id: item.person_id, label: "Alice Rowan", kind: "person",
              })) };
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
  const search = page.locator("#author-query-form");
  await search.getByLabel("Find name").fill("Magpie");
  await search.getByLabel("Match names").selectOption("alias");
  await search.getByRole("button", { name: "Search history" }).click();
  await expect(page.locator("#author-query-results")).toContainText("Alice Rowan");
  await form.getByLabel("Enable this alias").uncheck();
  await form.getByRole("button", { name: "Save person alias" }).click();
  await page.getByRole("button", { name: "Find possible mentions" }).click();
  await expect(page.locator("#author-discovery-results li")).toHaveCount(0);
  await expect(form.locator('select[name="alias_id"]')).toHaveValue("");
  await expect(form.getByLabel("Alternative name")).toHaveValue("");
  await expect(form.getByLabel("Valid from day (optional)")).toHaveValue("");
  await expect(form.getByLabel("Valid before day (optional)")).toHaveValue("");
  await expect(form.getByLabel("Enable this alias")).toBeChecked();
  await search.getByRole("button", { name: "Search history" }).click();
  await expect(page.locator("#author-query-status")).toContainText("0 match(es)");
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


test("oversized worlds preserve pinned indexed search while disabling incomplete graph editing", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "large", name: "Large archive", revision: 7 };
    const calls: { command: string; args: Record<string, unknown> }[] = [];
    let mismatch = false;
    Object.assign(window, {
      isTauri: true,
      __largeWorldCalls: calls,
      __largeWorldMismatch: (value: boolean) => { mismatch = value; },
      __largeWorldAdvance: () => { world.revision++; },
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          calls.push({ command, args });
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            scene_limited: args.revision !== 1,
            people: args.revision === 1 ? [{ id: "person", label: "Obsidian navigator" }] : [],
            relationships: [], events: [], definitions: [], aliases: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_search") return {
            revision: mismatch ? world.revision : args.revision, truncated: false,
            rows: [{ id: "person", label: "Obsidian navigator", kind: "person" }],
          };
          if (command === "workspace_query_participant_events" || command === "workspace_query_path") {
            const request = args.request as { revision: number };
            return { revision: request.revision, truncated: false,
              rows: [{ id: "meeting", label: "Guild meeting", kind: "historical_event" }] };
          }
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("Search-only mode");
  await expect(page.locator("#workspace")).toBeHidden();
  await expect(page.locator(".editor-forms")).toBeHidden();
  await expect(page.locator("#author-mention-discovery")).toBeVisible();
  const form = page.locator("#author-query-form");
  await form.getByLabel("Find name").fill("Obsidian");
  await form.getByLabel("Match names").selectOption("fulltext");
  const search = form.getByRole("button", { name: "Search history" });
  await page.evaluate(() => (window as unknown as { __largeWorldAdvance: () => void }).__largeWorldAdvance());
  await search.click();
  await expect(page.locator("#author-query-results")).toContainText("Obsidian navigator");
  await expect(page.locator("#author-query-status")).toContainText("author revision 7");
  await expect(page.locator("#author-path-form")).toBeHidden();
  await page.getByRole("button", { name: "Explore Obsidian navigator person" }).click();
  await expect(page.locator("#author-search-focus")).toContainText("Obsidian navigator · person");
  await expect(page.locator("#author-path-form select[name=start]")).toHaveValue("person");
  await page.getByRole("button", { name: "Explore connections" }).click();
  await expect(page.locator("#author-query-results")).toContainText("Guild meeting");
  await page.getByRole("button", { name: "Find participant events" }).click();
  await expect(page.locator("#author-query-results")).toContainText("Guild meeting");
  await page.evaluate(() => (window as unknown as { __largeWorldMismatch: (value: boolean) => void }).__largeWorldMismatch(true));
  await search.click();
  await expect(page.locator("#author-query-status")).toContainText("different author revision");
  await expect(page.locator("#author-query-results li")).toHaveCount(0);
  await page.evaluate(() => (window as unknown as { __largeWorldMismatch: (value: boolean) => void }).__largeWorldMismatch(false));
  await page.getByRole("button", { name: "Refresh historical view" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 8 / 8");
  await expect(page.locator("#author-path-form")).toBeHidden();
  await expect(page.locator("#author-participant-events-form")).toBeHidden();
  await search.click();
  await expect(page.locator("#author-query-status")).toContainText("author revision 8");
  const revision = page.locator("#author-revision");
  await revision.fill("1");
  await revision.blur();
  await expect(page.locator("#workspace")).toBeVisible();
  await expect(page.locator(".editor-forms")).toBeVisible();
  await expect(page.locator("#author-create-person button")).toBeDisabled();
  const calls = await page.evaluate(() => (window as unknown as { __largeWorldCalls: unknown[] }).__largeWorldCalls);
  expect(calls).toMatchObject([
    { command: "workspace_snapshot", args: { revision: null } },
    { command: "workspace_saved_queries", args: { revision: 7 } },
    { command: "workspace_search", args: { revision: 7, mode: "fulltext" } },
    { command: "workspace_query_path", args: { request: { revision: 7, start_id: "person" } } },
    { command: "workspace_query_participant_events", args: { request: { revision: 7, participant_id: "person" } } },
    { command: "workspace_search", args: { revision: 7, mode: "fulltext" } },
    { command: "workspace_snapshot", args: { revision: null } },
    { command: "workspace_saved_queries", args: { revision: 8 } },
    { command: "workspace_search", args: { revision: 8, mode: "fulltext" } },
    { command: "workspace_snapshot", args: { revision: 1 } },
    { command: "workspace_saved_queries", args: { revision: 1 } },
  ]);
});

test("historical result pages use opaque native tokens and reset on refresh", async ({ page }) => {
  await page.addInitScript(() => {
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __pageCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          const world = { id: "archive", name: "Archive", revision: 7 };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 7, scene_limited: true,
            people: [], relationships: [], events: [], definitions: [], aliases: [],
          };
          if (command === "workspace_saved_queries") return [];
          calls.push({ command, args });
          if (command === "workspace_search") return {
            revision: args.revision, truncated: true, next_cursor: "opaque-first",
            rows: [{ id: "one", label: "Identical name", kind: "person" }],
          };
          if (command === "workspace_next_search_page") return {
            revision: args.revision, truncated: false, next_cursor: null,
            rows: [{ id: "two", label: "Identical name", kind: "person" }],
          };
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("Search-only mode");
  const form = page.locator("#author-query-form");
  await form.getByLabel("Find name").fill("Identical");
  const search = form.getByRole("button", { name: "Search history" });
  const next = page.getByRole("button", { name: "Next results page" });
  await search.click();
  await expect(page.locator("#author-query-status")).toContainText("page 1");
  await expect(next).toBeVisible();
  await next.click();
  await expect(page.locator("#author-query-status")).toContainText("page 2");
  await expect(page.locator("#author-query-results")).toContainText("Identical name · person · two");
  await expect(next).toBeHidden();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await page.locator("#author-query-status").locator("..").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_query_mock-native-page-two_01.png",
      animations: "disabled",
    });
  }
  await search.click();
  await expect(next).toBeVisible();
  await page.getByRole("button", { name: "Refresh historical view" }).click();
  await expect(next).toBeHidden();
  const calls = await page.evaluate(() => (window as unknown as { __pageCalls: unknown[] }).__pageCalls);
  expect(calls).toMatchObject([
    { command: "workspace_search", args: { worldId: "archive", revision: 7 } },
    { command: "workspace_next_search_page", args: { worldId: "archive", day: 0, revision: 7, token: "opaque-first" } },
    { command: "workspace_search", args: { worldId: "archive", revision: 7 } },
  ]);
});

test("cancels exact pending search and ignores cancelled or superseded replies", async ({ page }) => {
  await page.addInitScript(() => {
    const pending = new Map<string, (value: unknown) => void>();
    const requests: string[] = [];
    const cancellations: string[] = [];
    Object.assign(window, {
      isTauri: true,
      __queryRequests: requests,
      __queryCancellations: cancellations,
      __resolveQuery: (request: string, label: string) => {
        pending.get(request)?.({
          revision: 7, truncated: true, next_cursor: "stale-token",
          rows: [{ id: request, label, kind: "person" }],
        });
      },
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          const world = { id: "archive", name: "Archive", revision: 7 };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 7, scene_limited: true,
            people: [], relationships: [], events: [], definitions: [], aliases: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_search") {
            const requestId = String(args.requestId);
            requests.push(requestId);
            return new Promise((resolve) => pending.set(requestId, resolve));
          }
          if (command === "workspace_cancel_query") {
            cancellations.push(String(args.requestId));
            return null;
          }
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("Search-only mode");
  const form = page.locator("#author-query-form");
  await form.getByLabel("Find name").fill("First search");
  const search = form.getByRole("button", { name: "Search history" });
  const cancel = page.getByRole("button", { name: "Cancel search" });
  const queryStatus = page.locator("#author-query-status");
  const requests = () => page.evaluate(() => (window as unknown as { __queryRequests: string[] }).__queryRequests);
  const cancelled = () => page.evaluate(() => (window as unknown as { __queryCancellations: string[] }).__queryCancellations);
  const resolve = (request: string, label: string) => page.evaluate(({ request, label }) =>
    (window as unknown as { __resolveQuery: (request: string, label: string) => void }).__resolveQuery(request, label),
    { request, label });
  await search.click();
  await expect.poll(requests).toHaveLength(1);
  await expect(cancel).toBeVisible();
  await search.click();
  await expect.poll(requests).toHaveLength(2);
  const [first, second] = await requests();
  expect(first).not.toBe(second);
  await expect.poll(cancelled).toEqual([first]);
  await resolve(first, "Superseded result");
  await expect(page.locator("#author-query-results li")).toHaveCount(0);
  await cancel.click();
  await expect.poll(cancelled).toEqual([first, second]);
  await expect(queryStatus).toContainText("Search cancellation requested.");
  await resolve(second, "Cancelled result");
  await expect(page.locator("#author-query-results li")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Next results page" })).toBeHidden();
  await search.click();
  await expect.poll(requests).toHaveLength(3);
  const third = (await requests())[2];
  await page.getByRole("button", { name: "Refresh historical view" }).click();
  await expect.poll(cancelled).toEqual([first, second, third]);
  await resolve(third, "Refreshed result");
  await expect(page.locator("#author-query-results li")).toHaveCount(0);
  await expect(cancel).toBeHidden();
});

test("saved parameters use typed bindings and server-side saved identity", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "world", name: "Parameterized archive", revision: 2 };
    const saved: Record<string, unknown>[] = [];
    const calls: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __parameterCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [], relationships: [], events: [], definitions: [], aliases: [],
          };
          if (command === "workspace_saved_queries") return saved.filter(item => Number(item.recorded_at) <= Number(args.revision ?? world.revision));
          if (command === "workspace_save_query") {
            const request = args.request as Record<string, unknown>;
            calls.push({ operation: "save", ...request });
            world.revision++;
            saved.push({ id: "parameterized", name: request.name, mode: "fulltext", term: request.term,
              limit: request.limit, recorded_at: world.revision,
              parameters: [{ name: "label_phrase_1", default: { type: "text", value: request.term } }] });
            return "parameterized";
          }
          if (command === "workspace_replay_saved_query") {
            calls.push({ operation: "replay", ...args });
            const request = args.request as { revision: number; bindings: { value: { value: string } }[] };
            return { revision: request.revision, truncated: false,
              rows: [{ id: "record", label: request.bindings[0].value.value, kind: "person" }] };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const form = page.locator("#author-query-form");
  const saved = page.locator("#author-save-query-form");
  await form.getByLabel("Find name").fill("Alice");
  await form.getByLabel("Match names").selectOption("fulltext");
  await saved.getByLabel("Saved search name").fill("Find a person");
  await saved.getByLabel("Allow typed values to change at replay").check();
  await saved.getByRole("button", { name: "Save search definition" }).click();
  const parameter = saved.getByLabel("label phrase 1");
  await expect(parameter).toHaveValue("Alice");
  await parameter.fill("Bob");
  await saved.getByRole("button", { name: "Run saved search" }).click();
  await expect(page.locator("#author-query-results")).toContainText("Bob");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await saved.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_query_saved-parameter-override_01.png",
      animations: "disabled",
    });
  }
  const calls = await page.evaluate(() => (window as unknown as { __parameterCalls: unknown[] }).__parameterCalls);
  expect(calls).toMatchObject([
    { operation: "save", parameterize: true, term: "Alice" },
    { operation: "replay", request: { world_id: "world", revision: 3, query_id: "parameterized",
      bindings: [{ name: "label_phrase_1", value: { type: "text", value: "Bob" } }] } },
  ]);
  await page.getByRole("button", { name: "Refresh historical view" }).click();
  await expect(parameter).toHaveValue("Alice");
  await page.locator("#author-revision").fill("2");
  await page.locator("#author-revision").blur();
  await expect(page.locator("#author-saved-parameters")).toBeHidden();
  await expect(saved.getByRole("button", { name: "Run saved search" })).toBeDisabled();
});

test("oversized world discovers one exact claim and cancels stale discovery", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Large archive", revision: 7 };
    const calls: Record<string, unknown>[] = [];
    const cancellations: string[] = [];
    const pending = new Map<string, (value: unknown) => void>();
    Object.assign(window, {
      isTauri: true,
      __focusDiscoveryCalls: calls,
      __focusDiscoveryCancellations: cancellations,
      __resolveDiscovery: (request: string) => pending.get(request)?.({
        revision: 7, truncated: false, rows: [{
          source_id: "claim", source_branch: "main", source_state_revision: 5,
          source_revision: 5, source_label: "Diary entry", statement: "Eleanor Vale arrived.",
          target_id: "person", target_label: "Eleanor Vale", matched_label: "Eleanor Vale",
          start_byte: 0, end_byte: 12, collisions: 2, disposition: null,
        }],
      }),
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: args.day, view_revision: args.revision ?? 7, scene_limited: true,
            people: [], relationships: [], events: [], definitions: [], aliases: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_search") return {
            revision: args.revision, truncated: false, rows: [{ id: "claim", label: "Diary entry",
              kind: "claim", statement: "Eleanor Vale arrived.", source_citation: "Diary 1" }],
          };
          if (command === "workspace_discover_mentions") {
            calls.push(args);
            return new Promise((resolve) => pending.set(String(args.requestId), resolve));
          }
          if (command === "workspace_cancel_query") {
            cancellations.push(String(args.requestId));
            return null;
          }
          throw new Error("Unexpected native command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("Search-only mode");
  const search = page.locator("#author-query-form");
  await search.getByLabel("Find name").fill("Eleanor");
  await search.getByLabel("Match names").selectOption("claimtext");
  await search.getByRole("button", { name: "Search history" }).click();
  const focus = page.getByRole("button", { name: "Find mentions in Diary entry claim" });
  const cancel = page.getByRole("button", { name: "Cancel mention scan" });
  const calls = () => page.evaluate(() => (window as unknown as { __focusDiscoveryCalls: Record<string, unknown>[] }).__focusDiscoveryCalls);
  const cancellations = () => page.evaluate(() => (window as unknown as { __focusDiscoveryCancellations: string[] }).__focusDiscoveryCancellations);
  const resolve = (request: string) => page.evaluate((request) =>
    (window as unknown as { __resolveDiscovery: (request: string) => void }).__resolveDiscovery(request), request);
  await focus.click();
  await expect.poll(calls).toHaveLength(1);
  const first = String((await calls())[0].requestId);
  expect((await calls())[0]).toMatchObject({ worldId: "archive", day: 0, revision: 7, sourceId: "claim", limit: 100 });
  await cancel.click();
  await expect.poll(cancellations).toContain(first);
  await resolve(first);
  await expect(page.locator("#author-discovery-results li")).toHaveCount(0);
  await focus.click();
  await expect.poll(calls).toHaveLength(2);
  const second = String((await calls())[1].requestId);
  await resolve(second);
  await expect(page.locator("#author-discovery-results")).toContainText("2 possible identity/identities");
  await expect(page.locator("#author-discovery-status")).toContainText("claim: Diary entry");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await page.locator("#author-mention-discovery").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_discovery_focused-claim_01.png",
      animations: "disabled",
    });
  }
  await page.getByRole("button", { name: "Refresh historical view" }).click();
  await expect(page.locator("#author-discovery-results li")).toHaveCount(0);
  await expect(cancel).toBeHidden();
});

test("public entry search uses reader IPC and discards obsolete responses", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "River Archive", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }];
    const requests: unknown[] = [];
    Object.assign(window, {
      isTauri: true,
      __entrySearchRequests: requests,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, payload: { text?: string; day?: number }) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: payload.day ?? 0, view_revision: 4, people,
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_entry_layout") return null;
          if (command === "workspace_search_public_entry") {
            requests.push(payload);
            if (payload.text === "old") return new Promise(resolve => {
              Object.assign(window, { __releaseOldEntrySearch: () => resolve({
                matches: [["old-block", "Obsolete result"]], truncated: false,
              }) });
            });
            if (payload.text === "gazette") return {
              matches: [["public-block", "Gazette biography"]], truncated: false,
            };
            if (payload.text === "revoked") return null;
            return { matches: [], truncated: false };
          }
          if (command === "workspace_public_entry") return { content: { Text: "Gazette biography" } };
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await graph.getByRole("button", { name: "Open character page" }).click();
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  const input = inspector.getByLabel("Find in public page");
  const find = inspector.getByRole("button", { name: "Find", exact: true });
  await input.fill("old");
  await find.click();
  await expect.poll(() => page.evaluate(() => typeof (window as unknown as {
    __releaseOldEntrySearch?: () => void;
  }).__releaseOldEntrySearch)).toBe("function");
  await input.fill("gazette");
  await find.click();
  await expect(inspector.getByText("1 matching blocks.", { exact: true })).toBeVisible();
  await expect(inspector.getByText("Gazette biography", { exact: true })).toBeVisible();
  await page.evaluate(() => (window as unknown as { __releaseOldEntrySearch: () => void }).__releaseOldEntrySearch());
  await expect(inspector.getByText("Obsolete result")).toHaveCount(0);
  const requests = await page.evaluate(() => (window as unknown as { __entrySearchRequests: unknown[] }).__entrySearchRequests);
  expect(requests).toEqual([
    { worldId: "archive", entryId: "alice", day: 0, text: "old" },
    { worldId: "archive", entryId: "alice", day: 0, text: "gazette" },
  ]);
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_query_mock-native-public-entry-find_01.png",
      animations: "disabled",
    });
  }
  await input.fill("private");
  await find.click();
  await expect(inspector.getByText("0 matching blocks.", { exact: true })).toBeVisible();
  await input.fill("revoked");
  await find.click();
  await expect(inspector.getByText("Unpublished at this date.", { exact: true })).toBeVisible();
});

test("world public search opens only the current reader projection", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "River Archive", revision: 4 };
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, payload: { day?: number; text?: string; entryId?: string }) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_snapshot") return {
            world, day: payload.day ?? 0, view_revision: 4, people: [],
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_search_public_pages") return {
            matches: payload.text === "gazette" ? [
              { entry: { world: "archive", id: "alice" }, element: "one", text: "Gazette biography" },
            ] : [], truncated: false,
          };
          if (command === "workspace_public_entry" && payload.entryId === "alice") return {
            content: { Stack: [{ content: { Text: "Gazette biography" } }, { content: { Text: "Released context" } }] },
          };
          if (command === "workspace_cancel_query") return null;
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const search = page.getByRole("region", { name: "Published page search" });
  await search.getByLabel("Text in public pages").fill("gazette");
  await search.getByRole("button", { name: "Search published pages" }).click();
  await expect(search.getByText("1 matching public blocks.")).toBeVisible();
  await search.getByRole("button", { name: "Read published page" }).click();
  await expect(search.getByText("Released context")).toBeVisible();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await search.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_query_mock-native-public-world-search_01.png",
      animations: "disabled",
    });
  }
  await search.getByLabel("Text in public pages").fill("private");
  await search.getByRole("button", { name: "Search published pages" }).click();
  await expect(search.getByText("0 matching public blocks.")).toBeVisible();
  await expect(search.getByText("Released context")).toHaveCount(0);
});

test("character canvas pins a private saved query and previews typed overrides", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "River Archive", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }];
    const requests: { command: string; payload: unknown }[] = [];
    let savedLayout: unknown = null;
    Object.assign(window, {
      isTauri: true,
      __canvasBindingRequests: requests,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, payload: { day?: number; revision?: number; document?: unknown; published?: boolean }) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: payload.day ?? 0, view_revision: world.revision, people,
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [{
            id: "saved", name: "Find a person", mode: "exact", term: "Alice", limit: 10,
            recorded_at: 3, parameters: [{ name: "exact_1", default: { type: "text", value: "Alice" } }],
          }];
          if (command === "workspace_entry_layout") return savedLayout;
          if (command === "workspace_preview_author_layout") {
            requests.push({ command, payload });
            return { content: { Stack: [
              { content: { Text: "Public biography" } },
              { content: { QueryRows: { records: [[{ world: "archive", id: "beatrice" }, "Beatrice"]], truncated: false } } },
            ] } };
          }
          if (command === "workspace_save_layout") {
            requests.push({ command, payload });
            savedLayout = [payload.document, payload.published];
            world.revision++;
            return null;
          }
          if (command === "workspace_public_entry") return { content: { Text: "Public biography" } };
          if (command === "workspace_cancel_query") return null;
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await graph.getByRole("button", { name: "Open character page" }).click();
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("Public biography");
  await inspector.getByLabel("Private saved query").selectOption("saved@3");
  await inspector.getByLabel("exact 1").fill("Beatrice");
  await inspector.getByRole("button", { name: "Preview author layout" }).click();
  await expect(inspector.getByRole("table", { name: "Private query table" })).toContainText("Beatrice");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-saved-binding_01.png",
      animations: "disabled",
    });
  }
  await inspector.getByLabel("Publish page text").check();
  await inspector.getByRole("button", { name: "Save page" }).click();
  await graph.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByLabel("Private saved query")).toHaveValue("saved@3");
  await expect(inspector.getByLabel("exact 1")).toHaveValue("Beatrice");
  const requests = await page.evaluate(() => (window as unknown as {
    __canvasBindingRequests: { command: string; payload: unknown }[];
  }).__canvasBindingRequests);
  const previews = requests.filter(request => request.command === "workspace_preview_author_layout");
  const saves = requests.filter(request => request.command === "workspace_save_layout");
  expect(previews).toHaveLength(1);
  expect(saves).toHaveLength(1);
  const preview = previews[0].payload as { revision: number; document: { root: { content: { Stack: unknown[] } } } };
  const save = saves[0].payload as { expectedRevision: number; published: boolean; document: typeof preview.document };
  expect(preview.revision).toBe(4);
  expect(save.expectedRevision).toBe(4);
  expect(save.published).toBe(true);
  expect(save.document.root.content.Stack[2]).toEqual(preview.document.root.content.Stack[2]);
  expect(save.document.root.content.Stack[2]).toMatchObject({
    audience: "Author", content: { SavedQuery: { world: "archive", query_id: "saved",
      definition_revision: 3, bindings: [{ name: "exact_1", value: { type: "text", value: "Beatrice" } }] } },
  });
  await inspector.getByRole("button", { name: "View current public page" }).click();
  await expect(inspector.getByText("Public biography", { exact: true })).toBeVisible();
  await expect(inspector.getByRole("table", { name: "Private query table" })).toHaveCount(0);
});

test("authored links save stable targets and reader navigation reauthorizes each page", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "River Archive", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }, { id: "beatrice", label: "Beatrice" }];
    const calls: { command: string; payload: unknown }[] = [];
    let saved: unknown = null;
    let released = true;
    Object.assign(window, {
      isTauri: true, __linkCalls: calls, __withdrawTarget: () => { released = false; },
      __TAURI_INTERNALS__: {
        invoke: async (command: string, payload: { day?: number; entryId?: string; document?: unknown; published?: boolean }) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [world];
          if (command === "workspace_snapshot") return {
            world, day: payload.day ?? 0, view_revision: world.revision, people,
            relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_save_layout") {
            calls.push({ command, payload });
            saved = [payload.document, payload.published];
            world.revision++;
            return null;
          }
          if (command === "workspace_public_entry") {
            calls.push({ command, payload });
            if (payload.entryId === "beatrice") return released ? { content: { Text: "Beatrice publication" } } : null;
            return { content: { Stack: [
              { content: { Text: "Alice publication" } },
              ...(released ? [{ content: { ObjectLink: { target: { world: "archive", id: "beatrice" }, label: "Read Beatrice" } } }] : []),
            ] } };
          }
          if (command === "workspace_public_entry_backlinks" || command === "workspace_author_entry_backlinks") {
            calls.push({ command, payload });
            return { matches: [{ source: { world: "archive", id: "alice" }, element: "link", label: "Read Beatrice" }], truncated: false };
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("Alice publication");
  await inspector.getByRole("button", { name: "Add entry link" }).click();
  await inspector.getByLabel("Link target identity").fill("beatrice");
  await inspector.getByLabel("Link caption").fill("Read Beatrice");
  await inspector.getByLabel("Link audience").selectOption("Public");
  await inspector.getByLabel("Publish page text").check();
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByLabel("Link target identity")).toHaveValue("beatrice");
  await inspector.getByRole("button", { name: "Find authored backlinks" }).click();
  await expect(inspector.getByText("1 authored references.", { exact: true })).toBeVisible();
  await inspector.getByRole("button", { name: "View current public page" }).click();
  await inspector.getByRole("button", { name: "Read Beatrice", exact: true }).click();
  await expect(inspector.getByText("Beatrice publication", { exact: true })).toBeVisible();
  await inspector.getByRole("button", { name: "Find published backlinks" }).click();
  await expect(inspector.getByRole("button", { name: "Referenced by: Read Beatrice" })).toBeVisible();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_links_mock-native-reader-backlinks_01.png",
      animations: "disabled",
    });
  }
  await inspector.getByRole("button", { name: "Back to previous page" }).click();
  await expect(inspector.getByText("Alice publication", { exact: true })).toBeVisible();
  await page.evaluate(() => (window as unknown as { __withdrawTarget: () => void }).__withdrawTarget());
  await inspector.getByRole("button", { name: "Read Beatrice", exact: true }).click();
  await expect(inspector.getByText("Unpublished at this date.", { exact: true })).toBeVisible();
  const calls = await page.evaluate(() => (window as unknown as {
    __linkCalls: { command: string; payload: unknown }[];
  }).__linkCalls);
  const save = calls.find(call => call.command === "workspace_save_layout")!.payload as {
    document: { root: { content: { Stack: unknown[] } } };
  };
  expect(save.document.root.content.Stack[2]).toMatchObject({
    audience: "Public", content: { ObjectLink: { target: { world: "archive", id: "beatrice" }, label: "Read Beatrice" } },
  });
  expect(calls.filter(call => call.command === "workspace_public_entry").map(call => call.payload)).toEqual([
    { worldId: "archive", day: 0, entryId: "alice" },
    { worldId: "archive", day: 0, entryId: "beatrice" },
    { worldId: "archive", day: 0, entryId: "alice" },
    { worldId: "archive", day: 0, entryId: "beatrice" },
  ]);
});

test("event and search-only source entries use the same persistent editor and historical scope", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const calls: { command: string; args: Record<string, unknown> }[] = [];
    const pages = new Map<string, unknown>();
    Object.assign(window, {
      isTauri: true, __generalEntryCalls: calls,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], definitions: [],
            events: [{ id: "meeting", label: "Guild meeting", day: 0, citation: null, participants: [], actions: [] }],
          };
          if (command === "workspace_saved_queries" || command === "workspace_evidence") return [];
          if (command === "workspace_search") return {
            revision: args.revision, truncated: false,
            rows: [{ id: "folio", label: "Guild minutes", kind: "source" }],
          };
          if (command === "workspace_entry_layout") {
            calls.push({ command, args });
            return pages.get(String(args.entryId)) ?? null;
          }
          if (command === "workspace_save_layout") {
            calls.push({ command, args });
            const document = args.document as { entry: { id: string } };
            pages.set(document.entry.id, [document, args.published]);
            world.revision++;
            return null;
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=meeting]").click();
  await inspector.getByRole("button", { name: "Open event page" }).click();
  await expect(inspector.getByRole("heading", { name: "Entry page · Guild meeting · event" })).toBeVisible();
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("The guild convened.");
  await inspector.getByRole("button", { name: "Add entry link" }).click();
  await expect(inspector.locator('datalist option[value="meeting"]')).toHaveCount(1);
  await inspector.getByLabel("Link target identity").fill("folio");
  await inspector.getByLabel("Link caption").fill("Council source");
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open event page" }).click();
  await expect(inspector.getByRole("textbox", { name: "Page text", exact: true })).toHaveValue("The guild convened.");
  const query = page.locator("#author-query-form");
  const results = page.locator("#author-query-results");
  await query.getByLabel("Find name").fill("Guild minutes");
  await query.getByRole("button", { name: "Search history" }).click();
  await results.getByRole("button", { name: "Open entry page" }).click();
  await expect(results.getByRole("heading", { name: "Entry page · Guild minutes" })).toBeVisible();
  await results.getByRole("textbox", { name: "Page text", exact: true }).fill("Folio 14.");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await results.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-source-entry_01.png",
      animations: "disabled",
    });
  }
  await results.getByRole("button", { name: "Save page" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 6 / 6");
  const revision = page.locator("#author-revision");
  await revision.fill("4");
  await revision.blur();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 4 / 6");
  await query.getByRole("button", { name: "Search history" }).click();
  await results.getByRole("button", { name: "Open entry page" }).click();
  await expect(results.getByRole("button", { name: "Save page" })).toBeDisabled();
  const calls = await page.evaluate(() => (window as unknown as {
    __generalEntryCalls: { command: string; args: Record<string, unknown> }[];
  }).__generalEntryCalls);
  expect(calls.filter(call => call.command === "workspace_save_layout").map(call => call.args)).toMatchObject([
    { worldId: "archive", expectedRevision: 4, document: { entry: { world: "archive", id: "meeting" } } },
    { worldId: "archive", expectedRevision: 5, document: { entry: { world: "archive", id: "folio" } } },
  ]);
  expect(calls.at(-1)).toMatchObject({
    command: "workspace_entry_layout", args: { worldId: "archive", entryId: "folio", revision: 4, day: 0 },
  });
});

test("Canvas arrangement changes preserve content IDs across reopen and public projection", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: { root: { content: { Stack: unknown[] } } }[] = [];
    let saved: unknown = null;
    Object.assign(window, {
      isTauri: true, __layoutWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_save_layout") {
            writes.push(args.document as typeof writes[number]);
            saved = [args.document, args.published];
            world.revision++;
            return null;
          }
          if (command === "workspace_public_entry") return { content: { Stack: [
            { content: { Panel: [{ content: { Text: "Public biography" } }] } },
          ] } };
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await inspector.getByLabel("Page arrangement").selectOption("2");
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("Public biography");
  await inspector.getByRole("textbox", { name: "Author notes" }).fill("Secret notes");
  await inspector.getByLabel("Publish page text").check();
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByLabel("Page arrangement")).toHaveValue("2");
  await inspector.getByLabel("Page arrangement").selectOption("panel");
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByLabel("Page arrangement")).toHaveValue("panel");
  await inspector.getByRole("button", { name: "View current public page" }).click();
  await expect(inspector.locator("[data-public-result] .entry-panel")).toHaveText("Public biography");
  await expect(inspector.locator("[data-public-result]")).not.toContainText("Secret notes");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-panel_01.png",
      animations: "disabled",
    });
  }
  const writes = await page.evaluate(() => (window as unknown as {
    __layoutWrites: { root: { content: { Stack: {
      id: string; content: { Grid?: { columns: number; children: unknown[] }; Panel?: unknown[] }
    }[] } } }[];
  }).__layoutWrites);
  const grid = writes[0].root.content.Stack[0];
  const panel = writes[1].root.content.Stack[0];
  expect(grid.content.Grid?.columns).toBe(2);
  expect(panel.id).toBe(grid.id);
  expect(panel.content.Panel).toEqual(grid.content.Grid?.children);
});

test("Canvas tree nests and moves stable components without widening inherited audience", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: unknown[] = [];
    let saved: unknown = [{
      schema_version: 1, entry: { world: "archive", id: "alice" },
      root: { id: "root", audience: "Public", content: { Stack: [
        { id: "private-panel", audience: "Author", content: { Panel: [
          { id: "secret", audience: "Public", content: { Text: "Secret testimony" } },
          { id: "binding", audience: "Author", content: { SavedQuery: {
            world: "archive", query_id: "query", definition_revision: 2, bindings: [],
          } } },
        ] } },
        { id: "public-grid", audience: "Public", content: { Grid: { columns: 2, children: [] } } },
      ] } },
    }, false];
    Object.assign(window, {
      isTauri: true, __treeWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_save_layout") {
            writes.push(args.document);
            saved = [args.document, args.published];
            world.revision++;
            return null;
          }
          if (command === "workspace_preview_author_layout") return { content: { Text: "Compiled preview" } };
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByRole("heading", { name: "Canvas tree" })).toBeVisible();
  await inspector.getByRole("button", { name: "Text · Secret testimony" }).click();
  await inspector.getByLabel("Move into container").selectOption("public-grid");
  await inspector.getByRole("button", { name: "Move component", exact: true }).click();
  await expect(inspector.getByLabel("Component audience")).toHaveValue("Author");
  await inspector.getByRole("button", { name: "SavedQuery · binding" }).click();
  await inspector.getByLabel("Move into container").selectOption("public-grid");
  await inspector.getByRole("button", { name: "Move component", exact: true }).click();
  await inspector.getByRole("button", { name: "Grid · public-g" }).click();
  await inspector.getByLabel("New component kind").selectOption("Panel");
  await inspector.getByRole("button", { name: "Add child" }).click();
  await inspector.getByLabel("New component kind").selectOption("Text");
  await inspector.getByRole("button", { name: "Add child" }).click();
  await inspector.getByLabel("Component text").fill("New nested text");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-tree_01.png",
      animations: "disabled",
    });
  }
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByRole("button", { name: "Text · New nested text" })).toBeVisible();
  const writes = await page.evaluate(() => (window as unknown as { __treeWrites: unknown[] }).__treeWrites);
  expect(writes).toMatchObject([{ root: { content: { Stack: [
    { id: "private-panel", content: { Panel: [] } },
    { id: "public-grid", content: { Grid: { columns: 2, children: [
      { id: "secret", audience: "Author", content: { Text: "Secret testimony" } },
      { id: "binding", audience: "Author", content: { SavedQuery: {
        query_id: "query", definition_revision: 2, bindings: [],
      } } },
      { audience: "Author", content: { Panel: [
        { audience: "Author", content: { Text: "New nested text" } },
      ] } },
    ] } } },
  ] } } }]);
  const revision = page.locator("#author-revision");
  await revision.fill("4");
  await revision.blur();
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByRole("button", { name: "Save page" })).toBeDisabled();
});

test("Canvas template instances and multiple query bindings save together without rewriting earlier pins", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: unknown[] = [];
    let saved: unknown = null;
    Object.assign(window, {
      isTauri: true, __templateWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [
            { id: "query", name: "People", mode: "contains", term: "Alice", limit: 20, recorded_at: 2,
              parameters: [{ name: "name", default: { type: "text", value: "Alice" } }] },
          ];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_save_layout") {
            writes.push(args.document);
            saved = [args.document, args.published];
            world.revision++;
            return null;
          }
          if (command === "workspace_preview_author_layout") return { content: { Text: "Compiled preview" } };
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("Biography");
  await inspector.getByRole("button", { name: "Open Canvas tree" }).click();
  const root = inspector.getByRole("button", { name: /^Stack ·/ }).first();
  await inspector.getByLabel("Canvas template").selectOption("character");
  await inspector.getByRole("button", { name: "Insert template" }).click();
  await expect(inspector.getByText("Template: character · version 1")).toBeVisible();
  await root.click();
  await inspector.getByLabel("New component kind").selectOption("SavedQuery");
  await inspector.getByRole("button", { name: "Add child" }).click();
  await inspector.getByLabel("Query parameter name").fill("First");
  await root.click();
  await inspector.getByLabel("New component kind").selectOption("SavedQuery");
  await inspector.getByRole("button", { name: "Add child" }).click();
  await inspector.getByLabel("Query parameter name").fill("Second");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-template-query_01.png",
      animations: "disabled",
    });
  }
  await inspector.getByRole("button", { name: "Save page" }).click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByRole("button", { name: "Template · character" })).toBeVisible();
  const writes = await page.evaluate(() => (window as unknown as { __templateWrites: unknown[] }).__templateWrites);
  expect(writes).toMatchObject([{ root: { content: { Stack: [
    { content: { Text: "Biography" } }, { content: { Text: "" } },
    { audience: "Author", content: { Template: { key: "character", version: 1 } } },
    { audience: "Author", content: { SavedQuery: { definition_revision: 2, bindings: [
      { name: "name", value: { type: "text", value: "First" } },
    ] } } },
    { audience: "Author", content: { SavedQuery: { definition_revision: 2, bindings: [
      { name: "name", value: { type: "text", value: "Second" } },
    ] } } },
  ] } } }]);
});

test("institution studio records a sourced fact and reviews a separate draft", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: { command: string; args: unknown }[] = [];
    Object.assign(window, {
      isTauri: true, __networkWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_network_facts") return [{
            id: "fact", input: { kind: "membership", role: "Treasurer" },
            subject_label: "Alice", target_label: "Guild",
          }];
          if (command === "workspace_ghost_proposals") return [{
            id: "proposal", claim: {world: "archive", id: "claim"}, label: "New Society",
            kind: "organization", reviewed: false, promoted: null,
          }];
          if (["workspace_create_identity", "workspace_record_network_fact", "workspace_propose_ghost", "workspace_review_ghost"].includes(command)) {
            writes.push({ command, args }); world.revision++; return null;
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const studio = page.locator("[data-network-studio]");
  await studio.locator("summary").click();
  await studio.getByLabel("Identity label").fill("Guild");
  await studio.getByRole("button", { name: "Create identity" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 5 / 5");
  await studio.locator("summary").click();
  await studio.getByLabel("Subject identity").fill("alice");
  await studio.getByLabel("Target identity").fill("guild");
  await studio.getByLabel("Evidence source identity").fill("folio");
  await studio.getByLabel("Role or causal rationale").fill("Treasurer");
  await studio.getByRole("button", { name: "Record network fact" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 6 / 6");
  await studio.locator("summary").click();
  await studio.getByLabel("Network focus identity").fill("guild");
  await studio.getByRole("button", { name: "Inspect network facts" }).click();
  await expect(studio.getByText(/Alice → Guild · membership · Treasurer/)).toBeVisible();
  await studio.getByRole("button", { name: "Review draft identities" }).click();
  await expect(studio.getByText(/New Society · organization/)).toBeVisible();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await studio.screenshot({ path: "../../Development-Screenshots/" + timestamp + "_phase-05_network_mock-native-institution-ghost_01.png", animations: "disabled" });
  }
  await studio.getByRole("button", { name: "Promote draft" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 7 / 7");
  const writes = await page.evaluate(() => (window as unknown as { __networkWrites: unknown[] }).__networkWrites);
  expect(writes).toMatchObject([
    {command: "workspace_create_identity", args: {request: {revision: 4}, kind: "organization", label: "Guild"}},
    {command: "workspace_record_network_fact", args: {request: {revision: 5}, input: {kind: "membership", source: {id: "folio"}}}},
    {command: "workspace_review_ghost", args: {request: {revision: 6}, proposalId: "proposal", promote: true}},
  ]);
  await studio.locator("summary").click();
  await studio.getByLabel("Identity type").selectOption("place");
  await studio.getByLabel("Identity label").fill("Northbridge Manor");
  await studio.getByRole("button", { name: "Create identity" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 8 / 8");
  await studio.locator("summary").click();
  await studio.getByLabel("Subject identity").fill("manor");
  await studio.getByLabel("Target identity").fill("town");
  await studio.getByLabel("Evidence source identity").fill("folio");
  await studio.getByLabel("Fact type").selectOption("proper_part_within");
  await studio.getByLabel("Role or causal rationale").fill("physical");
  await expect(studio.getByLabel("Fact type")).toHaveValue("proper_part_within");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await studio.locator("[data-fact]").screenshot({ path: "../../Development-Screenshots/" + timestamp + "_phase-03_spatial_mock-native-containment-form_01.png", animations: "disabled" });
  }
  await studio.getByRole("button", { name: "Record network fact" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 9 / 9");
  const spatialWrites = await page.evaluate(() => (window as unknown as { __networkWrites: unknown[] }).__networkWrites.slice(3));
  expect(spatialWrites).toMatchObject([
    {command: "workspace_create_identity", args: {request: {revision: 7}, kind: "place", label: "Northbridge Manor"}},
    {command: "workspace_record_network_fact", args: {request: {revision: 8}, input: {kind: "proper_part_within", role: "physical", subject: {id: "manor"}, target: {id: "town"}}}}
  ]);
});

test("custom template insertion uses exact pin and Canvas drag and undo preserve privacy", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: unknown[] = [];
    let saved: unknown = [{
      schema_version: 1, entry: { world: "archive", id: "alice" },
      root: { id: "root", audience: "Public", content: { Stack: [
        { id: "private-panel", audience: "Author", content: { Panel: [
          { id: "secret", audience: "Public", content: { Text: "Secret testimony" } },
        ] } },
        { id: "public-grid", audience: "Public", content: { Grid: { columns: 2, children: [] } } },
      ] } },
    }, false];
    Object.assign(window, {
      isTauri: true, __customTemplateWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [{ id: "alice", label: "Alice" }], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_custom_templates") return [{id: "template", name: "Family profile", revision: 3}];
          if (command === "workspace_instantiate_custom_template") {
            writes.push(args);
            return {id: "instance", audience: "Author", content: {CustomTemplate: {
              template_id: "template", definition_revision: 3, children: [{id: "new-root", audience: "Public", content: {Stack: []}}],
            }}};
          }
          if (command === "workspace_save_layout") {
            writes.push(args.document);
            saved = [args.document, args.published];
            world.revision++;
            return null;
          }
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(inspector.getByRole("heading", { name: "Canvas tree" })).toBeVisible();
  const text = inspector.getByRole("button", { name: "Text · Secret testimony" });
  const grid = inspector.getByRole("button", { name: "Grid · public-g" });
  await text.dragTo(grid);
  await expect(inspector.getByLabel("Component audience")).toHaveValue("Author");
  await inspector.getByRole("button", { name: "Undo Canvas edit" }).click();
  await text.click();
  await expect(inspector.getByLabel("Component audience")).toHaveValue("Public");
  await inspector.getByRole("button", { name: "Redo Canvas edit" }).click();
  await text.click();
  await expect(inspector.getByLabel("Component audience")).toHaveValue("Author");
  await grid.click();
  await inspector.getByRole("button", { name: "Load custom templates" }).click();
  await inspector.getByLabel("Custom template", { exact: true }).selectOption("template");
  await inspector.getByRole("button", { name: "Insert custom template" }).click();
  await expect(inspector.getByText("Custom template revision 3 · template")).toBeVisible();
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-custom-template_01.png", animations: "disabled"});
  }
  await inspector.getByRole("button", { name: "Save page" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 5 / 5");
  const writes = await page.evaluate(() => (window as unknown as { __customTemplateWrites: unknown[] }).__customTemplateWrites);
  expect(writes).toMatchObject([
    {request: {revision: 4}, templateId: "template", definitionRevision: 3, entryId: "alice"},
    {root: {content: {Stack: [
      {id: "private-panel", content: {Panel: []}},
      {id: "public-grid", content: {Grid: {children: [
        {id: "secret", audience: "Author"}, {id: "instance", content: {CustomTemplate: {template_id: "template", definition_revision: 3}}},
      ]}}},
    ]}}},
  ]);
});

test("imported PNG and scoped hotspots edit in Canvas and public projection navigates", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: unknown[] = [];
    let saved: unknown = null;
    Object.assign(window, {
      isTauri: true, __mediaWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {world:{...world},day:args.day,view_revision:args.revision??world.revision,people:[{id:"alice",label:"Alice"},{id:"bob",label:"Bob"}],relationships:[],events:[],definitions:[]};
          if (command === "workspace_saved_queries") return [];
          if (command === "workspace_media_assets") return [{id:"asset",name:"Archive illustration",width:240,height:120,revision:3}];
          if (command === "workspace_entry_layout") return saved;
          if (command === "workspace_save_layout") {writes.push(args.document);saved=[args.document,args.published];world.revision++;return null;}
          if (command === "workspace_public_entry") {
            if(args.entryId==="bob")return {content:{Text:"Released companion page"}};
            const canvas=document.createElement("canvas");canvas.width=240;canvas.height=120;
            const painter=canvas.getContext("2d")!;
            painter.fillStyle="#d1bca0";painter.fillRect(0,0,240,120);painter.fillStyle="#496657";painter.fillRect(24,20,70,80);painter.fillStyle="#725840";painter.fillRect(146,20,70,80);
            return {content:{Image:{width:240,height:120,png_base64:canvas.toDataURL("image/png").split(",")[1],alt:"Guild portrait",hotspots:[{id:"public-hotspot",x:2500,y:5000,link:{target:{world:"archive",id:"bob"},label:"Companion"}}]}}};
          }
          if(command==="workspace_public_entry_backlinks")return {matches:[],truncated:false};
          throw new Error("Unexpected mock command: "+command);
        },
      },
    });
  });
  await page.goto("/");
  const graph=page.getByRole("region",{name:"Saved world graph"});
  const inspector=graph.getByRole("complementary",{name:"Node inspector"});
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button",{name:"Open character page"}).click();
  await inspector.getByRole("textbox",{name:"Page text",exact:true}).fill("Guild biography");
  await inspector.getByRole("button",{name:"Open Canvas tree"}).click();
  await inspector.getByLabel("New component kind").selectOption("Image");
  await inspector.getByRole("button",{name:"Add child"}).click();
  await inspector.getByRole("button",{name:"Choose imported image"}).click();
  await inspector.getByLabel("Imported PNG").selectOption("asset");
  await inspector.getByLabel("Image alternative text").fill("Guild portrait");
  await inspector.getByLabel("Component audience").selectOption("Public");
  await inspector.getByRole("button",{name:"Add private hotspot"}).click();
  await inspector.getByLabel("Hotspot label 1").fill("Companion");
  await inspector.getByLabel("Hotspot target 1").fill("bob");
  await inspector.getByLabel("Hotspot x percent 1").fill("25");
  await inspector.getByLabel("Hotspot audience 1").selectOption("Public");
  await inspector.getByRole("button",{name:"Add private hotspot"}).click();
  await inspector.getByLabel("Hotspot label 2").fill("Private associate");
  await inspector.getByLabel("Hotspot target 2").fill("bob");
  await inspector.getByLabel("Publish page text").check();
  await inspector.getByRole("button",{name:"Save page"}).click();
  const writes=await page.evaluate(()=>(window as unknown as {__mediaWrites:unknown[]}).__mediaWrites);
  expect(writes).toMatchObject([{root:{content:{Stack:[{content:{Text:"Guild biography"}},{content:{Text:""}},{audience:"Public",content:{Image:{asset:"asset",alt:"Guild portrait",hotspots:[{x:2500,audience:"Public",link:{label:"Companion"}},{audience:"Author",link:{label:"Private associate"}}]}}}]}}}]);
  await inspector.getByRole("button",{name:"Open character page"}).click();
  await inspector.getByRole("button",{name:"View current public page"}).click();
  await expect(inspector.getByRole("img",{name:"Guild portrait"})).toBeVisible();
  await expect(inspector.getByRole("button",{name:"Private associate",exact:true})).toHaveCount(0);
  if(process.env.ATLAS_CAPTURE_PHASE3==="1"){
    const timestamp=new Date().toISOString().slice(0,19).replace("T","_").replaceAll(":","-")+"Z";
    await inspector.screenshot({path:"../../Development-Screenshots/"+timestamp+"_phase-04_canvas_mock-native-image-hotspots_01.png",animations:"disabled"});
  }
  await inspector.locator(".image-hotspot").click();
  await expect(inspector.getByText("Released companion page")).toBeVisible();
});

test("classification authoring and evidence backlinks use historical author commands", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    const writes: unknown[] = [];
    const fact = {
      reference: { world: "archive", family: "classification", id: "classification" },
      subject: { world: "archive", id: "language" }, target: null,
      source: { world: "archive", id: "folio" }, descriptor: { world: "archive", id: "tag" },
      description: "Language classification", rationale: "Record concept classification",
      recorded_at: 5, source_branch: "main", descriptor_revision: 3,
      descriptor_state_revision: 2, descriptor_branch: "main", corrects: null,
    };
    Object.assign(window, {
      isTauri: true, __classificationWrites: writes,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
          if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
          if (command === "workspace_worlds") return [{ ...world }];
          if (command === "workspace_snapshot") return {
            world: { ...world }, day: args.day, view_revision: args.revision ?? world.revision,
            people: [], relationships: [], events: [], definitions: [],
          };
          if (command === "workspace_saved_queries") return [];
          if (["workspace_record_classification", "workspace_withdraw_classification", "workspace_annotate_semantic_fact", "workspace_retire_semantic_descriptor"].includes(command)) {
            writes.push({ command, args }); world.revision++; return null;
          }
          if (command === "workspace_classifications") return [{
            id: "classification", profile: "language", descriptor_id: "tag", descriptor_revision: 3,
            source_id: "folio", starts_at: 0, ends_before: null, recorded_at: 5,
          }];
          if (command === "workspace_semantic_fact_backlinks") return [fact];
          if (command === "workspace_semantic_fact") {
            if ((args.reference as { family: string }).family === "site_area") {
              return { ...fact, reference: { world: "archive", family: "site_area", id: "area" },
                subject: { world: "archive", id: "workshop" }, description: "Site area: 12 ha", corrects: "prior-area" };
            }
            return fact;
          }
          if (command === "workspace_semantic_fact_annotations") return [{
            id: "note", claim: { world: "archive", id: "claim" }, recorded_at: 6,
          }];
          throw new Error("Unexpected mock command: " + command);
        },
      },
    });
  });
  await page.goto("/");
  const studio = page.locator("[data-network-studio]");
  await studio.locator("summary").click();
  const form = studio.locator("[data-classification]");
  await form.getByLabel("Concept identity").fill("language");
  await form.getByLabel("Semantic profile").selectOption("language");
  await form.getByLabel("Tag definition identity").fill("tag");
  await form.getByLabel("Classification evidence source").fill("folio");
  await form.getByRole("button", { name: "Record classification" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 5 / 5");
  await studio.locator("summary").click();
  await studio.getByLabel("Classification focus concept").fill("language");
  await studio.getByRole("button", { name: "Inspect classifications" }).click();
  await expect(studio.locator("[data-classifications]")).toContainText("revision 3");
  await studio.getByLabel("Evidence or identity backlinks").fill("folio");
  await studio.getByRole("button", { name: "Inspect semantic backlinks" }).click();
  await expect(studio.locator("[data-semantic-links]")).toContainText("Language classification");
  await studio.getByRole("button", { name: "Resolve classification fact" }).click();
  await expect(studio.locator("[data-semantic-detail]")).toContainText("evidence folio");
  await expect(studio.locator("[data-semantic-detail]")).toContainText("descriptor tag @ revision 3 (state 2, branch main)");
  await expect(studio.locator("[data-semantic-detail]")).toContainText("recorded on branch main");
  await expect(studio.locator("[data-semantic-detail]")).not.toContainText("corrects fact");
  await studio.locator("[data-semantic-fact]").getByRole("combobox").selectOption("site_area");
  await studio.getByLabel("Stable fact identity").fill("area");
  await studio.getByRole("button", { name: "Resolve fact", exact: true }).click();
  await expect(studio.locator("[data-semantic-detail]")).toContainText("corrects fact prior-area");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await studio.locator("[data-semantic-detail]").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_semantic_mock-native-annotation-provenance_01.png",
      animations: "disabled",
    });
  }
  await studio.getByRole("button", { name: "Select fact for annotation" }).click();
  await expect(studio.getByLabel("Annotation fact UUID")).toHaveValue("classification");
  await studio.getByLabel("Sourced claim UUID").fill("claim");
  await studio.getByRole("button", { name: "Link sourced annotation" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 6 / 6");
  await studio.locator("summary").click();
  await studio.getByLabel("Annotation fact family").selectOption("classification");
  await studio.getByLabel("Annotation fact UUID").fill("classification");
  await studio.getByRole("button", { name: "Inspect annotations" }).click();
  await expect(studio.locator("[data-annotations]")).toContainText("Claim claim");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await studio.locator("[data-fact-annotation]").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_semantic_mock-native-annotation-form_01.png",
      animations: "disabled",
    });
    await studio.locator("[data-annotations]").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_semantic_mock-native-annotation-list_02.png",
      animations: "disabled",
    });
  }
  await studio.getByLabel("Retire Tag definition UUID").fill("tag");
  await studio.getByLabel("Confirm retirement").check();
  await studio.getByRole("button", { name: "Retire descriptor", exact: true }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 7 / 7");
  await studio.locator("summary").click();
  await studio.getByLabel("Classification focus concept").fill("language");
  await studio.getByRole("button", { name: "Inspect classifications" }).click();
  await studio.getByRole("button", { name: "Withdraw classification" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 8 / 8");
  const writes = await page.evaluate(() => (window as unknown as { __classificationWrites: unknown[] }).__classificationWrites);
  expect(writes).toMatchObject([
    { command: "workspace_record_classification", args: { request: { revision: 4 },
      input: { subject_id: "language", descriptor_id: "tag", source_id: "folio", profile: "language" } } },
    { command: "workspace_annotate_semantic_fact", args: { request: { revision: 5 },
      fact: { world: "archive", family: "classification", id: "classification" }, claimId: "claim" } },
    { command: "workspace_retire_semantic_descriptor", args: { request: { revision: 6 }, descriptorId: "tag" } },
    { command: "workspace_withdraw_classification", args: { request: { revision: 7 }, factId: "classification" } },
  ]);
});

test("semantic inspection invalidates pending replies and stale actions when focus changes", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "archive", name: "Archive", revision: 4 };
    let release: (() => void) | undefined;
    Object.assign(window, {
      isTauri: true, __releaseSemantic: () => release?.(), __semanticPending: false,
      __selectionRequests: [] as Record<string, unknown>[],
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown>) => {
          switch (command) {
            case "platform_scene": return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
            case "northbridge_history": return { day: 0, relationships: [], cause: null };
            case "workspace_worlds": return [world];
            case "workspace_snapshot": return { world, day: args.day, view_revision: 4,
              people: [], relationships: [], events: [], definitions: [] };
            case "workspace_saved_queries": return [];
            case "workspace_semantic_candidates": {
              (window as unknown as { __selectionRequests: Record<string, unknown>[] }).__selectionRequests.push(args);
              if (args.kind === "culture") return { rows: [{id:"culture",label:"Estuary Culture"}], truncated:false };
              return { rows: [{id:"new",label:"Estuary Speech"}], truncated:false };
            }
            case "workspace_classifications": {
              if (args.subjectId === "old") {
                await new Promise<void>(resolve => {
                  release = resolve;
                  Object.assign(window, { __semanticPending: true });
                });
              }
              if (args.subjectId === "old") Object.assign(window, { __semanticReleased: true });
              return [{ id: String(args.subjectId), profile: String(args.subjectId), descriptor_id: "tag",
                descriptor_revision: 3, source_id: "folio", starts_at: 0, ends_before: null, recorded_at: 4 }];
            }
            default: throw new Error("Unexpected mock command: " + command);
          }
        },
      },
    });
  });
  await page.goto("/");
  const studio=page.locator("[data-network-studio]");
  await studio.locator("summary").click();
  const focus=studio.getByLabel("Classification focus concept");
  const inspect=studio.getByRole("button", { name: "Inspect classifications" });
  const results=studio.locator("[data-classifications]");
  await focus.fill("old");
  await inspect.click();
  await expect.poll(() => page.evaluate(() => (window as unknown as { __semanticPending: boolean }).__semanticPending)).toBe(true);
  await studio.getByLabel("Selection destination").selectOption({label:"Classification inspector"});
  await studio.getByLabel("Identity name or phrase").fill("Estuary");
  await studio.getByRole("button", {name:"Find compatible identities",exact:true}).click();
  await studio.getByRole("button", {name:"Estuary Speech · new",exact:true}).click();
  await expect(focus).toHaveValue("new");
  await inspect.click();
  await expect(results).toContainText("new");
  await page.evaluate(() => (window as unknown as { __releaseSemantic: () => void }).__releaseSemantic());
  await expect.poll(() => page.evaluate(() => (window as unknown as { __semanticReleased?: boolean }).__semanticReleased)).toBe(true);
  await expect(results).toContainText("new");
  await expect(results).not.toContainText("old");
  await focus.fill("different");
  await expect(results).toBeEmpty();
  await expect(results.getByRole("button", { name: "Withdraw classification" })).toHaveCount(0);

  const fact = studio.locator("[data-fact]");
  await fact.getByLabel("Fact type").selectOption("culture_language");
  await fact.getByLabel("Starts at day").fill("7");
  await studio.getByLabel("Selection destination").selectOption({label:"Semantic network target"});
  await studio.getByLabel("Identity name or phrase").fill("Estuary");
  await studio.getByRole("button", {name:"Find compatible identities",exact:true}).click();
  const language = studio.getByRole("button", {name:"Estuary Speech · new",exact:true});
  await expect(language).toBeVisible();
  const requests = await page.evaluate(() =>
    (window as unknown as { __selectionRequests: Record<string, unknown>[] }).__selectionRequests);
  expect(requests.at(-1)).toMatchObject({kind:"language", phrase:"Estuary", request:{world_id:"archive",revision:4,day:7}});
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-").replace(/-\d{3}Z$/, "Z").replace("T", "_");
    await studio.locator("[data-semantic-picker]").screenshot({
      path: "../../Development-Screenshots/" + timestamp + "_phase-03_semantic_mock-native-profile-picker_01.png",
      animations: "disabled",
    });
  }
  // Changing the effective day makes the old result unusable.
  await fact.getByLabel("Starts at day").fill("8");
  await expect(language).toHaveCount(0);
  await studio.getByRole("button", {name:"Find compatible identities",exact:true}).click();
  await language.click();
  await expect(fact.getByLabel("Target identity")).toHaveValue("new");

  await studio.getByLabel("Selection destination").selectOption({label:"Semantic network subject"});
  await studio.getByRole("button", {name:"Find compatible identities",exact:true}).click();
  await studio.getByRole("button", {name:"Estuary Culture · culture",exact:true}).click();
  await expect(fact.getByLabel("Subject identity")).toHaveValue("culture");
  await fact.getByLabel("Fact type").selectOption("membership");
  await studio.getByRole("button", {name:"Find compatible identities",exact:true}).click();
  await expect(studio.locator("[data-status]")).toContainText("Select Culture documented language");
  await expect(studio.locator("[data-picker-results]")).toBeEmpty();
});


test("creation commands use the loaded author revision and reject a changed day before refresh", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "guard-world", name: "Revision guard", revision: 7 };
    const people = [{ id: "alice", label: "Alice" }, { id: "beatrice", label: "Beatrice" }];
    const writes: { command: string; args: unknown }[] = [];
    Object.assign(window, { isTauri: true, __creationWrites: writes, __TAURI_INTERNALS__: {
      invoke: async (command: string, args: Record<string, unknown>) => {
        if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
        if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
        if (command === "workspace_worlds") return [world];
        if (command === "workspace_saved_queries") return [];
        if (command === "workspace_snapshot") return { world, day: args.day, view_revision: world.revision, people, relationships: [], events: [] };
        if (command === "workspace_create_person" || command === "workspace_create_relationship") {
          writes.push({ command, args });
          world.revision++;
          return null;
        }
        throw new Error("Unexpected mock command: " + command);
      },
    } });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("World loaded");
  const person = page.locator("#author-create-person");
  await person.locator('[name="name"]').fill("Cora");
  await page.evaluate(() => {
    document.querySelector<HTMLInputElement>("#author-day")!.value = "10";
    document.querySelector<HTMLFormElement>("#author-create-person")!.requestSubmit();
  });
  await expect(page.locator("#author-status")).toContainText("Refresh the selected historical view");
  const writes = () => page.evaluate(() => (window as unknown as { __creationWrites: { command: string; args: unknown }[] }).__creationWrites);
  expect(await writes()).toEqual([]);
  await page.locator("#author-refresh-view").click();
  await expect(page.locator("#author-world-info")).toContainText("historical day 10");
  await person.getByRole("button", { name: "Add person" }).click();
  await expect(page.locator("#author-status")).toContainText("Saved to local world history");
  expect((await writes())[0]).toEqual({ command: "workspace_create_person", args: { worldId: "guard-world", expectedRevision: 7, name: "Cora" } });
  await page.locator("#author-create-relationship button[type=submit]").click();
  await expect(page.locator("#author-status")).toContainText("Saved to local world history");
  expect((await writes())[1]).toMatchObject({ command: "workspace_create_relationship", args: { request: { world_id: "guard-world", expected_revision: 8, day: 10 } } });
  const kinds = await page.locator('#author-composite-form select[name="kind"] option').evaluateAll(options => options.map(option => (option as HTMLOptionElement).value));
  expect(kinds).toEqual(["","person","concept","place","organization","household","family_house","faction","conflict","office","title","asset","population","conflict_issue","conflict_phase","succession_rule","definition","relationship","event_involvement","actor_action","historical_event","source","claim"]);
});


test("entry draft blocks view changes, retains a failed save and can be explicitly discarded", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "draft-world", name: "Draft safety", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }, { id: "beatrice", label: "Beatrice" }];
    let saves = 0;
    let release: (() => void) | undefined;
    let saved: unknown = null;
    Object.assign(window, { isTauri: true, __resolveDraftSave: () => release?.(), __TAURI_INTERNALS__: {
      invoke: async (command: string, args: { day?: number; revision?: number; document?: unknown; published?: boolean }) => {
        if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
        if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
        if (command === "workspace_worlds") return [world];
        if (command === "workspace_saved_queries") return [];
        if (command === "workspace_snapshot") return { world, day: args.day ?? 0, view_revision: args.revision ?? world.revision, people, relationships: [], events: [] };
        if (command === "workspace_entry_layout") return saved;
        if (command === "workspace_save_layout") {
          saves++;
          if (saves === 1) throw new Error("Simulated stale revision");
          await new Promise<void>(resolve => { release = resolve; });
          saved = [args.document, args.published]; world.revision++;
          return null;
        }
        throw new Error("Unexpected mock command: " + command);
      },
    } });
  });
  await page.goto("/");
  await expect(page.locator("#author-status")).toContainText("World loaded");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await graph.locator("#nodes button[data-node=alice]").click();
  await inspector.getByRole("button", { name: "Open character page" }).click();
  const text = inspector.getByRole("textbox", { name: "Page text", exact: true });
  await text.fill("Retain this draft");
  await graph.locator("#nodes button[data-node=beatrice]").click();
  await expect(text).toHaveValue("Retain this draft");
  await expect(inspector.locator("[data-draft-status]")).toContainText("Save or discard");
  const day = page.locator("#author-day");
  await day.fill("42");
  await day.blur();
  await expect(day).toHaveValue("0");
  await expect(text).toHaveValue("Retain this draft");
  await inspector.getByRole("button", { name: "Save page" }).click();
  await expect(inspector.locator("[data-draft-status]")).toContainText("Save failed");
  await expect(text).toHaveValue("Retain this draft");
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({ path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-draft-guard_01.png", animations: "disabled" });
  }
  await inspector.getByRole("button", { name: "Discard unsaved page changes" }).click();
  await expect(text).toHaveValue("");
  await text.fill("Save this instead");
  await inspector.getByRole("button", { name: "Save page" }).click();
  await expect(inspector.locator("[data-draft-status]")).toHaveText("Saving page…");
  await graph.locator("#nodes button[data-node=beatrice]").click();
  await expect(inspector.locator("[data-draft-status]")).toContainText("Wait for the result");
  await page.evaluate(() => (window as unknown as { __resolveDraftSave: () => void }).__resolveDraftSave());
  await expect(page.locator("#author-world-info")).toContainText("Author revision 5 / 5");
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await expect(text).toHaveValue("Save this instead");
  await graph.locator("#nodes button[data-node=beatrice]").click();
  await expect(inspector.getByRole("button", { name: "Open character page" })).toBeVisible();
});


test("Canvas reference picker uses the pinned ontology search, preserves captions and ignores late results", async ({ page }) => {
  await page.addInitScript(() => {
    const world = { id: "reference-world", name: "Reference selection", revision: 4 };
    const people = [{ id: "alice", label: "Alice" }];
    const calls: { command: string; args: unknown }[] = [];
    let finishOld: (() => void) | undefined;
    Object.assign(window, { isTauri: true, __referenceCalls: calls, __finishOldReference: () => finishOld?.(), __TAURI_INTERNALS__: {
      invoke: async (command: string, args: { day?: number; phrase?: string }) => {
        if (command === "platform_scene") return { target: "test-native", smoke: false, scene: { nodes: [], edges: [] } };
        if (command === "northbridge_history") return { day: 0, relationships: [], cause: null };
        if (command === "workspace_worlds") return [world];
        if (command === "workspace_saved_queries") return [];
        if (command === "workspace_snapshot") return { world, day: args.day ?? 0, view_revision: world.revision, people, relationships: [], events: [] };
        if (command === "workspace_entry_layout") return null;
        if (command === "workspace_cancel_query") { calls.push({ command, args }); return null; }
        if (command === "workspace_semantic_candidates") {
          calls.push({ command, args });
          if (args.phrase === "Old") {
            await new Promise<void>(resolve => { finishOld = resolve; });
            return { rows: [{ id: "old", label: "Obsolete", kind: "concept" }], truncated: false };
          }
          return { rows: [{ id: "concept-one", label: "Amber", kind: "concept" },
            { id: "place-two", label: "Amber", kind: "place" }], truncated: false };
        }
        if (command === "workspace_save_layout") {
          calls.push({ command, args }); world.revision++; return null;
        }
        throw new Error("Unexpected mock command: " + command);
      },
    } });
  });
  await page.goto("/");
  const graph = page.getByRole("region", { name: "Saved world graph" });
  const inspector = graph.getByRole("complementary", { name: "Node inspector" });
  await inspector.getByRole("button", { name: "Open character page" }).click();
  await inspector.getByRole("textbox", { name: "Page text", exact: true }).fill("Amber research");
  await inspector.getByRole("button", { name: "Add entry link" }).click();
  await inspector.getByLabel("Link caption").fill("A carefully authored caption");
  await inspector.getByText("Find link target", { exact: true }).click();
  const picker = inspector.locator("[data-reference-picker]");
  await picker.getByRole("searchbox", { name: "Search link target" }).fill("Old");
  await picker.getByRole("button", { name: "Search references" }).click();
  await expect(picker.getByText("Searching the selected author view…")).toBeVisible();
  await picker.getByRole("searchbox", { name: "Search link target" }).fill("Amber");
  await picker.getByRole("button", { name: "Search references" }).click();
  await expect(picker.getByRole("button", { name: "Amber · concept · concept-one" })).toBeVisible();
  await page.evaluate(() => (window as unknown as { __finishOldReference: () => void }).__finishOldReference());
  await expect(picker.getByRole("button", { name: "Amber · place · place-two" })).toBeVisible();
  await expect(picker.getByText("Obsolete", { exact: false })).toHaveCount(0);
  if (process.env.ATLAS_CAPTURE_PHASE3 === "1") {
    const timestamp = new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-") + "Z";
    await inspector.screenshot({ path: "../../Development-Screenshots/" + timestamp + "_phase-04_canvas_mock-native-reference-picker_01.png", animations: "disabled" });
  }
  await picker.getByRole("button", { name: "Amber · concept · concept-one" }).click();
  await expect(inspector.getByLabel("Link target identity")).toHaveValue("concept-one");
  await expect(inspector.getByLabel("Link caption")).toHaveValue("A carefully authored caption");
  await expect(inspector.getByLabel("Link target identity")).toBeFocused();
  const calls = () => page.evaluate(() => (window as unknown as { __referenceCalls: { command: string; args: unknown }[] }).__referenceCalls);
  const before = await calls();
  expect(before.filter(call => call.command === "workspace_save_layout")).toHaveLength(0);
  expect(before.filter(call => call.command === "workspace_semantic_candidates")).toMatchObject([
    { args: { request: { world_id: "reference-world", day: 0, revision: 4 }, kind: "entry", phrase: "Old" } },
    { args: { request: { world_id: "reference-world", day: 0, revision: 4 }, kind: "entry", phrase: "Amber" } },
  ]);
  expect(before.filter(call => call.command === "workspace_cancel_query")).toHaveLength(1);
  await inspector.getByRole("button", { name: "Save page" }).click();
  await expect(page.locator("#author-world-info")).toContainText("Author revision 5 / 5");
  expect((await calls()).filter(call => call.command === "workspace_save_layout")).toMatchObject([
    { args: { document: { root: { content: { Stack: [
      expect.anything(), expect.anything(), { content: { ObjectLink: { target: { world: "reference-world", id: "concept-one" }, label: "A carefully authored caption" } } },
    ] } } } } },
  ]);
});
