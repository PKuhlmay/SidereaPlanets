// Files that come from outside the window: dropped onto it, or opened by Finder (double click on a project, Open With).

import { getCurrentWebview } from "@tauri-apps/api/webview";
import { isProjectFile } from "./logic/projectfile";
import { openRecording } from "./recording";
import { openProjectFile } from "./saving";
import { status } from "./state";

/**
 * A project file opens the project, anything else is a recording (several still frames form one sequence), like the files
 * picked with Select Recording…. Nothing opens during a calculation.
 */
export async function openPaths(paths: string[]): Promise<void> {
  if (paths.length === 0) return;
  if (status.busy !== null) {
    status.result = "Open: wait until the calculation has finished";
    return;
  }
  const project = paths.find(isProjectFile);
  if (project) await openProjectFile(project);
  else await openRecording(paths);
}

/** Files dropped onto the window open like Select Recording… (openPaths). Outside the Tauri window (browser preview) there is nothing to listen to. */
export function openDroppedFiles(): void {
  try {
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") void openPaths(event.payload.paths);
      })
      .catch(() => undefined);
  } catch {
    // Not running inside Tauri.
  }
}
