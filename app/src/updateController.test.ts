import { describe, expect, it, vi } from "vitest";
import { CHECK_INTERVAL_MS } from "./logic/updates";
import { createUpdater, type AvailableUpdate, type KeyValueStore } from "./updateController";

const NOW = 1_700_000_000_000;

function memoryStore(initial: Record<string, string> = {}): KeyValueStore & { data: Record<string, string> } {
  const data = { ...initial };
  return {
    data,
    get: (key) => data[key] ?? null,
    set: (key, value) => void (data[key] = value),
    remove: (key) => void delete data[key],
  };
}

interface Calls {
  checks: number;
  downloads: number;
  installs: number;
  relaunches: number;
  reports: string[];
}

function fakeUpdate(version: string, calls: Calls, failInstall = false): AvailableUpdate {
  return {
    version,
    notes: `Notes of ${version}`,
    download: async (onProgress) => {
      calls.downloads++;
      onProgress(0.5);
    },
    install: async () => {
      calls.installs++;
      if (failInstall) throw new Error("disk full");
    },
  };
}

/** An updater with fake plugin calls; `offered` is what the release feed has, `busy` whether a calculation runs. */
function setup(
  options: { offered?: string | null; busy?: boolean; supported?: boolean; store?: Record<string, string>; now?: number; failCheck?: boolean; failInstall?: boolean } = {},
) {
  const calls: Calls = { checks: 0, downloads: 0, installs: 0, relaunches: 0, reports: [] };
  const flags = { busy: options.busy ?? false };
  const idle: { release: () => void } = { release: () => undefined };
  const storage = memoryStore(options.store);
  const updater = createUpdater({
    check: async () => {
      calls.checks++;
      if (options.failCheck) throw new Error("offline");
      return options.offered ? fakeUpdate(options.offered, calls, options.failInstall) : null;
    },
    relaunch: async () => void calls.relaunches++,
    installSupported: async () => options.supported ?? true,
    isBusy: () => flags.busy,
    untilIdle: () => new Promise<void>((resolve) => (idle.release = resolve)),
    now: () => options.now ?? NOW,
    storage,
    report: (text) => calls.reports.push(text),
  });
  return { updater, calls, flags, idle, storage };
}

describe("the check at the start", () => {
  it("looks for an update, remembers the time and shows what it finds", async () => {
    const { updater, calls, storage } = setup({ offered: "0.2.0" });
    await updater.startCheck();
    expect(calls.checks).toBe(1);
    expect(storage.data["sp-update-last-check"]).toBe(String(NOW));
    expect(updater.state.update).toEqual({ version: "0.2.0", notes: "Notes of 0.2.0", canInstall: true });
  });

  it("does not look again within a day", async () => {
    const { updater, calls } = setup({ offered: "0.2.0", store: { "sp-update-last-check": String(NOW - CHECK_INTERVAL_MS / 2) } });
    await updater.startCheck();
    expect(calls.checks).toBe(0);
    expect(updater.state.update).toBeNull();
  });

  it("does not look at all when it is switched off in Settings", async () => {
    const { updater, calls } = setup({ offered: "0.2.0", store: { "sp-update-auto": "off" } });
    expect(updater.state.auto).toBe(false);
    await updater.startCheck();
    expect(calls.checks).toBe(0);
  });

  it("stays silent when nothing is new and when it fails", async () => {
    const none = setup({ offered: null });
    await none.updater.startCheck();
    expect(none.calls.reports).toEqual([]);
    const failing = setup({ failCheck: true });
    await failing.updater.startCheck();
    expect(failing.calls.reports).toEqual([]);
    expect(failing.updater.state.update).toBeNull();
  });

  it("stays silent about a skipped version", async () => {
    const { updater } = setup({ offered: "0.2.0", store: { "sp-update-skipped": "0.2.0" } });
    await updater.startCheck();
    expect(updater.state.update).toBeNull();
  });

  it("shows a version newer than the skipped one", async () => {
    const { updater } = setup({ offered: "0.2.1", store: { "sp-update-skipped": "0.2.0" } });
    await updater.startCheck();
    expect(updater.state.update?.version).toBe("0.2.1");
  });
});

