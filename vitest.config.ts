import { defineConfig } from "vitest/config";

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
    poolOptions: {
      threads: {
        singleThread: true, // SQLite requires single thread
      },
    },
  },
  resolve: {
    alias: {
      "@": "/src",
    },
  },
});
