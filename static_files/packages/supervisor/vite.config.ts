import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import checker from "vite-plugin-checker";
import { viteStaticCopy } from "vite-plugin-static-copy";

// https://vitejs.dev/config/
export default defineConfig({
  base: "",
  plugins: [
    react(),
    tailwindcss(),
    // checker({
    //   typescript: {
    //     tsconfigPath: "./tsconfig.json",
    //     buildMode: true,
    //   },
    // }),
    viteStaticCopy({
      targets: [
        {
          src: "../shared/public/*",
          dest: "",
        },
      ],
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@scipo-code/shared": path.resolve(__dirname, "../shared/src"),
    },
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
});
