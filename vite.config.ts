import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1525,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1526 } : undefined,
    watch: { ignored: ["**/src-tauri/**", "**/legacy-egui/**"] },
  },
  resolve: {
    alias: { $lib: path.resolve("./src/lib") },
  },
  optimizeDeps: {
    include: ["monaco-editor/esm/vs/editor/editor.api"],
  },
  worker: {
    format: "es",
  },
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
