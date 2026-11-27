import { afterEach, describe, expect, it, vi } from "vitest";

// updater.ts imports the Tauri plugins; only dialogOpen() is under test here.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: vi.fn() }));
vi.mock("@tauri-apps/plugin-updater", () => ({ check: vi.fn() }));

import { about } from "./state";
import { dialogOpen, updates } from "./updater";

afterEach(() => {
  about.open = false;
  updates.state.update = null;
});

describe("dialogOpen", () => {
  it("is false while no dialog is shown", () => {
    expect(dialogOpen()).toBe(false);
  });

  it("is true while the About dialog is open", () => {
    about.open = true;
    expect(dialogOpen()).toBe(true);
  });

  it("is true while the dialog of a new version is open, so shortcuts and the menu bar stay behind it", () => {
    updates.state.update = { version: "0.2.0", notes: "", canInstall: true };
    expect(dialogOpen()).toBe(true);
  });
});
