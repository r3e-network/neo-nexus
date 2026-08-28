import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    setupFiles: ["./tests/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      exclude: [
        "node_modules/",
        "tests/",
        "dist/",
        "web/",
        "archive/",
        "**/*.d.ts",
        "**/*.config.*",
      ],
    },
    exclude: [
      "node_modules/",
      "dist/",
      "web/",
      "archive/**",
    ],
    testTimeout: 30000,
    hookTimeout: 30000,
    pool: "threads",
    minWorkers: 1,
    maxWorkers: 1, // SQLite-backed tests must not run concurrently.
  },
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
});
