// Flip and Rotate and Resize Image in the window. The steps Wavelets and Save show
// the flipped and turned picture, so the crop rectangle is set on exactly what is on screen; Resize Image does not change the preview,
// its card and the step Save tell the size that is written. The core turns the pictures (`orient_preview`) and writes the file; the
// values and the sizes are in logic/geometry.ts.

import { invoke } from "@tauri-apps/api/core";
import { removeCrop } from "./cropTool";
import { failureText } from "./jobs";
import { isFull } from "./logic/crop";
import {
  geometryPayload,
  isOrientationNeutral,
  orientationLayout,
  orientedSize,
  resizeSize,
  type GeometryPayload,
  type OrientationValues,
  type Size,
} from "./logic/geometry";
import { session, settings, status } from "./state";

/** The size of the stack, before anything is flipped or turned. */
const stackSize = (): Size | null => (session.stackImage ? { width: session.stackImage.width, height: session.stackImage.height } : null);

/** The size of the picture after Flip and Rotate: the one the crop rectangle is set in; null before the stack exists. */
export function pictureSize(): Size | null {
  const size = stackSize();
  return size ? orientedSize(size.width, size.height, settings.geometry.orientation) : null;
}

/** The steps Wavelets and Save show the flipped and turned picture. */
export const orientationApplies = (): boolean =>
  session.stackImage !== null && !isOrientationNeutral(settings.geometry.orientation) && (session.view === "wavelets" || session.view === "save");

/** What Resize Image starts from: the crop rectangle (being edited, or else applied), or the whole turned picture. */
function geometrySource(rectangle: { minX: number; minY: number; maxX: number; maxY: number } | null): Size | null {
  const size = pictureSize();
  if (!size) return null;
  if (rectangle && !isFull(rectangle, size.width, size.height)) {
    return { width: rectangle.maxX - rectangle.minX, height: rectangle.maxY - rectangle.minY };
  }
  return size;
}

export const resizeSource = (): Size | null => geometrySource(session.crop.editing ? session.crop.draft : session.crop.applied);

/** The size of the file Save Image writes now; null before the stack exists. */
export function resultSize(): Size | null {
  const source = resizeSource();
  return source ? resizeSize(source, settings.geometry.resize) : null;
}

/** The geometry for the sharpened picture Save Image writes; null while it changes nothing. */
export function saveGeometry(): GeometryPayload | null {
  const source = geometrySource(session.crop.applied);
  return source ? geometryPayload(settings.geometry, source) : null;
}

/** Changes the orientation, unless the crop tool is open; a crop set on the old picture goes when the picture is turned or flipped. */
export function editOrientation(change: (o: OrientationValues) => void): void {
  if (session.crop.editing) {
    status.result = "Flip and Rotate: finish or cancel the crop first";
    return;
  }
  const before = orientationLayout(settings.geometry.orientation).join(",");
  change(settings.geometry.orientation);
  if (orientationLayout(settings.geometry.orientation).join(",") !== before && session.crop.applied) {
    removeCrop();
    status.result = "Crop removed: the picture was flipped or turned";
  }
}

// The newest request for each picture wins; an older answer that arrives late is dropped.
const requests = { shown: 0, before: 0 };

function decode(buffer: ArrayBuffer): ImageData {
  // Binary layout: width (u32 LE), height (u32 LE), then RGBA bytes.
  const view = new DataView(buffer);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(buffer, 8, width * height * 4), width, height);
}

/** Has the core turn `image` for the image area (the shown picture, or the left half of View Compare); none while nothing is turned. */
export async function orientImage(slot: "shown" | "before", image: ImageData | null): Promise<void> {
  const request = ++requests[slot];
  if (!image || !orientationApplies()) {
    session.oriented[slot] = null;
    return;
  }
  try {
    const bytes = new Uint8Array(image.data.buffer, image.data.byteOffset, image.data.byteLength);
    const buffer = await invoke<ArrayBuffer>("orient_preview", bytes, {
      headers: { width: String(image.width), height: String(image.height), orientation: JSON.stringify(settings.geometry.orientation) },
    });
    if (request === requests[slot]) session.oriented[slot] = decode(buffer);
  } catch (e) {
    if (request === requests[slot]) {
      session.oriented[slot] = null;
      status.result = failureText("Flip and Rotate", e);
    }
  }
}
