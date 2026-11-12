// The CPUs field of the toolbar: the number of threads for all calculations. The backend (src-tauri/src/compute.rs)
// holds the thread pool; this module reads the number of cores at the start, makes it the default and the upper end
// of the field, and tells the backend whenever the value changes. The value is not saved in the project file: the
// number of cores belongs to the computer.

import { invoke } from "@tauri-apps/api/core";
import { reactive, watch } from "vue";
import { clampCpus, coreCount } from "./logic/cpus";
import { settings } from "./state";

// The design preview of dev builds runs in a plain browser; there is no backend to ask.
const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

/** The upper end of the CPUs field: the number of logical cores, once the backend has told it. */
export const cpu = reactive({ max: 1 });

/** Reads the number of cores, sets the field to all of them and keeps the backend in step with it from then on. */
export async function initCpus(): Promise<void> {
  if (!inTauri()) return;
  cpu.max = coreCount(await invoke<number>("cpu_count"));
  watch(
    () => settings.cpus,
    (value) => {
      // The field cuts typed values already; this catches everything else that sets the value.
      const clamped = clampCpus(value, cpu.max);
      if (clamped !== value) settings.cpus = clamped;
      else void invoke("set_cpus", { count: clamped });
    },
  );
  // The backend starts with all cores as well, so this call only changes something when the value was not that.
  settings.cpus = cpu.max;
}
