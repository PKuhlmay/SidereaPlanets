// The keyboard shortcuts in the window (SP-017): ← and → page through the frames while the frame bar is there, a digit with Alt
// (on the Mac with ⌥⌘) opens a step like a click on the step bar, and the commands of the Mac menu bar have keys on every platform.
// What a key means is decided in logic/keys.ts. On the Mac the menu bar knows the same keys; whichever of the two sees the key
// first takes it (a handled key calls preventDefault), so a key never does its job twice.

import { goToStep, runCommand } from "./appCommands";
import { suppressContextMenu } from "./logic/contextMenu";
import { keyCommand, type KeyPress } from "./logic/keys";
import { isMac } from "./logic/platform";
import { showNextFrame } from "./recording";
import { about, session } from "./state";

/** The steps that show the frame bar (App.vue). */
const FRAME_STEPS = ["alignpoints", "align", "limit"];

/** A focused element that uses the arrow keys itself: text and number fields, sliders, radio buttons and lists. */
function usesArrows(target: EventTarget | null): boolean {
  if (typeof HTMLElement === "undefined" || !(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target.tagName === "TEXTAREA" || target.tagName === "SELECT") return true;
  return target.tagName === "INPUT" && !["checkbox", "button", "submit", "reset"].includes((target as HTMLInputElement).type);
}

/** Release builds only: the keys that reload the web view (PLANETS-26). Dev builds reload on purpose, by hand and through Vite. */
const RELEASE = import.meta.env.PROD;

function keyPress(event: KeyboardEvent): KeyPress {
  return {
    key: event.key,
    code: event.code,
    altKey: event.altKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
    inField: usesArrows(event.target),
    mac: isMac,
    release: RELEASE,
  };
}

/** Handles a key press of the main window; returns whether it was a shortcut. */
export function handleShortcut(event: KeyboardEvent): boolean {
  const command = keyCommand(keyPress(event));
  // The About dialog sits on top of the window: the keys do not reach what is behind it, but a reload would still wipe it.
  if (about.open && command?.kind !== "block") return false;
  if (!command) return false;
  // A blocked key does nothing but fall through to preventDefault below.
  if (command.kind === "frame") {
    if (!session.info || !FRAME_STEPS.includes(session.view)) return false;
    showNextFrame(command.by);
  } else if (command.kind === "step") {
    goToStep(command.step);
  } else if (command.kind === "action") {
    void runCommand(command.action);
  }
  // A shortcut is always taken, so that it does not reach the window menu of Windows or the page (zoom of the web view).
  event.preventDefault();
  return true;
}

/** The help window shows the same page without the shortcuts of the main window; it only has to keep the reload keys out. */
export function handleHelpShortcut(event: KeyboardEvent): void {
  if (keyCommand(keyPress(event))?.kind === "block") event.preventDefault();
}

/** The native context menu is suppressed in release builds except in text fields (PLANETS-26). */
export function handleContextMenu(event: MouseEvent): void {
  const element = event.target instanceof HTMLElement ? event.target : null;
  const suppress = suppressContextMenu({
    release: RELEASE,
    tag: element?.tagName ?? "",
    type: element instanceof HTMLInputElement ? element.type : "",
    editable: element?.isContentEditable ?? false,
  });
  if (suppress) event.preventDefault();
}
