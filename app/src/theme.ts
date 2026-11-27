import { theme } from "./state";
import { isMac } from "./logic/platform";
import type { Theme } from "./logic/settings";

const KEY = "sp-theme";

function show(next: Theme) {
  theme.current = next;
  document.documentElement.dataset.theme = next;
}

// Dark is the default (docs/design.md); the choice is remembered where the webview allows it.
export function applyTheme(next: Theme) {
  show(next);
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Storage can be blocked; the theme then simply starts dark next time.
  }
}

/** On the Mac the window follows the appearance of macOS, also while it runs; nothing is remembered and there is no choice. */
function followSystem() {
  const dark = window.matchMedia("(prefers-color-scheme: dark)");
  show(dark.matches ? "dark" : "light");
  dark.addEventListener("change", (event) => show(event.matches ? "dark" : "light"));
}

export function restoreTheme() {
  if (isMac) {
    followSystem();
    return;
  }
  let saved: string | null = null;
  try {
    saved = localStorage.getItem(KEY);
  } catch {
    // see above
  }
  applyTheme(saved === "light" ? "light" : "dark");
}
