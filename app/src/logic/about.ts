// What the About dialog shows (PLANETS-7): the license, the link to the source code and the third-party license notice.

import type { Theme } from "./settings";

export const LICENSE_ID = "GPL-3.0-or-later";

/** Where the source code lives. The only place the address is written down; change it here when the repository moves. */
export const SOURCE_URL = "https://github.com/PKuhlmay/SidereaPlanets";

/** Where the generated notice is served from (app/public, written by `pnpm gen:licenses`). */
export const NOTICE_URL = "third-party-licenses.html";

/** The first line the generator writes (app/scripts/notice.mjs); a dev server answers a missing file with its index page instead. */
export const NOTICE_MARKER = "<!-- siderea-third-party-licenses -->";

/** The notice with the app's theme applied, or `null` when the text is not the generated notice. */
export function themedNotice(html: string, theme: Theme): string | null {
  if (!html.includes(NOTICE_MARKER)) return null;
  return html.replace("<html", `<html data-theme="${theme}"`);
}
