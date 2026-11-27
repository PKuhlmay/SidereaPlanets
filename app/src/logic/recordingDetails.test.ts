import { describe, expect, it } from "vitest";
import { formatRate, recordingRows, rowExplanation } from "./recordingDetails";
import type { RecordingInfo } from "../state";

const info = (more: Partial<RecordingInfo> = {}): RecordingInfo => ({
  format: "AVI",
  codec: "DIB",
  width: 1024,
  height: 680,
  bitDepth: 8,
  frameCount: 10,
  color: "RGB",
  fps: 25,
  observer: null,
  instrument: null,
  telescope: null,
  hasTimestamps: false,
  startTime: null,
  startTimeFromFileDate: false,
  ...more,
});

describe("recordingRows (file info in the step Open)", () => {
  it("lists format, codec, size, bit depth, frames, color and frame rate", () => {
    expect(recordingRows(info())).toEqual([
      { label: "Format", value: "AVI" },
      { label: "Codec", value: "DIB" },
      { label: "Size", value: "1024×680" },
      { label: "Bit depth", value: "8 bit" },
      { label: "Frames", value: "10" },
      { label: "Color", value: "RGB" },
      { label: "Frame rate", value: "25 fps" },
    ]);
  });

  it("leaves out what the file does not have and adds observer, instrument and telescope where it has them", () => {
    const labels = recordingRows(info({ codec: null, fps: null, observer: "P. K.", telescope: "C11" })).map((row) => row.label);
    expect(labels).toEqual(["Format", "Size", "Bit depth", "Frames", "Color", "Observer", "Telescope"]);
    expect(recordingRows(info({ codec: "", instrument: "" })).map((row) => row.label)).not.toContain("Codec");
  });
});

describe("formatRate", () => {
  it("shows up to two decimals and none for whole numbers", () => {
    expect(formatRate(25)).toBe("25 fps");
    expect(formatRate(29.97)).toBe("29.97 fps");
  });
});

describe("rowExplanation", () => {
  it("explains the technical labels in a tooltip", () => {
    expect(rowExplanation("Codec")).toBe("How the frames are stored in the file.");
    expect(rowExplanation("Bit depth")).toBe("Brightness levels per pixel.");
    expect(rowExplanation("Frames")).toBeUndefined();
  });
});
