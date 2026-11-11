import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // Tests run in a far-off time zone (UTC+14) to prove dive times are never
    // shifted by the computer's own time zone. Set before workers start.
    globalSetup: ["./vitest.timezone.ts"],
  },
});
