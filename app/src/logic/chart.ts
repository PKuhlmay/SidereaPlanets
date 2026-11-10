// Line chart of the Registrationgraph.

export interface ChartBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** SVG points for one series, scaled to its own maximum; the first value sits left, the last right. */
export function seriesPoints(values: number[], box: ChartBox): string {
  const max = Math.max(...values, 0);
  if (values.length === 0 || max <= 0) return "";
  const dx = values.length > 1 ? box.width / (values.length - 1) : 0;
  return values
    .map((v, i) => `${(box.left + i * dx).toFixed(1)},${(box.top + box.height * (1 - v / max)).toFixed(1)}`)
    .join(" ");
}
