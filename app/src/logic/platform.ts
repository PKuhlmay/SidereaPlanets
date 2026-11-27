// The platform the window runs on. The Mac has its own conventions (Cmd instead of Ctrl, a menu bar, gestures, the system
// appearance); Windows and Linux keep the Ctrl shortcuts and the toolbar menus.

/** Whether the web view runs on macOS, from `navigator.platform` and the user agent. */
export const detectMac = (platform: string, userAgent: string): boolean => /^Mac/i.test(platform) || /Macintosh|Mac OS X/.test(userAgent);

/** A shortcut of the primary modifier and a key, for tooltips: "⌘O" on the Mac, "Ctrl+O" elsewhere. */
export const chord = (key: string, mac: boolean): string => (mac ? `⌘${key}` : `Ctrl+${key}`);

export const isMac: boolean = typeof navigator !== "undefined" && detectMac(navigator.platform ?? "", navigator.userAgent ?? "");
