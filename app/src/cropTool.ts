// View Compare and the crop tool of the steps Wavelets and Save (SP-021, after MAC-016 and MAC-019). View Compare splits the image at a
// divider: the stack on the left, the result on the right. The crop tool sets the rectangle Save Image cuts out; it is opened by the Crop
// button of the Adjustments panel and by Crop… in the step Save. The two exclude each other, and View Compare excludes Before/After.
// The geometry of the rectangle is in logic/crop.ts.

import { centred, clamped, cropToCore, formatAspect, full, isFull, fitted, type CropFormat, type CropRectangle } from "./logic/crop";
import { orientedSize } from "./logic/geometry";
import { session, settings, status } from "./state";

/** The Adjustments button that switches the split view on and off, and the one that opens the crop tool. */
export const COMPARE = "View Compare";
export const CROP = "Crop";

/** The divider keeps a sliver of each side, so that it can always be grabbed and neither image disappears. */
export const DIVIDER_RANGE = { min: 0.02, max: 0.98 };

export const clampedDivider = (share: number): number =>
  Number.isFinite(share) ? Math.min(DIVIDER_RANGE.max, Math.max(DIVIDER_RANGE.min, share)) : 0.5;

/** Whether a label (Before or After) fits between the divider and its edge of an area `width` wide. */
export const compareLabelFits = (share: number, width: number, needed: number, side: "before" | "after"): boolean =>
  (side === "before" ? share * width : (1 - share) * width) >= needed;

export const compareIsOn = (): boolean => settings.wavelet.functions.includes(COMPARE);

/** The split view is shown: in the Wavelets step, with a stack, not with Before/After on and not while the crop tool is open. */
export const isComparing = (): boolean =>
  session.view === "wavelets" && compareIsOn() && session.stackImage !== null && !session.before && !session.crop.editing;

function closeCompare(): void {
  settings.wavelet.functions = settings.wavelet.functions.filter((name) => name !== COMPARE);
}

/** Before/After in the zoom bar: turning it on ends the split view. */
export function setBefore(on: boolean): void {
  if (on) closeCompare();
  session.before = on;
}

/** The View Compare button. While the crop tool is open it stays off. */
export function toggleCompare(): void {
  if (session.crop.editing) {
    status.result = "View Compare: finish or cancel the crop first";
    return;
  }
  if (compareIsOn()) {
    closeCompare();
  } else {
    settings.wavelet.functions = [...settings.wavelet.functions, COMPARE];
    session.before = false;
  }
}

/** The size of the picture the rectangle is set in: the stack after Flip and Rotate (SP-022); null before there is one. */
export function cropImageSize(): { width: number; height: number } | null {
  const stack = session.stackImage;
  return stack ? orientedSize(stack.width, stack.height, settings.geometry.orientation) : null;
}

/** Crop is possible where the sharpened stack is shown and while nothing runs. */
export const canCrop = (): boolean =>
  session.stackImage !== null && status.busy === null && (session.view === "wavelets" || session.view === "save");

/** Opens the tool on the applied rectangle, or on one in the middle (80 % of each side, in the shape of the format). */
export function startCrop(): void {
  const size = cropImageSize();
  if (!canCrop() || session.crop.editing || !size) return;
  closeCompare();
  session.crop.draft = session.crop.applied ?? centred(0.8, size.width, size.height, formatAspect(session.crop.format));
  session.crop.editing = true;
}

/** Cancel: closes the tool and drops what was changed since it opened. */
export function cancelCrop(): void {
  session.crop.editing = false;
  session.crop.draft = null;
}

/** Done: the rectangle being edited is what Save Image cuts out; one that is the whole picture is none. */
export function finishCrop(): void {
  if (!session.crop.editing) return;
  const draft = session.crop.draft;
  const size = cropImageSize();
  session.crop.applied = draft && size && !isFull(draft, size.width, size.height) ? draft : null;
  cancelCrop();
}

/** Reset: the whole picture again, in the format Free; the next drag on the image draws a new rectangle. */
export function resetCrop(): void {
  const size = cropImageSize();
  if (!session.crop.editing || !size) return;
  session.crop.format = "free";
  session.crop.draft = full(size.width, size.height);
}

/** Remove Crop: Save Image writes the whole picture again. */
export function removeCrop(): void {
  session.crop.applied = null;
}

/** Chooses the format; a rectangle that is there is reshaped (same centre and area). */
export function setCropFormat(format: CropFormat): void {
  session.crop.format = format;
  const aspect = formatAspect(format);
  const size = cropImageSize();
  if (session.crop.editing && aspect && session.crop.draft && size) {
    session.crop.draft = fitted(session.crop.draft, aspect, size.width, size.height);
  }
}

/** The rectangle the user dragged, kept inside the picture. */
export function setCropDraft(rectangle: CropRectangle): void {
  const size = cropImageSize();
  if (!session.crop.editing || !size) return;
  session.crop.draft = clamped(rectangle, size.width, size.height);
}

/** The stack the rectangle belongs to is gone or another one was made. */
export function clearCrop(): void {
  Object.assign(session.crop, { format: "free", editing: false, applied: null, draft: null });
}

/** The Crop button: opens the tool, or closes it like Cancel. */
export function toggleCrop(): void {
  if (session.crop.editing) cancelCrop();
  else startCrop();
}

/** What Save Image cuts out of the sharpened picture, as the core takes it; null for the whole picture. */
export function saveCrop(): { x: number; y: number; width: number; height: number } | null {
  const applied = session.crop.applied;
  const size = cropImageSize();
  if (!applied || !size || isFull(applied, size.width, size.height)) return null;
  return cropToCore(applied);
}
