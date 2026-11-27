// The flow of the updater (PLANETS-13): check at the start and from the menu, the dialog with its three answers, Install and
// Restart. Everything the Tauri plugins, the clock, the storage and the window do comes in through `UpdaterDeps`, so the flow is a
// unit test (updateController.test.ts) and updater.ts only supplies the real calls. The rules themselves are in logic/updates.ts.

import { reactive } from "vue";
import { checkFailureText, checkOutcome, installBlock, shouldCheckAtStart, type CheckMode } from "./logic/updates";

const KEY_AUTO = "sp-update-auto";
const KEY_LAST_CHECK = "sp-update-last-check";
const KEY_SKIPPED = "sp-update-skipped";

/** The new version as the release feed describes it, with the two steps that fetch and install it. */
export interface AvailableUpdate {
  version: string;
  /** The release notes, plain text; empty when the release has none. */
  notes: string;
  /** Downloads the update; `onProgress` gets the share done (0..1), or null while the size is not known. */
  download(onProgress: (fraction: number | null) => void): Promise<void>;
  /** Installs what was downloaded. On Windows this ends the app, so it comes last before `relaunch`. */
  install(): Promise<void>;
}

export interface KeyValueStore {
  get(key: string): string | null;
  set(key: string, value: string): void;
  remove(key: string): void;
}

export interface UpdaterDeps {
  /** Asks the release feed; null when this version is the newest. Rejects when the feed cannot be reached. */
  check(): Promise<AvailableUpdate | null>;
  relaunch(): Promise<void>;
  /** Whether this installation can replace itself (false for a Linux .deb). */
  installSupported(): Promise<boolean>;
  /** A calculation (Align, Stack, Wavelets …) is running. */
  isBusy(): boolean;
  /** Resolves when no calculation runs any more. */
  untilIdle(): Promise<void>;
  now(): number;
  storage: KeyValueStore;
  /** Puts a one-line note in the status bar. */
  report(text: string): void;
}

const errorMessage = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export function createUpdater(deps: UpdaterDeps) {
  const state = reactive({
    /** Settings > Check for updates automatically. */
    auto: deps.storage.get(KEY_AUTO) !== "off",
    checking: false,
    installing: false,
    /** The download is done, but a calculation runs: the install waits for it. */
    waiting: false,
    progress: null as number | null,
    /** The update the dialog shows; null = no dialog. */
    update: null as { version: string; notes: string; canInstall: boolean } | null,
    error: null as string | null,
  });
  let pending: AvailableUpdate | null = null;

  const lastCheck = (): number | null => {
    const stored = deps.storage.get(KEY_LAST_CHECK);
    return stored !== null && Number.isFinite(Number(stored)) ? Number(stored) : null;
  };

  async function check(mode: CheckMode): Promise<void> {
    if (state.checking || state.installing) return;
    state.checking = true;
    // The attempt counts as the day's check, so a computer without a connection does not ask at every start.
    deps.storage.set(KEY_LAST_CHECK, String(deps.now()));
    try {
      const found = await deps.check();
      const outcome = checkOutcome(mode, found?.version ?? null, deps.storage.get(KEY_SKIPPED));
      if (outcome === "upToDate") deps.report("Siderea Planets is up to date");
      if (outcome === "show" && found) {
        pending = found;
        state.error = null;
        state.progress = null;
        state.update = { version: found.version, notes: found.notes, canInstall: await deps.installSupported() };
      }
    } catch (error) {
      // An automatic check that fails is not worth a word; a manual one says so in the status bar.
      if (mode === "manual") deps.report(checkFailureText(error));
    } finally {
      state.checking = false;
    }
  }

  return {
    state,
    /** The check at the start of the app: only when it is on in Settings and a day has passed. */
    startCheck: async (): Promise<void> => {
      if (shouldCheckAtStart(state.auto, lastCheck(), deps.now())) await check("automatic");
    },
    /** Check for Updates…: always asks at once. */
    checkNow: (): Promise<void> => check("manual"),
    /** Install and Restart. */
    install: async (): Promise<void> => {
      if (!pending || !state.update || state.installing) return;
      if (installBlock(deps.isBusy(), state.update.canInstall) !== null) return;
      state.installing = true;
      state.error = null;
      try {
        await pending.download((fraction) => (state.progress = fraction));
        // A calculation can have started while the file came down; the app is not closed under it.
        if (deps.isBusy()) {
          state.waiting = true;
          await deps.untilIdle();
          state.waiting = false;
        }
        await pending.install();
        await deps.relaunch();
      } catch (error) {
        state.error = `Could not install the update: ${errorMessage(error)}`;
        state.installing = false;
        state.waiting = false;
      }
    },
    /** Later: closes the dialog; the next automatic check offers the version again. */
    later: (): void => {
      if (state.installing) return;
      pending = null;
      state.update = null;
    },
    /** Skip This Version: automatic checks keep quiet about it and older ones; a newer version is reported. */
    skip: (): void => {
      if (state.installing || !state.update) return;
      deps.storage.set(KEY_SKIPPED, state.update.version);
      pending = null;
      state.update = null;
    },
    setAuto: (on: boolean): void => {
      state.auto = on;
      deps.storage.set(KEY_AUTO, on ? "on" : "off");
    },
  };
}
