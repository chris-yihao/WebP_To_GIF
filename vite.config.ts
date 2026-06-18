import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: {
    strictPort: true,
    port: 1420,
    host: "127.0.0.1",
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
