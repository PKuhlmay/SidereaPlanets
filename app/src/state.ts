// Shared state of the window. Components read and change it directly; the computing steps of later tickets
// write their results into `session` and `status` (see logic/actions.ts).

import { reactive } from "vue";
import type { Rect } from "./logic/align";
import type { CropFormat, CropRectangle } from "./logic/crop";
import type { DerotationInfo } from "./logic/derotation";
import { createSettings } from "./logic/settings";
import type { Step, ViewStep } from "./logic/workflow";
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
  /** The file has a time for every frame (SER trailer): De-rotate images needs no start time. */
  hasTimestamps: boolean;
  /** When the recording started, seconds since 1970 UTC: from the file, else guessed from the file date. */
  startTime: number | null;
  /** `startTime` is a guess from the date of the file. */
  startTimeFromFileDate: boolean;
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
  /** The step the left panel shows (step bar, docs/design.md). */
  view: "open" as ViewStep,
  /** Save image wrote the sharpened image since the last Do All (the Save step is done); its file name. */
  saved: false,
  savedName: "",
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
  /** Which of them the image area shows in the Wavelets and Save steps (the Toggle group of the Copy To panel). */
  toneView: "current" as "current" | "clipboard" | "difference",
  /** Number of stacked frames, for the position display of the Wavelets and Save steps. */
  stackSize: null as number | null,
  alignpoints: [] as Alignpoint[],
  /** The frame the alignpoints were set on; Align and Stack register the other frames against it. */
  referenceFrame: 0,
  /** How many points satisfy the spacing rules before the threshold: the upper end of the Threshold slider (0 = none set). */
  alignpointTotal: 0,
  frames: [] as FrameEntry[],
  /** The region of interest of Set Alignpoints and Align, in image pixels (R.o.Interest); null = the whole frame. */
  roi: null as Rect | null,
  /** True while the next drag in the image draws the region. */
  roiDrawing: false,
  /** The frame Go to Best Frame jumped to, named by the note in the step Alignpoints; Set Alignpoints, Align or another recording forget it. */
  bestFrameHint: null as number | null,
  /** Masking, Show mask: the image area shows where the sharpening applies (`maskImage`). Only a view, never saved. */
  showMask: false,
  maskImage: null as ImageData | null,
  /** Before/After in the zoom bar of the Wavelets and Save steps: the stack without anything applied. */
  before: false,
  /** View Compare: where the divider sits, as a share of the width of the image area. Not stored in the project file. */
  compareDivider: 0.5,
  /**
   * The crop tool (MAC-016): the rectangle Save Image cuts out (`applied`) and, while the tool is open, the one being edited (`draft`),
   * both in pixels of the stack. Not stored in the project file; a new stack or another recording drops it.
   */
  crop: { format: "free" as CropFormat, editing: false, applied: null as CropRectangle | null, draft: null as CropRectangle | null },
  /** Flip and Rotate: the pictures of the image area turned by the core, the shown one and the left half of View Compare (RK-022). */
  oriented: { shown: null as ImageData | null, before: null as ImageData | null },
  /** De-rotate images (RK-023): span, rotation and the disc measured in the reference frame; null while off or before Limit. */
  derotation: null as DerotationInfo | null,
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
