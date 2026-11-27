import { describe, expect, it } from "vitest";
import { parsePreview, writePreview } from "./wavelet";

/** The answer of wavelet_preview for the area (1, 1, 3, 2): the header, then `pixels`. */
function answer(pixels: number[]): ArrayBuffer {
  const buffer = new ArrayBuffer(16 + pixels.length);
  const view = new DataView(buffer);
  [1, 1, 3, 2].forEach((v, i) => view.setUint32(i * 4, v, true));
  new Uint8Array(buffer, 16).set(pixels);
  return buffer;
}

const gray = [10, 20, 30, 40, 50, 60];
const rgba = gray.flatMap((v) => [v, v, v, 255]);

describe("a preview without colour", () => {
  it("has one byte per pixel and says so", () => {
    const p = parsePreview(answer(gray));
    expect(p.mono).toBe(true);
    expect(p.rgba).toHaveLength(6);
  });

  it("is told from a colour preview by its length", () => {
    expect(parsePreview(answer(rgba)).mono).toBe(false);
  });

  it("puts the same pixels into the image as the RGBA answer for the same area", () => {
    const fromGray = new Uint8ClampedArray(5 * 4 * 4);
    const fromRgba = new Uint8ClampedArray(5 * 4 * 4);
    writePreview(fromGray, 5, parsePreview(answer(gray)));
    writePreview(fromRgba, 5, parsePreview(answer(rgba)));
    expect(fromGray).toEqual(fromRgba);
    expect(fromGray[(1 * 5 + 1) * 4 + 3]).toBe(255);
    expect(fromGray[(2 * 5 + 3) * 4 + 1]).toBe(60);
  });
});
