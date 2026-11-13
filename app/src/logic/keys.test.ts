import { describe, expect, it } from "vitest";
import { keyCommand } from "./keys";

const key = (k: string, more: Partial<Parameters<typeof keyCommand>[0]> = {}) =>
  keyCommand({ key: k, code: "", altKey: false, ctrlKey: false, metaKey: false, shiftKey: false, inField: false, ...more });

describe("keyCommand (Windows shortcuts of RK-017)", () => {
  it("pages through the frames with the left and right arrow keys", () => {
    expect(key("ArrowLeft")).toEqual({ kind: "frame", by: -1 });
    expect(key("ArrowRight")).toEqual({ kind: "frame", by: 1 });
  });

  it("leaves the arrow keys to a field that has the focus and ignores them with a modifier", () => {
    expect(key("ArrowLeft", { inField: true })).toBeNull();
    expect(key("ArrowRight", { shiftKey: true })).toBeNull();
    expect(key("ArrowRight", { ctrlKey: true })).toBeNull();
  });

  it("opens step 1 to 7 with Alt and the digit, also on the number pad and while a field has the focus", () => {
    expect(key("1", { altKey: true, code: "Digit1" })).toEqual({ kind: "step", step: "open" });
    expect(key("4", { altKey: true, code: "Digit4" })).toEqual({ kind: "step", step: "limit" });
    expect(key("7", { altKey: true, code: "Numpad7", inField: true })).toEqual({ kind: "step", step: "save" });
  });

  it("does nothing for other digits or without Alt", () => {
    expect(key("8", { altKey: true, code: "Digit8" })).toBeNull();
    expect(key("2", { code: "Digit2" })).toBeNull();
    expect(key("2", { altKey: true, ctrlKey: true, code: "Digit2" })).toBeNull();
  });
});
