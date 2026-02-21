import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Read app version from package.json
import { readFileSync } from "node:fs";
const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },

  // ── Build optimizations ──────────────────────────────────────────
  // Tauri WebView is modern (Chromium 120+ / WebKit 16+), target esnext
  // for smallest output with no polyfills. Code-split React + Tauri SDK.
  build: {
    target: "esnext",
    reportCompressedSize: false, // skip gzip calc → faster builds
    // esbuild minifier (built into Vite, zero extra deps)
    minify: "esbuild",
    rollupOptions: {
      output: {
        manualChunks: {
          react: ["react", "react-dom"],
          tauri: ["@tauri-apps/api"],
        },
      },
    },
  },

  // ── esbuild: strip dev-only code ────────────────────────────────
  esbuild: {
    legalComments: "none",
    drop: ["console", "debugger"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
