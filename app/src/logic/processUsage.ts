// CPU load and memory of the app for the left end of the status bar (RK-019). The same rules and texts as the Mac app
// (mac/App/Monitor/ProcessUsage.swift), which takes them from Siderea, so the readout looks the same across the suite.

/** One reading of the process: CPU time used so far, memory held, and when the reading was taken. */
export interface ProcessSample {
  /** User plus system CPU time since the process started, in seconds. */
  cpuSeconds: number;
  /** Memory the process holds (Windows: working set), in bytes. */
  memoryBytes: number;
  /** Monotonic clock at the time of the reading, in seconds. Only differences are meaningful. */
  uptime: number;
}

/** Share of all cores at which the load counts as high. */
export const HIGH_LOAD_FRACTION = 0.9;

const isReading = (value: number | null): value is number => value !== null && Number.isFinite(value) && value >= 0;

/** CPU load between two readings in percent of one core (four busy cores read 400); null when they are not in order. */
export function cpuPercent(earlier: ProcessSample, later: ProcessSample): number | null {
  const elapsed = later.uptime - earlier.uptime;
  const used = later.cpuSeconds - earlier.cpuSeconds;
  if (elapsed <= 0 || used < 0) return null;
  return (used / elapsed) * 100;
}

/**
 * The CPU text: the share of the whole machine, whole percent, padded to three digits so that it does not jitter ("  7 %",
 * " 50 %", "100 %"), capped at 100. Null where there is no reading; the status bar shows a dash then.
 */
export function cpuReading(percent: number | null, coreCount: number): string | null {
  if (!isReading(percent) || coreCount <= 0) return null;
  return `${String(Math.round(Math.min(percent / coreCount, 100))).padStart(3, " ")} %`;
}

/** Memory in B, KB, MB, GB, TB counted in 1024s: no decimals up to 1 KB and from 100 on, one decimal in between. */
export function byteText(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = Math.max(bytes, 0);
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  if (unit === 0) return `${Math.floor(value)} B`;
  const tenths = (v: number) => Math.round(v * 10) / 10;
  // Rounding can reach the next unit (1023.96 KB reads 1024.0): it shows 1.0 of that one then.
  if (tenths(value) >= 1024 && unit < units.length - 1) {
    value = tenths(value) / 1024;
    unit += 1;
  }
  return `${value.toFixed(tenths(value) >= 100 ? 0 : 1)} ${units[unit]}`;
}

/** The load is high when the process keeps nearly every core busy; no reading or no cores is never high. */
export const isHigh = (percent: number | null, coreCount: number): boolean =>
  isReading(percent) && coreCount > 0 && percent >= HIGH_LOAD_FRACTION * 100 * coreCount;

/** Measuring costs a little, so it runs only while the readout is shown and the window is visible. */
export const isMeasuring = (shown: boolean, windowVisible: boolean): boolean => shown && windowVisible;
