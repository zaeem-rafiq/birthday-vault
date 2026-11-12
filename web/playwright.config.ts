import { defineConfig, devices } from "@playwright/test";
import { E2E_PORT } from "./e2e/port";

// One worker: every test shares one local sandbox chain (web/e2e/global-setup.ts).
export default defineConfig({
  testDir: "./e2e",
  globalSetup: "./e2e/global-setup.ts",
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  timeout: 180_000,
  expect: { timeout: 30_000 },
  reporter: [["list"]],
  use: { baseURL: `http://127.0.0.1:${E2E_PORT}`, trace: "retain-on-failure" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
