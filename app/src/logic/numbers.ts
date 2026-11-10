// Number fields: parsing, formatting and stepping (docs/design.md, "Zahlenfeld").

/** Formats with a fixed number of decimals; the decimal separator is always a point. */
export function formatNumber(value: number, decimals: number): string {
  return value.toFixed(decimals);
}

/** Reads what the user typed; null for anything that is not a number. */
export function parseNumber(text: string): number | null {
  const trimmed = text.trim();
  if (!/^[+-]?(\d+\.?\d*|\.\d+)$/.test(trimmed)) return null;
  return Number(trimmed);
}

export function clampNumber(value: number, min?: number, max?: number): number {
  let v = value;
  if (min !== undefined) v = Math.max(min, v);
  if (max !== undefined) v = Math.min(max, v);
  return v;
}

/** One click on an arrow key: moves by `step` and rounds to the shown decimals so 0.1 + 0.2 stays 0.3. */
export function stepNumber(value: number, direction: 1 | -1, step: number, decimals: number, min?: number, max?: number): number {
  return clampNumber(Number((value + direction * step).toFixed(decimals)), min, max);
}
