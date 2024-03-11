import { defineConfig } from "astro/config";

import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  vite: {
    optimizeDeps: {
      esbuildOptions: {
        define: {
          // Needed because the agent uses `global` to build the Actor
          global: "globalThis"
        }
      }
    }
  },
  integrations: [tailwind()]
});