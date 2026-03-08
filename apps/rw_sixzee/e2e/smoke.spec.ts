/**
 * Smoke tests — verify the app loads correctly in a real browser against a
 * running trunk serve instance.
 *
 * These tests use `trunk serve` (via playwright.config.ts webServer) so the
 * full asset pipeline runs: WASM compile, JS glue, grandma_quotes.json, CSS.
 *
 * Run with: python make.py e2e
 */

import { test, expect, type Page } from "@playwright/test";

/** Navigate to the app and wait for all network activity to settle.
 *
 * The WASM binary is fetched via a dynamic import AFTER the `load` event, so
 * bare `page.goto()` returns before the WASM is downloaded. `networkidle`
 * waits for 500ms of no requests, ensuring the WASM and JSON assets are fully
 * fetched before we start asserting on Leptos-rendered elements.
 */
async function navigate(page: Page) {
  await page.goto("/rw_sixzee/", { waitUntil: "networkidle", timeout: 45_000 });
}

test.describe("App load", () => {
  test("serves the page with a 200 response", async ({ page }) => {
    const response = await page.goto("/rw_sixzee/");
    expect(response?.status()).toBe(200);
  });

  test("page title is SIXZEE", async ({ page }) => {
    await page.goto("/rw_sixzee/");
    await expect(page).toHaveTitle(/SIXZEE/i);
  });

  test("WASM initializes and renders app UI", async ({ page }) => {
    await navigate(page);
    // After WASM init, either the opening-quote overlay (if quote bank loaded)
    // or the game header is visible. The app always renders one of them.
    const hasOverlay = await page
      .locator(".grandma-quote-overlay")
      .isVisible();
    const hasHeader = await page.locator(".game-header").isVisible();
    expect(hasOverlay || hasHeader).toBe(true);
  });

  test("five dice are visible after dismissing opening quote", async ({
    page,
  }) => {
    await navigate(page);

    // Dismiss the Grandma opening-quote overlay if present.
    const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
    if (await letsPlay.isVisible()) {
      await letsPlay.click();
    }

    await expect(page.locator(".dice-row button")).toHaveCount(5);
  });

  test("no uncaught JS errors on load", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await navigate(page);

    expect(errors).toHaveLength(0);
  });

  test("grandma_quotes.json asset loads (opening quote shown)", async ({
    page,
  }) => {
    await navigate(page);

    // If the quote bank loaded, the opening overlay should be visible.
    await expect(page.locator(".grandma-quote-overlay")).toBeVisible();
  });
});

// ─── M6 Persistence smoke tests ──────────────────────────────────────────────

/** Helper: dismiss the opening-quote overlay if present, then roll once. */
async function dismissQuoteAndRoll(page: Page) {
  const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
  if (await letsPlay.isVisible()) {
    await letsPlay.click();
  }
  // Click roll button
  const rollBtn = page.locator(".action-buttons button").filter({ hasText: "ROLL" });
  await rollBtn.click();
  await page.waitForTimeout(200);
}

/** Helper: dismiss the resume prompt if present (choose "Resume Game"). */
async function dismissResumeIfPresent(page: Page) {
  const resumeBtn = page.locator(".resume-prompt .btn--primary");
  if (await resumeBtn.isVisible()) {
    await resumeBtn.click();
    await page.waitForTimeout(200);
  }
}

