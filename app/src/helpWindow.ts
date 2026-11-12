// Help opens the documentation in a window of its own, above the main window; the work view is not touched.

import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

/** The query that makes the page show the documentation instead of the work view (see main.ts). */
export const HELP_QUERY = "view=help";

const LABEL = "help";
const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

/** Opens the help window, or brings it to the front when it is already open. */
export async function openHelp(): Promise<void> {
  if (!inTauri()) {
    // Browser preview of a dev build: a popup window does the same job.
    window.open(`?${HELP_QUERY}`, LABEL, "popup,width=1000,height=720");
    return;
  }
  const existing = await WebviewWindow.getByLabel(LABEL);
  if (existing) {
    await existing.setFocus();
    return;
  }
  const help = new WebviewWindow(LABEL, {
    url: `index.html?${HELP_QUERY}`,
    title: "SidereaPlanets Help",
    width: 1000,
    height: 720,
    minWidth: 640,
    minHeight: 400,
    center: true,
    parent: "main",
  });
  help.once("tauri://error", (event) => console.error("Help window:", event.payload));
}

/** Closes the window the documentation is shown in (Escape in the help window). */
export async function closeHelp(): Promise<void> {
  if (!inTauri()) {
    window.close();
    return;
  }
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}
