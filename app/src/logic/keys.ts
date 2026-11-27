// The keyboard shortcuts of the window (SP-017, PLANETS-3): ← and → page through the frames, a digit opens a step of the step
// bar, and the commands of the Mac menu bar have a key on every platform. "Primary" is Cmd on the Mac and Ctrl elsewhere.
//
//   steps    Windows and Linux: Alt+1 to Alt+7; Mac: ⌥⌘1 to ⌥⌘7 (as the menu Run ▸ Go to Step of the earlier Mac app)
//   frames   ← →; primary+[ and primary+] (found by position of the key: a German keyboard has no bracket there)
//   actions  primary+O select, +S save image, +, settings, +. cancel, +plus / +minus zoom, +0 fit, +1 actual size

import type { CommandId } from "./commands";
import { viewSteps, type ViewStep } from "./workflow";

export type KeyCommand =
  | { kind: "frame"; by: -1 | 1 }
  | { kind: "step"; step: ViewStep }
  | { kind: "action"; action: CommandId }
  /** A key that would reload the web view (release builds only): taken and dropped. */
  | { kind: "block" };

export interface KeyPress {
  key: string;
  /** `KeyboardEvent.code`: "Digit3" or "Numpad3", the same whatever the keyboard layout makes of Alt and the digit. */
  code: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  /** A text or number field has the focus: the arrow keys move its cursor then. */
  inField: boolean;
  /** The window runs on a Mac. */
  mac: boolean;
  /** A release build: the keys that reload the web view are blocked; a dev build keeps them. */
  release?: boolean;
}

/** The keys that reload the web view and would throw away the open recording and every result: F5 and primary+R (with or without Shift). */
function isReloadKey(press: KeyPress, primary: boolean): boolean {
  if (press.altKey) return false;
  if (press.key === "F5") return !press.mac;
  return primary && (press.code === "KeyR" || press.key.toLowerCase() === "r");
}

/** What a key press does, or null when it is not a shortcut. */
export function keyCommand(press: KeyPress): KeyCommand | null {
  const primary = press.mac ? press.metaKey : press.ctrlKey;
  const foreign = press.mac ? press.ctrlKey : press.metaKey;
  if (press.release && isReloadKey(press, primary) && !foreign) return { kind: "block" };
  if (foreign) return null;

  if (!primary && !press.altKey && !press.shiftKey && !press.inField) {
    if (press.key === "ArrowLeft") return { kind: "frame", by: -1 };
    if (press.key === "ArrowRight") return { kind: "frame", by: 1 };
  }

  // The steps: Alt and the digit, on the Mac with Cmd as well.
  if (press.altKey && !press.shiftKey && primary === press.mac) {
    const digit = /^(?:Digit|Numpad)([1-7])$/.exec(press.code)?.[1];
    return digit ? { kind: "step", step: viewSteps[Number(digit) - 1].id } : null;
  }

  if (!primary || press.altKey) return null;
  if (!press.shiftKey) {
    if (press.code === "BracketLeft") return { kind: "frame", by: -1 };
    if (press.code === "BracketRight") return { kind: "frame", by: 1 };
  }
  const letter = press.key.toLowerCase();
  if (press.shiftKey) return letter === "+" ? { kind: "action", action: "zoomIn" } : null;
  const actions: Record<string, CommandId> = {
    o: "selectRecording",
    s: "saveImage",
    ",": "settings",
    ".": "cancel",
    "=": "zoomIn",
    "+": "zoomIn",
    "-": "zoomOut",
    "0": "zoomFit",
    "1": "zoomActual",
  };
  const action = actions[letter];
  return action ? { kind: "action", action } : null;
}

/** The shortcut of a step for its tooltip: "Alt+2", on the Mac "⌥⌘2". */
export const stepShortcut = (id: ViewStep, mac: boolean): string => {
  const number = viewSteps.findIndex((s) => s.id === id) + 1;
  return mac ? `⌥⌘${number}` : `Alt+${number}`;
};
