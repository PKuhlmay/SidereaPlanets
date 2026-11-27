// The auto-updater (PLANETS-13): the real calls behind updateController.ts. The Tauri updater plugin reads the release feed named in
// `plugins.updater` of src-tauri/tauri.conf.json (GitHub Releases), checks the signature of the download against the public key
// there and installs it; the process plugin relaunches. The flow and the rules are in updateController.ts and logic/updates.ts.

import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { watch } from "vue";
import { about, status } from "./state";
import { createUpdater, type AvailableUpdate, type KeyValueStore } from "./updateController";

/** Waits this long after the start before the automatic check, so it does not compete with the window coming up. */
const START_DELAY_MS = 5000;

/** localStorage of the webview; where it is blocked every read says "nothing stored" and a write is dropped. */
const storage: KeyValueStore = {
  get: (key) => {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set: (key, value) => {
    try {
      localStorage.setItem(key, value);
    } catch {
      // Storage can be blocked; the choice is then forgotten at the next start.
    }
  },
  remove: (key) => {
    try {
      localStorage.removeItem(key);
    } catch {
      // see above
    }
  },
};

/** The plugin's answer as the controller sees it: version, notes and the two steps. */
async function checkRelease(): Promise<AvailableUpdate | null> {
  const update = await check();
  if (!update) return null;
  return {
    version: update.version,
    notes: update.body ?? "",
    download: async (onProgress) => {
      let total: number | null = null;
      let received = 0;
      await update.download((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? null;
        if (event.event === "Progress") received += event.data.chunkLength;
        onProgress(total ? Math.min(1, received / total) : null);
      });
    },
    install: () => update.install(),
  };
}

/** Resolves when `status.busy` is empty, i.e. no calculation runs. */
function untilIdle(): Promise<void> {
  return new Promise((resolve) => {
    const stop = watch(
      () => status.busy === null,
      (idle) => {
        if (!idle) return;
        stop();
        resolve();
      },
      { immediate: true },
    );
  });
}

/** The state of the dialog and the settings, and the actions of its buttons. */
export const updates = createUpdater({
  check: checkRelease,
  relaunch,
  installSupported: () => invoke<boolean>("update_install_supported"),
  isBusy: () => status.busy !== null,
  untilIdle,
  now: () => Date.now(),
  storage,
  report: (text) => (status.result = text),
});

/** Whether a dialog sits on top of the window (About or the new version): the keys and the menu bar do not reach what is behind it. */
export function dialogOpen(): boolean {
  return about.open || updates.state.update !== null;
}

const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

/** The check at the start of the app, once a day and only when switched on in Settings. Called once from App.vue. */
export function startUpdateCheck(): void {
  // The design preview of dev builds runs in a plain browser; there is no updater there.
  if (!inTauri()) return;
  window.setTimeout(() => void updates.startCheck(), START_DELAY_MS);
}

/** Check for Updates…: asks at once and tells the result. */
export const checkForUpdates = (): Promise<void> => updates.checkNow();
