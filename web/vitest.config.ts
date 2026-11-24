import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  test: {
    environment: "jsdom",
    environmentOptions: { jsdom: { url: "http://klar.test/" } },
    include: ["tests/**/*.test.{ts,tsx}"],
    setupFiles: ["./tests/support/setup.ts"],
    css: false,
    // Room for a busy CI runner; a passing test does not wait for it.
    testTimeout: 15000,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
