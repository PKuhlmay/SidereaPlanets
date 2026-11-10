// Shared state of the window. Components read and change it directly; the computing steps of later tickets
// write their results into `session` and `status` (see logic/actions.ts).

import { reactive } from "vue";
import { createSettings } from "./logic/settings";
import type { Step, Tab } from "./logic/workflow";
import type { Theme } from "./logic/settings";

export interface RecordingInfo {
  format: string;
  codec: string | null;
  width: number;
  height: number;
  bitDepth: number;
  frameCount: number;
  color: string;
  fps: number | null;
  observer: string | null;
  instrument: string | null;
  telescope: string | null;
}

export interface Alignpoint {
  x: number;
  y: number;
  /** Mean shift of the point after Limit (green stroke); unknown before. */
  shiftX?: number;
  shiftY?: number;
  /** Whether the point is in use (checkbox in the Alignmentpoints card). */
  checked: boolean;
}

export interface FrameEntry {
  index: number;
  checked: boolean;
  /** Set by Align, in percent; the Framelist is sorted by it. */
  quality: number | null;
  difference: number | null;
}

export interface Busy {
  /** Name shown in the window title, e.g. "Align". */
  name: string;
  paused: boolean;
  cancelled: boolean;
}

export const settings = reactive(createSettings());

export const session = reactive({
  info: null as RecordingInfo | null,
  fileName: "",
  step: "empty" as Step,
  tab: "align" as Tab,
  frame: 0,
  frameImage: null as ImageData | null,
  /** The stacked image (RK-006) and the sharpened one (RK-007). */
  stackImage: null as ImageData | null,
  processedImage: null as ImageData | null,
  /** Number of stacked frames, for the position display of the Wavelet tab. */
  stackSize: null as number | null,
  alignpoints: [] as Alignpoint[],
  /** How many points satisfy the spacing rules before the threshold: the upper end of the Threshold slider (0 = none set). */
  alignpointTotal: 0,
  frames: [] as FrameEntry[],
});

// The status bar: five fields (docs/design.md, "Statusleiste").
export const status = reactive({
  busy: null as Busy | null,
  /** 0..1 */
  progress: 0,
  action: "",
  result: "",
  position: "",
  timeUsed: 0,
  timeLeft: 0,
});

export const theme = reactive({ current: "dark" as Theme });
