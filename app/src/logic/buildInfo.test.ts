import { describe, expect, it } from "vitest";
import { buildLabel } from "./buildInfo";

// The version label at the right of the status bar, as in Siderea: "0.1.0 · 3801a70 · 2026-11-02 14:32" (SP-025).
describe("buildLabel", () => {
  it("joins version, commit and build time, the time in local time to the minute", () => {
    expect(buildLabel("0.1.0", "3801a70", new Date(2026, 10, 2, 14, 32, 59))).toBe("0.1.0 · 3801a70 · 2026-11-02 14:32");
  });

  it("leaves out what is not known", () => {
    expect(buildLabel("0.1.0", "", new Date(2026, 0, 5, 9, 7))).toBe("0.1.0 · 2026-01-05 09:07");
    expect(buildLabel("0.1.0", "abcdef0", null)).toBe("0.1.0 · abcdef0");
    expect(buildLabel("0.1.0", "", null)).toBe("0.1.0");
  });
});
