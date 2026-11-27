// The commands behind the native menu bar of the Mac and the keys of keys.ts (PLANETS-3): one id per command, which of them
// are possible right now, and the ids the menu sends back (src-tauri/src/menu.rs builds the menu with the same ids).

import { recentLabel } from "./recent";
import { viewSteps, type ViewStep } from "./workflow";

export type CommandId =
  | "selectRecording"
  | "openProject"
  | "saveImage"
  | "saveProject"
  | "cancel"
  | "togglePause"
  | "settings"
  | "help"
  | "about"
  | "zoomIn"
  | "zoomOut"
  | "zoomFit"
  | "zoomActual";

const commandIds: readonly string[] = [
  "selectRecording",
  "openProject",
  "saveImage",
  "saveProject",
  "cancel",
  "togglePause",
  "settings",
  "help",
  "about",
  "zoomIn",
  "zoomOut",
  "zoomFit",
  "zoomActual",
] satisfies CommandId[];

export type MenuTarget = { kind: "command"; id: CommandId } | { kind: "step"; step: ViewStep } | { kind: "recent"; index: number };

/** What a menu item stands for, from the id it sends (`menu-action`); null for an id of this app that is not known. */
export function parseMenuId(id: string): MenuTarget | null {
  if (commandIds.includes(id)) return { kind: "command", id: id as CommandId };
  const step = /^step:(.+)$/.exec(id)?.[1];
  if (step) return viewSteps.some((s) => s.id === step) ? { kind: "step", step: step as ViewStep } : null;
  const index = /^recent:(\d+)$/.exec(id)?.[1];
  return index === undefined ? null : { kind: "recent", index: Number(index) };
}

/** The facts that decide which commands work. */
export interface CommandState {
  /** A recording is open. */
  loaded: boolean;
  /** A calculation runs. */
  busy: boolean;
  /** The stack exists, so there is an image to save. */
  stacked: boolean;
  /** The image area shows an image. */
  hasImage: boolean;
  /** The running calculation is paused. */
  paused: boolean;
  /** Per step: it is not locked (the step bar lets it open). */
  stepOpenable: Record<ViewStep, boolean>;
  /** The recordings opened last, newest first. */
  recent: string[][];
}

/** Whether a command can run now; the menu dims it otherwise and the keys do nothing. */
export function commandEnabled(id: CommandId, state: CommandState): boolean {
  switch (id) {
    case "selectRecording":
    case "openProject":
      return !state.busy;
    case "saveImage":
      return state.stacked && !state.busy;
    case "saveProject":
      return state.loaded && !state.busy;
    case "cancel":
    case "togglePause":
      return state.busy;
    case "zoomIn":
    case "zoomOut":
    case "zoomFit":
    case "zoomActual":
      return state.hasImage;
    case "settings":
    case "help":
    case "about":
      return true;
  }
}

/** What the native menu is told: the state of every item, the name of Pause and the entries of Open Recent. */
export function menuUpdate(state: CommandState): { enabled: Record<string, boolean>; paused: boolean; recent: string[] } {
  const enabled: Record<string, boolean> = {};
  for (const id of commandIds) enabled[id] = commandEnabled(id as CommandId, state);
  for (const step of viewSteps) enabled[`step:${step.id}`] = state.stepOpenable[step.id] && !state.busy;
  state.recent.forEach((_, index) => (enabled[`recent:${index}`] = !state.busy));
  return { enabled, paused: state.busy && state.paused, recent: state.recent.map(recentLabel) };
}
