import { defineConfig, devices } from "@playwright/test";

// Smoke tests against a real engine binary that serves the built UI (`npm run build` first).
// Each test starts its own engine on a free port with an empty data directory; see
// tests/e2e/engine.ts for how the binary is found.
export default defineConfig({
  testDir: "tests/e2e",
  testMatch: "*.spec.ts",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  timeout: 30_000,
  reporter: "list",
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
