// What the step Open says about a loaded recording (MAC-005): the rows of the file info, with tooltips on the technical labels.

import type { RecordingInfo } from "../state";

export interface RecordingRow {
  label: string;
  value: string;
}

/** "25 fps", "29.97 fps": up to two decimals, none for whole numbers. */
export const formatRate = (fps: number): string => `${Number.isInteger(fps) ? String(fps) : fps.toFixed(2)} fps`;

/** The rows of the file info; codec, frame rate, observer, instrument and telescope only where the file has them. */
export function recordingRows(info: RecordingInfo): RecordingRow[] {
  const rows: RecordingRow[] = [{ label: "Format", value: info.format }];
  if (info.codec) rows.push({ label: "Codec", value: info.codec });
  rows.push({ label: "Size", value: `${info.width}×${info.height}` });
  rows.push({ label: "Bit depth", value: `${info.bitDepth} bit` });
  rows.push({ label: "Frames", value: String(info.frameCount) });
  rows.push({ label: "Color", value: info.color });
  if (info.fps !== null) rows.push({ label: "Frame rate", value: formatRate(info.fps) });
  const extra: [string, string | null][] = [
    ["Observer", info.observer],
    ["Instrument", info.instrument],
    ["Telescope", info.telescope],
  ];
  for (const [label, value] of extra) if (value) rows.push({ label, value });
  return rows;
}

const explanations: Record<string, string> = {
  Codec: "How the frames are stored in the file.",
  "Bit depth": "Brightness levels per pixel.",
};

/** The tooltip of a row label that needs one. */
export const rowExplanation = (label: string): string | undefined => explanations[label];
