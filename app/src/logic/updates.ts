// The rules of the updater (PLANETS-13): when the app looks for a new version, which versions it reports, when installing is
// allowed. No plugin and no window in here, so every rule is a unit test; updater.ts wires them to the Tauri updater plugin.

/** The automatic check runs at most once in this time. */
export const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;

/**
 * Where the releases live. The updater plugin reads its endpoint from `plugins.updater.endpoints` in `src-tauri/tauri.conf.json`;
 * a test keeps that and this address together, so a move of the repository is a change in these two places only.
 */
export const RELEASES_URL = "https://github.com/PKuhlmay/SidereaPlanets/releases";

export interface Version {
  major: number;
  minor: number;
  patch: number;
  /** The dot-separated identifiers after the hyphen ("rc", "1" for 1.0.0-rc.1); empty for a release. */
  prerelease: string[];
}

const SEMVER = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z.-]+)?$/;

/** A semantic version (https://semver.org) with an optional leading "v"; build metadata is dropped. Null for anything else. */
export function parseVersion(text: string): Version | null {
  const match = SEMVER.exec(text.trim());
  if (!match) return null;
  return { major: Number(match[1]), minor: Number(match[2]), patch: Number(match[3]), prerelease: match[4] ? match[4].split(".") : [] };
}

const isNumeric = (identifier: string): boolean => /^\d+$/.test(identifier);

/** Pre-release identifiers by semver precedence: numbers by value and before words, words by text, a shorter list before a longer one. */
function comparePrerelease(a: string[], b: string[]): number {
  // A release is newer than any pre-release of the same numbers.
  if (a.length === 0 || b.length === 0) return a.length === b.length ? 0 : a.length === 0 ? 1 : -1;
  for (let i = 0; i < Math.min(a.length, b.length); i++) {
    if (a[i] === b[i]) continue;
    const numeric = [isNumeric(a[i]), isNumeric(b[i])];
    if (numeric[0] && numeric[1]) return Number(a[i]) - Number(b[i]);
    if (numeric[0] !== numeric[1]) return numeric[0] ? -1 : 1;
    return a[i] < b[i] ? -1 : 1;
  }
  return a.length - b.length;
}

const compareParsed = (a: Version, b: Version): number =>
  a.major - b.major || a.minor - b.minor || a.patch - b.patch || comparePrerelease(a.prerelease, b.prerelease);

/** Negative when `a` is the older version, positive when it is the newer one, 0 when equal in precedence. Throws on a text that is no version. */
export function compareVersions(a: string, b: string): number {
  const left = parseVersion(a);
  const right = parseVersion(b);
  if (!left || !right) throw new Error(`Not a version: ${left ? b : a}`);
  return compareParsed(left, right);
}

/** Whether a day has passed since the last check (milliseconds since 1970; null = never). A time in the future counts as due: the clock was changed. */
export function isCheckDue(lastCheck: number | null, now: number): boolean {
  return lastCheck === null || lastCheck > now || now - lastCheck >= CHECK_INTERVAL_MS;
}

/** Whether the check at the start of the app runs: switched on in Settings, and not yet done today. */
export function shouldCheckAtStart(enabled: boolean, lastCheck: number | null, now: number): boolean {
  return enabled && isCheckDue(lastCheck, now);
}

export type CheckMode = "automatic" | "manual";

/** What a finished check does: open the dialog, tell that nothing is new, or do nothing. */
export type CheckOutcome = "show" | "upToDate" | "silent";

/**
 * `available` is the version the release feed offers (null: none newer than this one), `skipped` the version of "Skip This Version".
 * A manual check always shows what it found; an automatic one stays silent about the skipped version and anything older.
 */
export function checkOutcome(mode: CheckMode, available: string | null, skipped: string | null): CheckOutcome {
  if (available === null) return mode === "manual" ? "upToDate" : "silent";
  if (mode === "manual") return "show";
  const found = parseVersion(available);
  const skip = skipped === null ? null : parseVersion(skipped);
  return found && skip && compareParsed(found, skip) <= 0 ? "silent" : "show";
}

/** Why "Install and Restart" is not possible now (shown as its tooltip), or null when it is. */
export function installBlock(busy: boolean, supported: boolean): string | null {
  if (!supported) return "This installation cannot update itself. Download the new version from the release page.";
  if (busy) return "A calculation is running. Wait until it has finished, or cancel it.";
  return null;
}

/** The note for a manual check that failed, for example because the computer is offline. */
export function checkFailureText(error: unknown): string {
  return `Could not check for updates: ${error instanceof Error ? error.message : String(error)}`;
}
