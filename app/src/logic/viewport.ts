// Zoom and pan of the image area. A view maps image pixels to the area: screen = image * scale + (x, y).

export interface View {
  scale: number;
  x: number;
  y: number;
}
export interface Size {
  w: number;
  h: number;
}

export const MARGIN = 12;
export const MIN_SCALE = 0.02;
export const MAX_SCALE = 32;

const clampScale = (s: number) => Math.min(MAX_SCALE, Math.max(MIN_SCALE, s));

/** The whole image centred in the area with a margin around it. */
export function fitView(box: Size, image: Size, margin = MARGIN): View {
  const scale = clampScale(Math.min((box.w - 2 * margin) / image.w, (box.h - 2 * margin) / image.h));
  return { scale, x: (box.w - image.w * scale) / 2, y: (box.h - image.h * scale) / 2 };
}

/**
 * The view while a drag moves the image: `grab` is where the drag started (pointer minus view position). Returns
 * null while the pointer has not left its starting place, so that a plain click does not count as moving the image
 * (a moved image no longer follows the size of the area).
 */
export function dragView(view: View, grab: { x: number; y: number }, pointer: { x: number; y: number }): View | null {
  const x = pointer.x - grab.x;
  const y = pointer.y - grab.y;
  return x === view.x && y === view.y ? null : { ...view, x, y };
}

/** Zooms by `factor`, keeping the image point under (px, py) in place. */
export function zoomAt(view: View, factor: number, px: number, py: number): View {
  const scale = clampScale(view.scale * factor);
  const k = scale / view.scale;
  return { scale, x: px - (px - view.x) * k, y: py - (py - view.y) * k };
}

/** Shows the image at 100 % of its pixels, around the centre of the area. */
export function actualSize(view: View, box: Size): View {
  return zoomAt(view, 1 / view.scale, box.w / 2, box.h / 2);
}

export const formatZoom = (scale: number): string => `${Math.round(scale * 100)}%`;

/** Image pixel under a point of the area, or null outside the image. */
export function imagePixel(view: View, image: Size, px: number, py: number): { x: number; y: number } | null {
  const x = Math.floor((px - view.x) / view.scale);
  const y = Math.floor((py - view.y) / view.scale);
  return x >= 0 && y >= 0 && x < image.w && y < image.h ? { x, y } : null;
}

/** Position in image pixels under a point of the area, also outside the image and between pixels. */
export function imagePosition(view: View, px: number, py: number): { x: number; y: number } {
  return { x: (px - view.x) / view.scale, y: (py - view.y) / view.scale };
}

/** The square of the Processing Area in the middle of the image (smaller if the image is). */
export function processingArea(image: Size, side: number): { x: number; y: number; w: number; h: number } {
  const w = Math.min(side, image.w);
  const h = Math.min(side, image.h);
  return { x: Math.round((image.w - w) / 2), y: Math.round((image.h - h) / 2), w, h };
}
