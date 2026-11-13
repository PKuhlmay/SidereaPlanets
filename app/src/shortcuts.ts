// The keyboard shortcuts in the window (RK-017): ← and → page through the frames while the frame bar is there, Alt+1 to Alt+7 open
// a step like a click on the step bar. What a key means is decided in logic/keys.ts.

import { keyCommand } from "./logic/keys";
import { showNextFrame } from "./recording";
import { session, status } from "./state";
import { states } from "./steps";

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
  });
  if (!command) return false;
  if (command.kind === "frame") {
    if (!session.info || !FRAME_STEPS.includes(session.view)) return false;
    showNextFrame(command.by);
  } else if (status.busy === null && states.value[command.step].status !== "locked") {
    session.view = command.step;
  }
  // Alt and a digit is always taken, so that it does not reach the window menu of Windows.
  event.preventDefault();
  return true;
}
