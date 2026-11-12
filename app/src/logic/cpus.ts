// The CPUs field of the toolbar: how many threads the calculations use (docs/ui.md). The backend runs them in a pool
// of that size; the number of logical cores is the default and the upper end.

import { wholeNumber } from "./numbers";

/** The number of logical cores as the backend reports it; anything that is not a usable number counts as one core. */
export function coreCount(reported: number): number {
  return Number.isFinite(reported) ? Math.max(1, Math.floor(reported)) : 1;
}

/** A typed or sent value as a whole number from 1 up to the number of cores. */
export function clampCpus(value: number, cores: number): number {
  return Math.min(coreCount(cores), wholeNumber(value, 1));
}
