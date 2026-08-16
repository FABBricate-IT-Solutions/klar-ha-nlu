import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  server: {
    proxy: {
      "/api": "http://127.0.0.1:10520",
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
