import { theme } from "./state";
import type { Theme } from "./logic/settings";

const KEY = "sp-theme";

// Dark is the default (docs/design.md); the choice is remembered where the webview allows it.
export function applyTheme(next: Theme) {
  theme.current = next;
  document.documentElement.dataset.theme = next;
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Storage can be blocked; the theme then simply starts dark next time.
  }
}

export function restoreTheme() {
  let saved: string | null = null;
  try {
    saved = localStorage.getItem(KEY);
  } catch {
    // see above
  }
  applyTheme(saved === "light" ? "light" : "dark");
}
