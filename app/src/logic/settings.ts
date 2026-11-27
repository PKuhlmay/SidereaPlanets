// All adjustable values with their defaults (docs/ui.md). Typed values outside the usual
// ranges are accepted as they are; a limit is only enforced where docs/ui.md names one.

import { defaultGeometry } from "./geometry";

export type Intensity = "default" | "area3x3" | "lowest";
export type LimitMode = "lowestQuality" | "bestPercent" | "framesPerPoint" | "bestCount";
export type StackSize = "default" | "minimized" | "maximized" | "roi";
export type ResampleMethod = "bell" | "bspline" | "lanczos" | "mitchell";
export type AlignedSize = "source" | "minimum" | "maximum" | "roi";
export type IntegratedMethod = "moving" | "compressed";
export type WaveletScheme = "dyadic" | "linear";
export type WaveletFilter = "default" | "gaussian";
export type ProcessingArea = 2048 | 1024 | 512 | 256 | 128;
export type StretchFit = "ask" | "yes" | "no";
export type Theme = "dark" | "light";
export type Planet = "jupiter" | "saturn" | "mars";

export interface WaveletLayer {
  enabled: boolean;
  denoise: number;
  sharpen: number;
  /** Slider position, -50 .. 1000; the Preview button shows it divided by 10 (10 = 1.0). */
  level: number;
}

export const LAYER_COUNT = 6;
export const LAYER_RANGE = { min: -50, max: 1000 };
export const LUMINANCE = { red: 0.299, green: 0.587, blue: 0.114 };

// Contrast/Brightness: the sliders start at 250 and 0 while the fields show 100 and 0. The ratio 2.5 follows
// from that one pair of values and is a design decision (a linear mapping). The engine works on the field value.
export const CONTRAST_RATIO = 2.5;
export const CONTRAST_RANGE = { min: 1, max: 1000 };
export const BRIGHTNESS_RANGE = { min: -512, max: 512 };

/** The Preview button of a layer. */
export const formatPreview = (level: number): string => (level / 10).toFixed(1);

export const functionNames = [
  "Histogram", "Gamma", "Color Mixing",
  "View Zoomed", "View Compare", "View Stack Size",
  "Flip and Rotate", "RGB Align", "RGB Balance",
  "Resize Image", "Denoise/Deringing", "Wavelet Filter",
  "Masking", "Show Line Graph", "Crop",
] as const;

export const codecs = ["DIB", "cvid", "i420", "iyuv", "mrle", "msvc", "uyvy", "yuy2", "yvu9", "yvyu"] as const;

export interface Shift {
  x: number;
  y: number;
}

export interface ChannelShifts {
  red: Shift;
  green: Shift;
  blue: Shift;
}

/** The values of the cards behind the Functions buttons (docs/design.md, "Karten hinter den Functions-Schaltflächen"). */
export interface ToneSettings {
  /** RGB Align: shift of every channel in pixels. */
  shifts: ChannelShifts;
  /** RGB Align, Auto: how far the search goes, in pixels. */
  maxShift: number;
  /** RGB Balance in percent, 100 = unchanged. */
  red: number;
  green: number;
  blue: number;
  /** Histogram: black and white point on a 0..255 scale, the share of pixels Auto cuts off at each end, and the graph shown. */
  black: number;
  white: number;
  clip: number;
  histogramView: "input" | "result";
  /** 1.0 = unchanged. */
  gamma: number;
  /** Color Mixing: output channel `row` is the sum of red, green and blue times `matrix[row * 3 + column]`, in percent. */
  mix: ColorMixValues;
}

export interface ColorMixValues {
  matrix: number[];
  /** Keeps the brightness of every pixel while the colors change. */
  preserveBrightness: boolean;
}

/** The identity of Color Mixing: 100 on the diagonal leaves the image as it is. */
export const IDENTITY_MIX = [100, 0, 0, 0, 100, 0, 0, 0, 100];

/** The cards Denoise/Deringing and Masking (docs/wavelets.md), in the units of the cards. Defaults as `RefineSettings` of the core. */
export type RefineValues = {
  /** Denoise and Deringing in percent, 0 = off. */
  denoise: number;
  deringing: number;
  /** How far from an edge Deringing looks for the dark ring, in pixels. */
  radius: number;
  maskOn: boolean;
  /** Brightness of the stack (0..255) above which the image is sharpened. */
  threshold: number;
  /** Width of the soft edge of the mask, in pixels. */
  feather: number;
  invert: boolean;
};

export const REFINE_LIMITS = {
  percent: { min: 0, max: 100 },
  radius: { min: 1, max: 10 },
  threshold: { min: 0, max: 255 },
  feather: { min: 0, max: 50 },
};

export const defaultRefine = (): RefineValues => ({ denoise: 0, deringing: 0, radius: 4, maskOn: false, threshold: 20, feather: 4, invert: false });

