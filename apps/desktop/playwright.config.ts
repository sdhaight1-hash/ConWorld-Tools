import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  use: {
    baseURL: "http://localhost:4173",
    launchOptions: { executablePath: process.env.ATLAS_CHROMIUM },
  },
  webServer: {
    command: "npm exec vite -- preview --host localhost --port 4173 --strictPort",
    url: "http://localhost:4173",
  },
});
