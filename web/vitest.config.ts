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
    restoreMocks: true,
    unstubGlobals: true,
  },
});