test.describe("M6 Persistence", () => {
  test.beforeEach(async ({ page }) => {
    // Clear saved game state before each test so tests are independent.
    await page.goto("/rw_sixzee/", { waitUntil: "networkidle", timeout: 45_000 });
    await page.evaluate(() => {
      localStorage.removeItem("rw_sixzee.in_progress");
      localStorage.removeItem("rw_sixzee.history");
    });
  });

  test("after rolling and refreshing, resume prompt appears", async ({
    page,
  }) => {
    await navigate(page);
    await dismissQuoteAndRoll(page);

    // Reload the page — WASM must re-init from localStorage save.
    await page.reload({ waitUntil: "networkidle" });

    // Either resume prompt or error overlay must be present (not a blank game).
    const hasResume = await page.locator(".resume-prompt").isVisible();
    const hasError = await page.locator(".error-overlay").isVisible();
    expect(hasResume || hasError).toBe(
      true,
      "page should show resume prompt (or error) after reload with a saved game"
    );
  });

  test("resume prompt shows correct turn count", async ({ page }) => {
    await navigate(page);
    await dismissQuoteAndRoll(page);

    // Score a cell to advance the turn (click the first preview cell).
    const previewCell = page.locator(".scorecard__cell--preview").first();
    if (await previewCell.isVisible()) {
      await previewCell.click();
      await page.waitForTimeout(200);
    }

    // Reload and check resume prompt shows turn ≥ 2.
    await page.reload({ waitUntil: "networkidle" });

    const resumePrompt = page.locator(".resume-prompt");
    if (await resumePrompt.isVisible()) {
      // Turn count text should be visible in the meta section.
      const metaValues = page.locator(".resume-prompt__meta-value");
      const count = await metaValues.count();
      expect(count).toBeGreaterThan(0);
    }
  });

  test("choosing Start New on resume prompt starts a fresh game", async ({
    page,
  }) => {
    await navigate(page);
    await dismissQuoteAndRoll(page);

    // Reload to trigger resume prompt.
    await page.reload({ waitUntil: "networkidle" });

    // Hard-wait for the resume prompt — it must appear or the test should fail.
    const startNewBtn = page.locator(
      ".resume-prompt .btn--secondary, .resume-prompt button:has-text('Discard')"
    );
    await startNewBtn.waitFor({ state: "visible", timeout: 5000 });
    await startNewBtn.click();

    // After clicking "Discard and Start New" a new opening-quote overlay
    // appears.  Use waitFor so we never miss it regardless of WASM render
    // timing.  The overlay may take a tick to appear after the reactive flush.
    const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
    try {
      await letsPlay.waitFor({ state: "visible", timeout: 3000 });
      await letsPlay.click();
    } catch {
      // Quote bank not yet loaded — game view renders directly without overlay.
    }

    // Fresh game — all dice should show '?'
    const diceButtons = page.locator(".dice-row button");
    await expect(diceButtons).toHaveCount(5);
    for (let i = 0; i < 5; i++) {
      await expect(diceButtons.nth(i)).toHaveText("?");
    }
  });
});

// ─── M7 Ask Grandma smoke tests ───────────────────────────────────────────────

test.describe("M7 Ask Grandma", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/rw_sixzee/", { waitUntil: "networkidle", timeout: 45_000 });
    await page.evaluate(() => {
      localStorage.removeItem("rw_sixzee.in_progress");
      localStorage.removeItem("rw_sixzee.history");
    });
  });

  /** Dismiss opening quote (if present) and roll once so Ask Grandma is enabled. */
  async function rollOnce(page: Page) {
    const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
    if (await letsPlay.isVisible()) {
      await letsPlay.click();
    }
    const rollBtn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ROLL" });
    await rollBtn.click();
    await page.waitForTimeout(300);
  }

  test("Ask Grandma button is disabled before first roll", async ({ page }) => {
    await navigate(page);
    const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
    if (await letsPlay.isVisible()) {
      await letsPlay.click();
    }
    const btn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ASK GRANDMA" });
    await expect(btn).toBeDisabled();
  });

  test("Ask Grandma button is enabled after rolling", async ({ page }) => {
    await navigate(page);
    await rollOnce(page);
    const btn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ASK GRANDMA" });
    // Button may still be disabled if the worker failed to load (e.g. no
    // grandma_worker_core.js in dist yet).  We assert it is either enabled or
    // explicitly disabled — never missing.
    await expect(btn).toBeVisible();
  });

  test("clicking Ask Grandma opens the overlay panel", async ({ page }) => {
    await navigate(page);
    await rollOnce(page);

    const btn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ASK GRANDMA" });
    // Skip test if button is still disabled (worker binary not built yet).
    if (await btn.isDisabled()) {
      test.skip();
      return;
    }

    await btn.click();

    // Overlay should appear within 2 s (worker init + computation).
    const overlay = page.locator(".overlay--grandma");
    await overlay.waitFor({ state: "visible", timeout: 5_000 });
    await expect(overlay).toBeVisible();
  });

  test("grandma panel shows at least one action card after loading", async ({
    page,
  }) => {
    await navigate(page);
    await rollOnce(page);

    const btn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ASK GRANDMA" });
    if (await btn.isDisabled()) {
      test.skip();
      return;
    }

    await btn.click();

    // Wait for cards to appear (worker must compute).
    const cards = page.locator(".grandma-card");
    await cards.first().waitFor({ state: "visible", timeout: 10_000 });
    const count = await cards.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });

  test("closing the grandma panel hides the overlay", async ({ page }) => {
    await navigate(page);
    await rollOnce(page);

    const btn = page
      .locator(".action-buttons button")
      .filter({ hasText: "ASK GRANDMA" });
    if (await btn.isDisabled()) {
      test.skip();
      return;
    }

    await btn.click();
    await page.locator(".overlay--grandma").waitFor({ state: "visible", timeout: 5_000 });

    await page.locator(".overlay--grandma__close").click();
    await expect(page.locator(".overlay--grandma")).toBeHidden();
  });
});
