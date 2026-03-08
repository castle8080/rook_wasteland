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
