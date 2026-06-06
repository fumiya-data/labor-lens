import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const localServerTarget = process.env.LABORLENS_LOCAL_SERVER_URL ?? "http://127.0.0.1:5174";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    proxy: {
      "/api": {
        target: localServerTarget,
        changeOrigin: true,
      },
    },
  },
});
