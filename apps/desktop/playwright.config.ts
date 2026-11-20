import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  use: {
    baseURL: "http://localhost:4173",
    launchOptions: {
      executablePath: process.env.ATLAS_CHROMIUM,
      args: process.env.ATLAS_SOFTWARE_WEBGPU === "1"
        ? ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--enable-unsafe-webgpu", "--disable-vulkan-surface", "--in-process-gpu"]
        : [],
    },
  },
  webServer: {
    command: "npm exec vite -- preview --host localhost --port 4173 --strictPort",
    url: "http://localhost:4173",
  },
});
