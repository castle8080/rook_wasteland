import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: "list",

  use: {
    baseURL: "http://localhost:8080",
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  // Start trunk serve before running tests.
  // reuseExistingServer lets you keep trunk running in a terminal during dev.
  webServer: {
    command: "trunk serve --no-autoreload",
    url: "http://localhost:8080/rw_sixzee/",
    reuseExistingServer: true,
    // WASM compile on first run can take ~30s
    timeout: 120_000,
  },
});
