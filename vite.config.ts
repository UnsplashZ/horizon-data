import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 期望前端固定端口、不清屏
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
});
