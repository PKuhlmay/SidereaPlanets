import { describe, expect, it } from "vitest";
import { commandEnabled, menuUpdate, parseMenuId, type CommandState } from "./commands";

const idle: CommandState = {
  loaded: true,
  busy: false,
  stacked: true,
  hasImage: true,
  paused: false,
  stepOpenable: { open: true, alignpoints: true, align: true, limit: true, stack: true, wavelets: true, save: false },
  recent: [["C:/data/JUPITER.SER"], ["C:/data/a.tif", "C:/data/b.tif"]],
};

describe("parseMenuId", () => {
  it("reads the ids the native menu sends", () => {
    expect(parseMenuId("selectRecording")).toEqual({ kind: "command", id: "selectRecording" });
    expect(parseMenuId("togglePause")).toEqual({ kind: "command", id: "togglePause" });
    // "About Siderea Planets" opens the About dialog of the window, not the default macOS about panel.
    expect(parseMenuId("about")).toEqual({ kind: "command", id: "about" });
    expect(parseMenuId("step:wavelets")).toEqual({ kind: "step", step: "wavelets" });
    expect(parseMenuId("recent:2")).toEqual({ kind: "recent", index: 2 });
  });

  it("ignores everything else", () => {
    expect(parseMenuId("step:nowhere")).toBeNull();
    expect(parseMenuId("recent:x")).toBeNull();
    expect(parseMenuId("quit")).toBeNull();
  });
});

describe("commandEnabled", () => {
  it("allows opening and saving only while nothing runs, and saving only with something to save", () => {
    expect(commandEnabled("selectRecording", idle)).toBe(true);
    expect(commandEnabled("selectRecording", { ...idle, busy: true })).toBe(false);
    expect(commandEnabled("openProject", { ...idle, busy: true })).toBe(false);
    expect(commandEnabled("saveProject", { ...idle, loaded: false })).toBe(false);
    expect(commandEnabled("saveProject", { ...idle, busy: true })).toBe(false);
    expect(commandEnabled("saveImage", idle)).toBe(true);
    expect(commandEnabled("saveImage", { ...idle, stacked: false })).toBe(false);
  });

  it("allows Cancel and Pause only during a calculation", () => {
    expect(commandEnabled("cancel", idle)).toBe(false);
    expect(commandEnabled("cancel", { ...idle, busy: true })).toBe(true);
    expect(commandEnabled("togglePause", { ...idle, busy: true })).toBe(true);
  });

  it("needs an image for the zoom and always allows Settings and Help", () => {
    expect(commandEnabled("zoomIn", idle)).toBe(true);
    expect(commandEnabled("zoomFit", { ...idle, hasImage: false })).toBe(false);
    expect(commandEnabled("settings", { ...idle, busy: true })).toBe(true);
    expect(commandEnabled("help", { ...idle, busy: true })).toBe(true);
    expect(commandEnabled("about", { ...idle, busy: true })).toBe(true);
  });
});

describe("menuUpdate", () => {
  it("lists every item with its state, like the step bar for the steps", () => {
    const update = menuUpdate(idle);
    expect(update.enabled["about"]).toBe(true);
    expect(update.enabled["step:align"]).toBe(true);
    expect(update.enabled["step:save"]).toBe(false);
    expect(update.enabled["saveImage"]).toBe(true);
    expect(menuUpdate({ ...idle, busy: true }).enabled["step:align"]).toBe(false);
  });

  it("names the recordings of Open Recent and locks them during a calculation", () => {
    expect(menuUpdate(idle).recent).toEqual(["JUPITER.SER", "a.tif (+1)"]);
    expect(menuUpdate(idle).enabled["recent:1"]).toBe(true);
    expect(menuUpdate({ ...idle, busy: true }).enabled["recent:1"]).toBe(false);
  });

  it("tells whether Pause reads Resume", () => {
    expect(menuUpdate(idle).paused).toBe(false);
    expect(menuUpdate({ ...idle, busy: true, paused: true }).paused).toBe(true);
  });
});
