import { describe, expect, it } from "vitest";
import { chord, detectMac } from "./platform";

describe("detectMac", () => {
  it("knows a Mac from the platform of the web view", () => {
    expect(detectMac("MacIntel", "")).toBe(true);
    expect(detectMac("", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15")).toBe(true);
  });

  it("takes Windows and Linux for the others", () => {
    expect(detectMac("Win32", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")).toBe(false);
    expect(detectMac("Linux x86_64", "Mozilla/5.0 (X11; Linux x86_64)")).toBe(false);
  });
});

describe("chord", () => {
  it("writes a shortcut the way the platform does", () => {
    expect(chord("O", true)).toBe("⌘O");
    expect(chord("O", false)).toBe("Ctrl+O");
  });
});
