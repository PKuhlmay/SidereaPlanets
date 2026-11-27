import { describe, expect, it } from "vitest";
import { keyCommand, stepShortcut } from "./keys";

const key = (k: string, more: Partial<Parameters<typeof keyCommand>[0]> = {}) =>
  keyCommand({ key: k, code: "", altKey: false, ctrlKey: false, metaKey: false, shiftKey: false, inField: false, mac: false, ...more });
const mac = (k: string, more: Partial<Parameters<typeof keyCommand>[0]> = {}) => key(k, { mac: true, ...more });

describe("keyCommand on Windows and Linux", () => {
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

  it("uses Ctrl, not Cmd, for the shortcuts of the menu bar", () => {
    expect(key("o", { ctrlKey: true })).toEqual({ kind: "action", action: "selectRecording" });
    expect(key("s", { ctrlKey: true })).toEqual({ kind: "action", action: "saveImage" });
    expect(key("o", { metaKey: true })).toBeNull();
    expect(key("o")).toBeNull();
  });

  it("does not take Alt and Cmd for the steps", () => {
    expect(key("1", { altKey: true, metaKey: true, code: "Digit1" })).toBeNull();
  });
});

describe("keyCommand on the Mac", () => {
  it("opens step 1 to 7 with Option and Cmd and the digit, whatever character the layout gives Option and the digit", () => {
    expect(mac("¡", { altKey: true, metaKey: true, code: "Digit1" })).toEqual({ kind: "step", step: "open" });
    expect(mac("7", { altKey: true, metaKey: true, code: "Numpad7", inField: true })).toEqual({ kind: "step", step: "save" });
  });

  it("does not take Option alone, or Cmd alone, or Ctrl, for the steps", () => {
    expect(mac("1", { altKey: true, code: "Digit1" })).toBeNull();
    expect(mac("1", { altKey: true, ctrlKey: true, code: "Digit1" })).toBeNull();
    expect(mac("8", { altKey: true, metaKey: true, code: "Digit8" })).toBeNull();
  });

  it("pages through the frames with the arrow keys and with Cmd and the bracket keys, found by position on the keyboard", () => {
    expect(mac("ArrowLeft")).toEqual({ kind: "frame", by: -1 });
    expect(mac("[", { metaKey: true, code: "BracketLeft" })).toEqual({ kind: "frame", by: -1 });
    // A German keyboard has no bracket on this key; the position still counts.
    expect(mac("ü", { metaKey: true, code: "BracketLeft" })).toEqual({ kind: "frame", by: -1 });
    expect(mac("]", { metaKey: true, code: "BracketRight", inField: true })).toEqual({ kind: "frame", by: 1 });
    expect(mac("[", { ctrlKey: true, code: "BracketLeft" })).toBeNull();
  });

  it("takes Cmd for the shortcuts of the menu bar and ignores Ctrl", () => {
    expect(mac("o", { metaKey: true })).toEqual({ kind: "action", action: "selectRecording" });
    expect(mac("O", { metaKey: true })).toEqual({ kind: "action", action: "selectRecording" });
    expect(mac("s", { metaKey: true })).toEqual({ kind: "action", action: "saveImage" });
    expect(mac(",", { metaKey: true })).toEqual({ kind: "action", action: "settings" });
    expect(mac(".", { metaKey: true })).toEqual({ kind: "action", action: "cancel" });
    expect(mac("o", { ctrlKey: true })).toBeNull();
    expect(mac("o", { metaKey: true, shiftKey: true })).toBeNull();
    expect(mac("o", { metaKey: true, altKey: true })).toBeNull();
  });

  it("zooms with Cmd and plus, minus, 0 and 1", () => {
    expect(mac("=", { metaKey: true })).toEqual({ kind: "action", action: "zoomIn" });
    expect(mac("+", { metaKey: true, shiftKey: true })).toEqual({ kind: "action", action: "zoomIn" });
    expect(mac("-", { metaKey: true })).toEqual({ kind: "action", action: "zoomOut" });
    expect(mac("0", { metaKey: true })).toEqual({ kind: "action", action: "zoomFit" });
    expect(mac("1", { metaKey: true })).toEqual({ kind: "action", action: "zoomActual" });
  });
});

describe("stepShortcut", () => {
  it("names the shortcut of a step for its tooltip, per platform", () => {
    expect(stepShortcut("align", false)).toBe("Alt+3");
    expect(stepShortcut("align", true)).toBe("⌥⌘3");
  });
});

describe("keys that reload the web view (PLANETS-26)", () => {
  it("blocks F5, Ctrl+R and Ctrl+Shift+R in a release build on Windows and Linux", () => {
    expect(key("F5", { release: true, code: "F5" })).toEqual({ kind: "block" });
    expect(key("r", { release: true, ctrlKey: true, code: "KeyR" })).toEqual({ kind: "block" });
    expect(key("R", { release: true, ctrlKey: true, shiftKey: true, code: "KeyR" })).toEqual({ kind: "block" });
  });

  it("blocks Cmd+R and Cmd+Shift+R in a release build on the Mac, not Ctrl+R", () => {
    expect(mac("r", { release: true, metaKey: true, code: "KeyR" })).toEqual({ kind: "block" });
    expect(mac("R", { release: true, metaKey: true, shiftKey: true, code: "KeyR" })).toEqual({ kind: "block" });
    expect(mac("r", { release: true, ctrlKey: true, code: "KeyR" })).toBeNull();
  });

  it("blocks them also while a field has the focus", () => {
    expect(key("r", { release: true, ctrlKey: true, inField: true, code: "KeyR" })).toEqual({ kind: "block" });
  });

  it("keeps every one of them in a dev build, which reloads on Vite changes and by hand", () => {
    expect(key("F5", { code: "F5" })).toBeNull();
    expect(key("r", { ctrlKey: true, code: "KeyR" })).toBeNull();
    expect(mac("r", { metaKey: true, code: "KeyR" })).toBeNull();
  });

  it("leaves other keys alone, also with the same modifiers", () => {
    expect(key("t", { release: true, ctrlKey: true, code: "KeyT" })).toBeNull();
    expect(key("F6", { release: true, code: "F6" })).toBeNull();
    expect(key("r", { release: true, code: "KeyR" })).toBeNull();
    expect(key("r", { release: true, ctrlKey: true, altKey: true, code: "KeyR" })).toBeNull();
    expect(mac("F5", { release: true, code: "F5" })).toBeNull();
  });

  it("does not change what the existing shortcuts do in a release build", () => {
    expect(key("o", { release: true, ctrlKey: true })).toEqual({ kind: "action", action: "selectRecording" });
    expect(key("ArrowLeft", { release: true })).toEqual({ kind: "frame", by: -1 });
  });
});
