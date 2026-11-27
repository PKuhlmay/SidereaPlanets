// Which part of an image the viewer has to draw again. A preview changes the Processing Area only, so the viewer puts that
// rectangle onto the canvas instead of the whole image (docs/wavelets.md, Vorschau).

import type { Rect } from "./align";

/** The smallest rectangle that holds both. */
export function unionRect(a: Rect | undefined, b: Rect): Rect {
  if (!a) return b;
  const x = Math.min(a.x, b.x);
  const y = Math.min(a.y, b.y);
  return { x, y, width: Math.max(a.x + a.width, b.x + b.width) - x, height: Math.max(a.y + a.height, b.y + b.height) - y };
}

/**
 * The rectangles that changed in a pixel buffer since the viewer drew it last. The viewer does not draw every write (Vue
 * draws once for two writes in the same tick), so the rectangles pile up until `take`.
 */
export class ChangeLog {
  private pending = new WeakMap<object, Rect>();

  mark(buffer: object, rect: Rect): void {
    this.pending.set(buffer, unionRect(this.pending.get(buffer), rect));
  }

  take(buffer: object): Rect | undefined {
    const rect = this.pending.get(buffer);
    this.pending.delete(buffer);
    return rect;
  }
}

/** What is on a canvas, or what is to be put on it: the canvas element, the pixel buffer and the size of the image. */
export interface Drawn {
  surface: object;
  buffer: object;
  width: number;
  height: number;
}

/**
 * The part of `image` the viewer has to put on the canvas: only `changed` when the canvas still shows the same buffer at the
 * same size, else the whole image (`undefined`).
 */
export function drawRegion(last: Drawn | null, image: Drawn, changed: Rect | undefined): Rect | undefined {
  if (!changed || !last) return undefined;
  if (last.surface !== image.surface || last.buffer !== image.buffer || last.width !== image.width || last.height !== image.height) return undefined;
  const x = Math.max(0, changed.x);
  const y = Math.max(0, changed.y);
  return { x, y, width: Math.max(0, Math.min(changed.x + changed.width, image.width) - x), height: Math.max(0, Math.min(changed.y + changed.height, image.height) - y) };
}

/** The preview in the Wavelets step marks its writes here (wavelet.ts); the image viewer takes them. */
export const changes = new ChangeLog();
