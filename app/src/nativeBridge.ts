// What the system sends to the window besides the mouse and the keys: the clicks of the native menu bar (Mac), and files
// that Finder asks the app to open (double click on a project, Open With). The Rust side is src-tauri/src/menu.rs and lib.rs.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { watch } from "vue";
import { commandState, goToStep, runCommand } from "./appCommands";
import { menuUpdate, parseMenuId } from "./logic/commands";
import { isMac } from "./logic/platform";
import { openPaths } from "./openFromOutside";
import { openRecording, recent } from "./recording";
import { status } from "./state";

const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

/** What a click on a menu item does, from the id the menu sent. */
export async function runMenuAction(id: string): Promise<void> {
  const target = parseMenuId(id);
  if (!target) return;
  if (target.kind === "command") await runCommand(target.id);
  else if (target.kind === "step") goToStep(target.step);
  else {
    const paths = recent.list[target.index];
    if (paths && status.busy === null) await openRecording(paths);
  }
}

/** Opens the files that the system asked the app to open; they wait in the backend until the window asks. */
async function openRequestedFiles(): Promise<void> {
  const files = await invoke<string[]>("take_opened_files");
  if (files.length > 0) await openPaths(files);
}

/** Starts listening: the menu bar on the Mac and the files to open. Outside the Tauri window (browser preview) it does nothing. */
export async function startNativeBridge(): Promise<void> {
  if (!inTauri()) return;
  await listen("files-opened", () => void openRequestedFiles());
  await openRequestedFiles();
  if (!isMac) return;
  await listen<string>("menu-action", ({ payload }) => void runMenuAction(payload));
  // The menu bar learns which items work now, and the Open Recent list, whenever either changes.
  watch(
    () => JSON.stringify(menuUpdate(commandState.value)),
    (update) => void invoke("update_menu", { update: JSON.parse(update) as unknown }).catch(() => undefined),
    { immediate: true },
  );
}
