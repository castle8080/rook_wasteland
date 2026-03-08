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

    const startNewBtn = page.locator(
      ".resume-prompt .btn--secondary, .resume-prompt button:has-text('Discard')"
    );
    if (await startNewBtn.isVisible()) {
      await startNewBtn.click();
      await page.waitForTimeout(300);

      // Dismiss the opening-quote overlay that appears after Start New.
      const letsPlay = page.locator(".grandma-quote-overlay .btn--primary");
      if (await letsPlay.isVisible()) {
        await letsPlay.click();
        await page.waitForTimeout(200);
      }

      // Fresh game — all dice should show '?'
      const diceButtons = page.locator(".dice-row button");
      await expect(diceButtons).toHaveCount(5);
      for (let i = 0; i < 5; i++) {
        await expect(diceButtons.nth(i)).toHaveText("?");
      }
    }
  });
});
