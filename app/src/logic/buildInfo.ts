// Version, commit and build time of the running app, for the status bar (RK-025), like Siderea's BuildInfo: "0.1.0 · 3801a70 ·
// 2026-11-02 14:32". The values are written into the bundle when it is built (vite.config.ts: __APP_VERSION__, __APP_COMMIT__,
// __APP_BUILT__).

const two = (n: number) => String(n).padStart(2, "0");

/** "0.1.0 · 3801a70 · 2026-11-02 14:32", leaving out what is not known; the time in local time, to the minute. */
export function buildLabel(version: string, commit: string, builtAt: Date | null): string {
  const parts = [version];
  if (commit) parts.push(commit);
  if (builtAt) {
    parts.push(
      `${builtAt.getFullYear()}-${two(builtAt.getMonth() + 1)}-${two(builtAt.getDate())} ${two(builtAt.getHours())}:${two(builtAt.getMinutes())}`,
    );
  }
  return parts.join(" · ");
}

/** The label of this build. */
export const appBuildLabel = (): string => buildLabel(__APP_VERSION__, __APP_COMMIT__, __APP_BUILT__ ? new Date(__APP_BUILT__) : null);
