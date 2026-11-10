import { describe, expect, it } from "vitest";
import {
  contrastField,
  contrastSlider,
  createSettings,
  formatPreview,
  functionNames,
  LAYER_COUNT,
  LAYER_RANGE,
} from "./settings";

// Defaults and ranges from docs/ui.md.
describe("defaults", () => {
  const s = createSettings();

  it("Align tab", () => {
    expect(s.align).toMatchObject({
      minDistance: 30,
      edgeDistance: 20,
      intensity: "default",
      lo: 30,
      hi: 230,
      boxSize: 30,
      maxMovement: 5,
      limitMode: "lowestQuality",
      limitPercent: 80,
      limitCount: 200,
      showAlignment: false,
    });
  });

  it("Stack tab", () => {
    expect(s.stack).toMatchObject({
      stackSize: "default",
      nearestPoints: 3,
      normalize: false,
      correctGeometry: true,
      stretchHistogram: false,
      derotate: false,
      edgePixels: 0,
      resampling: false,
      resampleMethod: "lanczos",
      resampleFactor: 2.0,
      drizzling: false,
      drizzleFactor: 0.5,
      codec: "DIB",
      compressQuality: 7500,
      framerate: 25,
      alignedSize: "original",
      sortOnQuality: false,
      usePrefilters: false,
      integratedMethod: "moving",
      averageFrames: 5,
    });
  });

  it("Wavelet tab", () => {
    expect(s.wavelet).toMatchObject({
      automatic: false,
      hold: false,
      scheme: "linear",
      initialLayer: 1,
      stepIncrement: 0,
      filter: "gaussian",
      linked: false,
      contrast: 250,
      brightness: 0,
    });
    expect(s.wavelet.layers).toHaveLength(LAYER_COUNT);
    for (const layer of s.wavelet.layers) {
      expect(layer).toEqual({ enabled: true, denoise: 0, sharpen: 0.1, level: 10 });
    }
  });

  it("menus", () => {
    expect(s.cpus).toBe(1);
    expect(s.menu).toMatchObject({
      processingArea: 512,
      stretchFit: "ask",
      luminance: { red: 0.299, green: 0.587, blue: 0.114 },
      biasSubtract: 0,
      darkframeFactor: 1,
      autodetectColour: false,
    });
    expect(s.menu.automatic).toEqual({ alignToWavelet: false, alignToStack: false, stackToWavelet: false });
  });

  it("gives every call its own layers, so editing one does not change the defaults", () => {
    const other = createSettings();
    other.wavelet.layers[0].level = 99;
    expect(createSettings().wavelet.layers[0].level).toBe(10);
  });
});

describe("wavelet layers", () => {
  it("have the slider range of the original", () => {
    expect(LAYER_RANGE).toEqual({ min: -50, max: 1000 });
  });

  it("show the slider value divided by ten on the Preview button", () => {
    expect(formatPreview(10)).toBe("1.0");
    expect(formatPreview(32)).toBe("3.2");
    expect(formatPreview(-50)).toBe("-5.0");
    expect(formatPreview(1000)).toBe("100.0");
  });
});

describe("Functions", () => {
  it("lists the fifteen buttons in the order of the original (three columns)", () => {
    expect(functionNames).toHaveLength(15);
    expect(functionNames.slice(0, 3)).toEqual(["Histogram", "Gamma", "Colour Mixing"]);
    expect(functionNames[functionNames.length - 1]).toBe("Cropping Area");
  });
});

describe("Contrast/Brightness", () => {
  it("maps the default slider position 250 to the field value 100", () => {
    expect(contrastField(250)).toBe(100);
    expect(contrastSlider(100)).toBe(250);
  });

  it("keeps the slider within 1 to 1000", () => {
    expect(contrastSlider(0)).toBe(1);
    expect(contrastSlider(1000)).toBe(1000);
  });
});
