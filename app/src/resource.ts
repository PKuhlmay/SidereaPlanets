// The CPU and RAM readout at the left end of the status bar (RK-019): reads the app's CPU time and memory from the backend
// (`process_usage`, src-tauri/src/usage_commands.rs) about once a second, only while the readout is shown (Settings) and the window
// is visible. The math and the texts are in logic/processUsage.ts.

import { invoke } from "@tauri-apps/api/core";
import { reactive, watch } from "vue";
import { cpuPercent, isMeasuring, type ProcessSample } from "./logic/processUsage";

const KEY = "rk-resource-usage";
const INTERVAL_MS = 1000;

/** Takes readings from `sample` and keeps the last load and memory. A failed reading or a stop forgets both. */
export function createMonitor(sample: () => Promise<ProcessSample | null>) {
  const state = reactive({ cpuPercent: null as number | null, memoryBytes: null as number | null });
  let previous: ProcessSample | null = null;

  async function refresh(): Promise<void> {
    const current = await sample().catch(() => null);
    if (!current) {
      stop();
      return;
    }
    state.memoryBytes = current.memoryBytes;
    state.cpuPercent = previous ? cpuPercent(previous, current) : null;
    previous = current;
  }

  function stop(): void {
    previous = null;
    state.cpuPercent = null;
    state.memoryBytes = null;
  }

  return { state, refresh, stop };
}

function loadShown(): boolean {
  try {
    return localStorage.getItem(KEY) !== "off";
  } catch {
    return true;
  }
}

/** Settings > Show CPU and Memory Usage; on by default, remembered where the webview allows it. */
export const resourceSettings = reactive({ shown: loadShown() });

const backend = createMonitor(async () => {
  const reading = await invoke<{ cpuSeconds: number; memoryBytes: number } | null>("process_usage");
  return reading ? { ...reading, uptime: performance.now() / 1000 } : null;
});

/** The values the status bar shows. */
export const usage = backend.state;

/** Starts and stops the readings with the setting and the visibility of the window. Called once from App.vue. */
export function startResourceMonitor(): void {
  // The design preview of dev builds runs in a plain browser; there is no backend to ask.
  if (!("__TAURI_INTERNALS__" in window)) return;
  const visible = reactive({ value: document.visibilityState === "visible" });
  // A minimized window is hidden for the webview.
  document.addEventListener("visibilitychange", () => (visible.value = document.visibilityState === "visible"));
  let timer: number | undefined;
  watch(
    () => isMeasuring(resourceSettings.shown, visible.value),
    (measuring) => {
      window.clearInterval(timer);
      timer = undefined;
      if (!measuring) {
        backend.stop();
        return;
      }
      void backend.refresh();
      timer = window.setInterval(() => void backend.refresh(), INTERVAL_MS);
    },
    { immediate: true },
  );
  watch(
    () => resourceSettings.shown,
    (shown) => {
      try {
        localStorage.setItem(KEY, shown ? "on" : "off");
      } catch {
        // Storage can be blocked; the readout is then on again next time.
      }
    },
  );
}
