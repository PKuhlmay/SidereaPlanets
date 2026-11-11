// Shared state of the window. Components read and change it directly; the computing steps of later tickets
// write their results into `session` and `status` (see logic/actions.ts).

import { reactive } from "vue";
import type { Rect } from "./logic/align";
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
  /** The files of the open recording, as given to Select. */
  paths: [] as string[],
  step: "empty" as Step,
  tab: "align" as Tab,
  frame: 0,
  frameImage: null as ImageData | null,
  /** The stacked image (RK-006) and the sharpened one (RK-007). */
  stackImage: null as ImageData | null,
  processedImage: null as ImageData | null,
  /** The base image with the Functions cards and Contrast/Brightness applied (RK-008); null while they are all neutral. */
  adjustedImage: null as ImageData | null,
  /** Copy To: the image kept for comparison, and the difference to it. */
  clipboardImage: null as ImageData | null,
  differenceImage: null as ImageData | null,
  /** Which of them the image area shows in the Wavelet tab (the Toggle group of the Copy To panel). */
  toneView: "current" as "current" | "clipboard" | "difference",
  /** Number of stacked frames, for the position display of the Wavelet tab. */
  stackSize: null as number | null,
  alignpoints: [] as Alignpoint[],
  /** The frame the alignpoints were set on; Align and Stack register the other frames against it. */
  referenceFrame: 0,
  /** How many points satisfy the spacing rules before the threshold: the upper end of the Threshold slider (0 = none set). */
  alignpointTotal: 0,
  frames: [] as FrameEntry[],
  /** The region of interest of the Align tab, in image pixels (R.o.Interest); null = the whole frame. */
  roi: null as Rect | null,
  /** True while the next drag in the image draws the region. */
  roiDrawing: false,
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
