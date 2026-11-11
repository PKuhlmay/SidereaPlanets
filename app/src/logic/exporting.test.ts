import { describe, expect, it } from "vitest";
import { directoryOf, stem } from "./exporting";

describe("stem", () => {
  it("drops the last extension", () => {
    expect(stem("JUPITER.AVI")).toBe("JUPITER");
    expect(stem("jupiter.v2.ser")).toBe("jupiter.v2");
  });
  it("keeps names without extension and hidden names", () => {
    expect(stem("jupiter")).toBe("jupiter");
    expect(stem(".hidden")).toBe(".hidden");
    expect(stem("")).toBe("");
  });
});

describe("directoryOf", () => {
  it("handles both kinds of separator", () => {
    expect(directoryOf("C:\Aufnahmen\JUPITER.AVI")).toBe("C:\Aufnahmen");
    expect(directoryOf("/home/me/jupiter.ser")).toBe("/home/me");
  });
  it("has no folder for a bare name or nothing", () => {
    expect(directoryOf("jupiter.ser")).toBeUndefined();
    expect(directoryOf(undefined)).toBeUndefined();
  });
});
