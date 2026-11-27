// The keyboard shortcuts of the Windows app (SP-017): ← and → page through the frames, Alt+1 to Alt+7 open the steps of the
// step bar. The Mac app has ⌘[ ⌘] and ⌥⌘1 to ⌥⌘7 in its menus instead.

import { viewSteps, type ViewStep } from "./workflow";

export type KeyCommand = { kind: "frame"; by: -1 | 1 } | { kind: "step"; step: ViewStep };

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
}

/** What a key press does, or null when it is not a shortcut. */
export function keyCommand(press: KeyPress): KeyCommand | null {
  const modifiers = press.altKey || press.ctrlKey || press.metaKey || press.shiftKey;
  if (!modifiers && !press.inField) {
    if (press.key === "ArrowLeft") return { kind: "frame", by: -1 };
    if (press.key === "ArrowRight") return { kind: "frame", by: 1 };
  }
  if (press.altKey && !press.ctrlKey && !press.metaKey && !press.shiftKey) {
    const digit = /^(?:Digit|Numpad)([1-7])$/.exec(press.code)?.[1];
    if (digit) return { kind: "step", step: viewSteps[Number(digit) - 1].id };
  }
  return null;
}

/** The shortcut of a step for its tooltip: "Alt+2". */
export const stepShortcut = (id: ViewStep): string => `Alt+${viewSteps.findIndex((s) => s.id === id) + 1}`;
