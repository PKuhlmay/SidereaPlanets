// All adjustable values with the defaults observed in the reference program (docs/ui.md). Typed values outside the usual
// ranges are accepted as they are, as in the original; a limit is only enforced where docs/ui.md names one.

export type Intensity = "default" | "area3x3" | "lowest";
export type LimitMode = "lowestQuality" | "bestPercent" | "framesPerPoint" | "bestCount";
export type StackSize = "default" | "minimized" | "maximized" | "roi";
export type ResampleMethod = "bell" | "bspline" | "lanczos" | "mitchell";
export type AlignedSize = "original" | "minimum" | "maximum" | "roi";
export type IntegratedMethod = "moving" | "compressed";
export type WaveletScheme = "dyadic" | "linear";
export type WaveletFilter = "default" | "gaussian";
export type ProcessingArea = 2048 | 1024 | 512 | 256 | 128;
export type StretchFit = "ask" | "yes" | "no";
export type Theme = "dark" | "light";

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
// from those two defaults and is not confirmed against the original.
export const CONTRAST_RATIO = 2.5;
export const CONTRAST_RANGE = { min: 1, max: 1000 };
export const BRIGHTNESS_RANGE = { min: -512, max: 512 };

/** The Preview button of a layer. */
export const formatPreview = (level: number): string => (level / 10).toFixed(1);

export const functionNames = [
  "Histogram", "Gamma", "Colour Mixing",
  "View Zoomed", "View Compare", "View Stacksize",
  "Flip and Rotate", "RGB Align", "RGB Balance",
  "Resize Image", "Denoise/ Deringing", "Wavelet Filter",
  "Masking", "Show Linegraph", "Cropping Area",
] as const;

export const codecs = ["DIB", "cvid", "i420", "iyuv", "mrle", "msvc", "uyvy", "yuy2", "yvu9", "yvyu"] as const;

export const defaultLayer = (): WaveletLayer => ({ enabled: true, denoise: 0, sharpen: 0.1, level: 10 });

export function createSettings() {
  return {
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
    // Colour and LRGB appear in the header of every tab in the original; they belong together, so one value serves all.
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
      edgePixels: 0,
      codec: "DIB" as string,
      compressQuality: 7500,
      framerate: 25,
      alignedSize: "original" as AlignedSize,
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
      /** Pressed Functions buttons; every button opens its own panel (not built yet). */
      functions: [] as string[],
      contrast: 250,
      brightness: 0,
      holdContrast: false,
    },
  };
}

/** The Contrast field shows the slider position divided by the ratio (250 -> 100). */
export const contrastField = (slider: number): number => Math.round(slider / CONTRAST_RATIO);
export const contrastSlider = (field: number): number =>
  Math.min(CONTRAST_RANGE.max, Math.max(CONTRAST_RANGE.min, Math.round(field * CONTRAST_RATIO)));