describe("the check from the menu", () => {
  it("checks at once, even right after another check, and says when the app is up to date", async () => {
    const { updater, calls } = setup({ offered: null, store: { "sp-update-last-check": String(NOW - 1000), "sp-update-auto": "off" } });
    await updater.checkNow();
    expect(calls.checks).toBe(1);
    expect(calls.reports).toEqual(["Siderea Planets is up to date"]);
  });

  it("shows the skipped version again", async () => {
    const { updater } = setup({ offered: "0.2.0", store: { "sp-update-skipped": "0.2.0" } });
    await updater.checkNow();
    expect(updater.state.update?.version).toBe("0.2.0");
  });

  it("reports a failure quietly in one line", async () => {
    const { updater, calls } = setup({ failCheck: true });
    await updater.checkNow();
    expect(calls.reports).toEqual(["Could not check for updates: offline"]);
    expect(updater.state.checking).toBe(false);
  });

  it("starts no second check while one runs", async () => {
    const { updater, calls } = setup({ offered: null });
    await Promise.all([updater.checkNow(), updater.checkNow()]);
    expect(calls.checks).toBe(1);
  });
});

describe("the dialog", () => {
  it("closes on Later and asks again the next day, not for this version only", async () => {
    const { updater, storage } = setup({ offered: "0.2.0" });
    await updater.startCheck();
    updater.later();
    expect(updater.state.update).toBeNull();
    expect(storage.data["sp-update-skipped"]).toBeUndefined();
  });

  it("stores the version on Skip This Version and closes", async () => {
    const { updater, storage } = setup({ offered: "0.2.0" });
    await updater.startCheck();
    updater.skip();
    expect(updater.state.update).toBeNull();
    expect(storage.data["sp-update-skipped"]).toBe("0.2.0");
  });

  it("marks an installation that cannot update itself", async () => {
    const { updater } = setup({ offered: "0.2.0", supported: false });
    await updater.checkNow();
    expect(updater.state.update?.canInstall).toBe(false);
  });
});

describe("Install and Restart", () => {
  it("downloads, installs and relaunches", async () => {
    const { updater, calls } = setup({ offered: "0.2.0" });
    await updater.checkNow();
    await updater.install();
    expect([calls.downloads, calls.installs, calls.relaunches]).toEqual([1, 1, 1]);
    expect(updater.state.progress).toBe(0.5);
  });

  it("does nothing while a calculation runs", async () => {
    const { updater, calls } = setup({ offered: "0.2.0", busy: true });
    await updater.checkNow();
    await updater.install();
    expect([calls.downloads, calls.installs, calls.relaunches]).toEqual([0, 0, 0]);
    expect(updater.state.installing).toBe(false);
  });

  it("does nothing for an installation that cannot update itself", async () => {
    const { updater, calls } = setup({ offered: "0.2.0", supported: false });
    await updater.checkNow();
    await updater.install();
    expect(calls.downloads).toBe(0);
  });

  it("waits with the install while a calculation that started during the download runs", async () => {
    const { updater, calls, flags, idle } = setup({ offered: "0.2.0" });
    await updater.checkNow();
    const running = updater.install();
    // The download finishes, then a calculation is found running.
    flags.busy = true;
    await vi.waitFor(() => expect(updater.state.waiting).toBe(true));
    expect(calls.installs).toBe(0);
    flags.busy = false;
    idle.release();
    await running;
    expect([calls.installs, calls.relaunches]).toEqual([1, 1]);
    expect(updater.state.waiting).toBe(false);
  });

  it("shows a failure in the dialog and lets the user try again", async () => {
    const { updater, calls } = setup({ offered: "0.2.0", failInstall: true });
    await updater.checkNow();
    await updater.install();
    expect(updater.state.error).toBe("Could not install the update: disk full");
    expect(updater.state.installing).toBe(false);
    expect(calls.relaunches).toBe(0);
  });
});

describe("the automatic check setting", () => {
  it("is remembered", () => {
    const { updater, storage } = setup();
    expect(updater.state.auto).toBe(true);
    updater.setAuto(false);
    expect(updater.state.auto).toBe(false);
    expect(storage.data["sp-update-auto"]).toBe("off");
    updater.setAuto(true);
    expect(storage.data["sp-update-auto"]).toBe("on");
  });
});
