import { defineConfig } from "vite";

export default defineConfig({
  root: "ui",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**", "**/target/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2021",
  },
});
