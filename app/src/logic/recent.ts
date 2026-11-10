// MRU: the files opened last, newest first.

export const MAX_RECENT = 10;

/** Moves `paths` to the front; the same selection is not listed twice. */
export function pushRecent(list: string[][], paths: string[]): string[][] {
  const same = (a: string[]) => a.length === paths.length && a.every((p, i) => p === paths[i]);
  return [paths, ...list.filter((entry) => !same(entry))].slice(0, MAX_RECENT);
}

export function fileName(path: string): string {
  return path.split(/[\/]/).pop() ?? path;
}

/** Menu text: the file name, plus the number of further files for a still-frame sequence. */
export function recentLabel(paths: string[]): string {
  const [first = ""] = paths;
  return paths.length > 1 ? `${fileName(first)} (+${paths.length - 1})` : fileName(first);
}