export const TONE_LIMITS = {
  shift: 20,
  maxShift: { min: 1, max: 20 },
  balance: { min: 0, max: 200 },
  mix: { min: -200, max: 200 },
  gamma: { min: 0.1, max: 5 },
  level: { min: 0, max: 255 },
  clip: { min: 0, max: 49 },
};

const noShift = (): Shift => ({ x: 0, y: 0 });
export const defaultShifts = (): ChannelShifts => ({ red: noShift(), green: noShift(), blue: noShift() });

export const defaultTone = (): ToneSettings => ({
  shifts: defaultShifts(),
  maxShift: 5,
  red: 100,
  green: 100,
  blue: 100,
  black: 0,
  white: 255,
  clip: 0.1,
  histogramView: "input",
  gamma: 1,
  mix: { matrix: [...IDENTITY_MIX], preserveBrightness: false },
});

export const defaultLayer = (): WaveletLayer => ({ enabled: true, denoise: 0, sharpen: 0.1, level: 10 });

export function createSettings() {
  return {
    /** Placeholder until the backend reports the number of cores; compute.ts then sets all of them. */
    cpus: 1,
    menu: {
      darkframeFactor: 1,
      recordInFilename: false,
      autodetectColour: false,
      processingArea: 512 as ProcessingArea,
      stretchFit: "ask" as StretchFit,
      advancedControls: false,
      playSounds: false,
      swapRedBlue: false,
      waitSingleImage: false,
      maximizeMessages: false,
      saveToSubdir: "",
      biasSubtract: 0,
      automatic: { alignToWavelet: false, alignToStack: false, stackToWavelet: false },
      luminance: { ...LUMINANCE },
    },
    // Colour and LRGB appear in the header of every tab; they belong together, so one value serves all.
    colour: true,
    lrgb: false,
    align: {
      showFullImage: false,
      showRoi: false,
      showAlignpoints: true,
      showAligndata: false,
      showFramelist: false,
      showRegistrationgraph: false,
      showPrefilter: false,
      minDistance: 30,
      edgeDistance: 20,
      intensity: "default" as Intensity,
      lo: 30,
      hi: 230,
      /** Slider position: how many of the weakest points are dropped (docs/workflow.md, 2.1). */
      threshold: 0,
      showAlignment: false,
      boxSize: 30,
      maxMovement: 5,
      /** Align by center of gravity and its Brightness threshold (0–255), off by default. */
      centreOfGravity: false,
      brightnessThreshold: 20,
      limitMode: "lowestQuality" as LimitMode,
      limitPercent: 80,
      limitCount: 200,
    },
    stack: {
      showFullImage: true,
      showAlignSections: true,
      showStackgraph: false,
      showStacklist: false,
      resampling: false,
      resampleMethod: "lanczos" as ResampleMethod,
      resampleFactor: 2.0,
      drizzling: false,
      drizzleFactor: 0.5,
      stackSize: "default" as StackSize,
      nearestPoints: 3,
      normalize: false,
      correctGeometry: true,
      stretchHistogram: false,
      derotate: false,
      /** De-rotate images: the planet, the camera angle in degrees counter-clockwise (0 = north up), a mirror image,
       *  and when the first frame was taken (seconds since 1970 UTC; used only when the file has no time per frame). */
      planet: "jupiter" as Planet,
      cameraAngle: 0,
      mirrored: false,
      startTime: null as number | null,
      edgePixels: 0,
      codec: "DIB" as string,
      compressQuality: 7500,
      framerate: 25,
      alignedSize: "source" as AlignedSize,
      sortOnQuality: false,
      usePrefilters: false,
      integratedMethod: "moving" as IntegratedMethod,
      averageFrames: 5,
    },
    wavelet: {
      showFullImage: false,
      showAlignpoints: false,
      showProcessingArea: false,
      automatic: false,
      hold: false,
      scheme: "linear" as WaveletScheme,
      initialLayer: 1,
      stepIncrement: 0,
      filter: "gaussian" as WaveletFilter,
      linked: false,
      layers: Array.from({ length: LAYER_COUNT }, defaultLayer),
      functionsOpen: true,
      /** Pressed Functions buttons in the order they were pressed; each opens its own card. Cards exist for `toneCards` (logic/colour.ts). */
      functions: [] as string[],
      contrast: 250,
      brightness: 0,
      holdContrast: false,
      tone: defaultTone(),
    },
    /** Denoise/Deringing and Masking: applied to the sharpened image before all other cards; the section `refine` of the project file. */
    refine: defaultRefine(),
    /** Flip and Rotate and Resize Image: the last steps of Save Image; the section `geometry` of the project file. */
    geometry: defaultGeometry(),
  };
}

/** The Contrast field shows the slider position divided by the ratio (250 -> 100). */
export const contrastField = (slider: number): number => Math.round(slider / CONTRAST_RATIO);
export const contrastSlider = (field: number): number =>
  Math.min(CONTRAST_RANGE.max, Math.max(CONTRAST_RANGE.min, Math.round(field * CONTRAST_RATIO)));
