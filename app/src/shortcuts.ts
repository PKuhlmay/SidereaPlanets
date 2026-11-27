// The keyboard shortcuts in the window (SP-017): ← and → page through the frames while the frame bar is there, a digit with Alt
// (on the Mac with ⌥⌘) opens a step like a click on the step bar, and the commands of the Mac menu bar have keys on every platform.
// What a key means is decided in logic/keys.ts. On the Mac the menu bar knows the same keys; whichever of the two sees the key
// first takes it (a handled key calls preventDefault), so a key never does its job twice.

import { goToStep, runCommand } from "./appCommands";
import { keyCommand } from "./logic/keys";
import { isMac } from "./logic/platform";
import { showNextFrame } from "./recording";
import { session } from "./state";

/** The steps that show the frame bar (App.vue). */
const FRAME_STEPS = ["alignpoints", "align", "limit"];

/** A focused element that uses the arrow keys itself: text and number fields, sliders, radio buttons and lists. */
function usesArrows(target: EventTarget | null): boolean {
  if (typeof HTMLElement === "undefined" || !(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target.tagName === "TEXTAREA" || target.tagName === "SELECT") return true;
  return target.tagName === "INPUT" && !["checkbox", "button", "submit", "reset"].includes((target as HTMLInputElement).type);
}

/** Handles a key press of the main window; returns whether it was a shortcut. */
export function handleShortcut(event: KeyboardEvent): boolean {
  const command = keyCommand({
    key: event.key,
    code: event.code,
    altKey: event.altKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
    inField: usesArrows(event.target),
    mac: isMac,
  });
  if (!command) return false;
  if (command.kind === "frame") {
    if (!session.info || !FRAME_STEPS.includes(session.view)) return false;
    showNextFrame(command.by);
  } else if (command.kind === "step") {
    goToStep(command.step);
  } else {
    void runCommand(command.action);
  }
  // A shortcut is always taken, so that it does not reach the window menu of Windows or the page (zoom of the web view).
  event.preventDefault();
  return true;
}
