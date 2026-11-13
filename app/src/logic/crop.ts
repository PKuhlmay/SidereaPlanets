// The geometry of the crop rectangle (MAC-016), a port of mac/App/Model/CropRectangle.swift (which comes from Siderea): GUI-free and
// in whole image pixels, top-left origin. `minX`/`minY` are the first kept column and row, `maxX`/`maxY` the first one past the
// rectangle. Nothing here is ever outside the image or smaller than `MINIMUM_SIZE`. Rounding is half away from zero like Swift's.

/** The parts of a rectangle a pointer can take hold of. */
export type CropHandle = "topLeft" | "top" | "topRight" | "right" | "bottomRight" | "bottom" | "bottomLeft" | "left" | "body";

/** A fixed shape `long` : `short`; whether the rectangle lies or stands is not part of it. */
export interface CropAspect {
  long: number;
  short: number;
}

export interface CropRectangle {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

/** The smallest side a rectangle can be dragged down to. */
export const MINIMUM_SIZE = 8;

/** Null unless 0 < `short` ≤ `long`. */
export const cropAspect = (long: number, short: number): CropAspect | null => (short > 0 && long >= short ? { long, short } : null);

export const ASPECTS = {
  square: { long: 1, short: 1 },
  threeByTwo: { long: 3, short: 2 },
  fourByThree: { long: 4, short: 3 },
} satisfies Record<string, CropAspect>;

/** The Format choice of the crop tool: Free, or a fixed shape. */
export type CropFormat = "free" | "square" | "threeByTwo" | "fourByThree";

export const CROP_FORMATS: { id: CropFormat; title: string; aspect: CropAspect | null }[] = [
  { id: "free", title: "Free", aspect: null },
  { id: "square", title: "1:1", aspect: ASPECTS.square },
  { id: "threeByTwo", title: "3:2", aspect: ASPECTS.threeByTwo },
  { id: "fourByThree", title: "4:3", aspect: ASPECTS.fourByThree },
];

export const formatAspect = (format: CropFormat): CropAspect | null => CROP_FORMATS.find((f) => f.id === format)?.aspect ?? null;

/** Width ÷ height of a rectangle of this shape lying down (`portrait` false) or standing up. */
export function ratio(aspect: CropAspect, portrait: boolean): number {
  const landscape = aspect.long / aspect.short;
  return portrait ? 1 / landscape : landscape;
}

/** Rounds half away from zero, like Swift's `rounded()`. */
const round = (v: number): number => Math.sign(v) * Math.round(Math.abs(v));
const clamp = (value: number, low: number, high: number): number => Math.min(Math.max(value, low), Math.max(high, low));
const widthOf = (r: CropRectangle) => r.maxX - r.minX;
const heightOf = (r: CropRectangle) => r.maxY - r.minY;

export const rect = (minX: number, minY: number, maxX: number, maxY: number): CropRectangle => ({ minX, minY, maxX, maxY });

/** The rectangle that is the whole image. */
export const full = (imageWidth: number, imageHeight: number): CropRectangle => rect(0, 0, imageWidth, imageHeight);

/** `share` (0…1) of each side of the image, in the middle, reshaped to `aspect` when there is one. */
export function centred(share: number, imageWidth: number, imageHeight: number, aspect?: CropAspect | null): CropRectangle {
  const w = Math.max(round(imageWidth * share), MINIMUM_SIZE);
  const h = Math.max(round(imageHeight * share), MINIMUM_SIZE);
  const x = clamp(Math.trunc((imageWidth - w) / 2), 0, imageWidth - w);
  const y = clamp(Math.trunc((imageHeight - h) / 2), 0, imageHeight - h);
  const r = rect(x, y, Math.min(x + w, imageWidth), Math.min(y + h, imageHeight));
  return aspect ? fitted(r, aspect, imageWidth, imageHeight) : r;
}

/** The rectangle held inside the image: sides between `MINIMUM_SIZE` and the image, then slid inside. */
export function clamped(r: CropRectangle, imageWidth: number, imageHeight: number): CropRectangle {
  const w = clamp(widthOf(r), Math.min(MINIMUM_SIZE, imageWidth), imageWidth);
  const h = clamp(heightOf(r), Math.min(MINIMUM_SIZE, imageHeight), imageHeight);
  const x = clamp(r.minX, 0, imageWidth - w);
  const y = clamp(r.minY, 0, imageHeight - h);
  return rect(x, y, x + w, y + h);
}

/** Whether the rectangle is the whole image, so there is nothing to crop. */
export const isFull = (r: CropRectangle, imageWidth: number, imageHeight: number): boolean =>
  r.minX <= 0 && r.minY <= 0 && r.maxX >= imageWidth && r.maxY >= imageHeight;

/** The rule-of-thirds lines in image pixels, not rounded: a display aid. */
export function thirdsLines(r: CropRectangle): { xs: number[]; ys: number[] } {
  const w = widthOf(r);
  const h = heightOf(r);
  return { xs: [r.minX + w / 3, r.minX + (w * 2) / 3], ys: [r.minY + h / 3, r.minY + (h * 2) / 3] };
}

/** The grab reach for `requested` image pixels, capped at a third of the shorter side so the middle stays the body. */
export const effectiveTolerance = (r: CropRectangle, requested: number): number => Math.min(requested, Math.min(widthOf(r), heightOf(r)) / 3);

/** The arm of the centre cross, capped at a quarter of the shorter side. Display only. */
export const centreCrossArm = (r: CropRectangle, requested: number): number => Math.min(requested, Math.min(widthOf(r), heightOf(r)) * 0.25);

/** What lies under the image position: a corner, an edge (along its own length), the inside, or nothing. Corners win over edges. */
export function handleAt(r: CropRectangle, x: number, y: number, requested: number): CropHandle | null {
  const t = effectiveTolerance(r, requested);
  const near = (a: number, b: number) => Math.abs(a - b) <= t;
  const alongX = x >= r.minX - t && x <= r.maxX + t;
  const alongY = y >= r.minY - t && y <= r.maxY + t;
  if (near(r.minX, x) && near(r.minY, y)) return "topLeft";
  if (near(r.maxX, x) && near(r.minY, y)) return "topRight";
  if (near(r.maxX, x) && near(r.maxY, y)) return "bottomRight";
  if (near(r.minX, x) && near(r.maxY, y)) return "bottomLeft";
  if (near(r.minY, y) && alongX) return "top";
  if (near(r.maxY, y) && alongX) return "bottom";
  if (near(r.minX, x) && alongY) return "left";
  if (near(r.maxX, x) && alongY) return "right";
  if (x > r.minX && x < r.maxX && y > r.minY && y < r.maxY) return "body";
  return null;
}

const LEFT: CropHandle[] = ["left", "topLeft", "bottomLeft"];
const RIGHT: CropHandle[] = ["right", "topRight", "bottomRight"];
const TOP: CropHandle[] = ["top", "topLeft", "topRight"];
const BOTTOM: CropHandle[] = ["bottom", "bottomLeft", "bottomRight"];

/**
 * The rectangle with `handle` moved by (`dx`, `dy`) image pixels: an edge or corner moves only its edges, stopping at the image and
 * `MINIMUM_SIZE` short of the opposite edge; the body moves as a whole and stays inside. With `aspect` the shape is kept.
 */
export function dragged(
  r: CropRectangle,
  handle: CropHandle,
  dx: number,
  dy: number,
  imageWidth: number,
  imageHeight: number,
  aspect?: CropAspect | null,
): CropRectangle {
  if (aspect && handle !== "body") return resized(r, handle, dx, dy, imageWidth, imageHeight, aspect);
  const stepX = round(dx);
  const stepY = round(dy);
  const w = widthOf(r);
  const h = heightOf(r);
  if (handle === "body") {
    const minX = clamp(r.minX + stepX, 0, Math.max(imageWidth - w, 0));
    const minY = clamp(r.minY + stepY, 0, Math.max(imageHeight - h, 0));
    return rect(minX, minY, minX + w, minY + h);
  }
  const result = { ...r };
  if (LEFT.includes(handle)) result.minX = clamp(r.minX + stepX, 0, r.maxX - MINIMUM_SIZE);
  if (RIGHT.includes(handle)) result.maxX = clamp(r.maxX + stepX, Math.min(r.minX + MINIMUM_SIZE, imageWidth), imageWidth);
  if (TOP.includes(handle)) result.minY = clamp(r.minY + stepY, 0, r.maxY - MINIMUM_SIZE);
  if (BOTTOM.includes(handle)) result.maxY = clamp(r.maxY + stepY, Math.min(r.minY + MINIMUM_SIZE, imageHeight), imageHeight);
  return result;
}

/** The rectangle between two image positions, kept inside the image; null while a side is below `MINIMUM_SIZE`. */
export function drawn(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  imageWidth: number,
  imageHeight: number,
  aspect?: CropAspect | null,
): CropRectangle | null {
  if (aspect) return drawnFixed(x0, y0, x1, y1, imageWidth, imageHeight, aspect);
  const ax = clamp(round(x0), 0, imageWidth);
  const bx = clamp(round(x1), 0, imageWidth);
  const ay = clamp(round(y0), 0, imageHeight);
  const by = clamp(round(y1), 0, imageHeight);
  const r = rect(Math.min(ax, bx), Math.min(ay, by), Math.max(ax, bx), Math.max(ay, by));
  return widthOf(r) >= MINIMUM_SIZE && heightOf(r) >= MINIMUM_SIZE ? r : null;
}

const isPortrait = (r: CropRectangle) => heightOf(r) > widthOf(r);

/** Reshaped to `aspect` with the same area and centre and orientation, cut only where the image is too small, slid inside. */
export function fitted(r: CropRectangle, aspect: CropAspect, imageWidth: number, imageHeight: number): CropRectangle {
  const k = ratio(aspect, isPortrait(r));
  const smallest = Math.max(MINIMUM_SIZE, MINIMUM_SIZE * k);
  const reshapedWidth = Math.min(Math.max(Math.sqrt(widthOf(r) * heightOf(r) * k), smallest), imageWidth, imageHeight * k);
  const w = Math.max(round(reshapedWidth), 1);
  const h = Math.max(round(reshapedWidth / k), 1);
  const x = clamp(round((r.minX + r.maxX) / 2 - w / 2), 0, imageWidth - w);
  const y = clamp(round((r.minY + r.maxY) / 2 - h / 2), 0, imageHeight - h);
  return rect(x, y, x + w, y + h);
}

function resized(
  r: CropRectangle,
  handle: CropHandle,
  dx: number,
  dy: number,
  imageWidth: number,
  imageHeight: number,
  aspect: CropAspect,
): CropRectangle {
  const k = ratio(aspect, isPortrait(r));
  const smallest = Math.max(MINIMUM_SIZE, MINIMUM_SIZE * k);
  const movesLeft = LEFT.includes(handle);
  const movesRight = RIGHT.includes(handle);
  const movesTop = TOP.includes(handle);
  const movesBottom = BOTTOM.includes(handle);

  if ((movesLeft || movesRight) && (movesTop || movesBottom)) {
    // The corner that stays, and which way the rectangle grows from it.
    const anchorX = movesLeft ? r.maxX : r.minX;
    const anchorY = movesTop ? r.maxY : r.minY;
    const signX = movesLeft ? -1 : 1;
    const signY = movesTop ? -1 : 1;
    const pointerX = (movesLeft ? r.minX : r.maxX) + dx;
    const pointerY = (movesTop ? r.minY : r.maxY) + dy;
    let w = Math.max((pointerX - anchorX) * signX, (pointerY - anchorY) * signY * k, smallest);
    w = Math.min(w, signX > 0 ? imageWidth - anchorX : anchorX, (signY > 0 ? imageHeight - anchorY : anchorY) * k);
    const wi = round(w);
    const hi = round(wi / k);
    return rect(
      signX > 0 ? anchorX : anchorX - wi,
      signY > 0 ? anchorY : anchorY - hi,
      signX > 0 ? anchorX + wi : anchorX,
      signY > 0 ? anchorY + hi : anchorY,
    );
  }
  if (movesLeft || movesRight) {
    const anchorX = movesLeft ? r.maxX : r.minX;
    const signX = movesLeft ? -1 : 1;
    const centreY = (r.minY + r.maxY) / 2;
    let w = Math.max(((movesLeft ? r.minX : r.maxX) + dx - anchorX) * signX, smallest);
    w = Math.min(w, signX > 0 ? imageWidth - anchorX : anchorX, 2 * Math.min(centreY, imageHeight - centreY) * k);
    const wi = round(w);
    const hi = round(wi / k);
    const y = clamp(round(centreY - hi / 2), 0, imageHeight - hi);
    return rect(signX > 0 ? anchorX : anchorX - wi, y, signX > 0 ? anchorX + wi : anchorX, y + hi);
  }
  const anchorY = movesTop ? r.maxY : r.minY;
  const signY = movesTop ? -1 : 1;
  const centreX = (r.minX + r.maxX) / 2;
  let h = Math.max(((movesTop ? r.minY : r.maxY) + dy - anchorY) * signY, smallest / k);
  h = Math.min(h, signY > 0 ? imageHeight - anchorY : anchorY, (2 * Math.min(centreX, imageWidth - centreX)) / k);
  const hi = round(h);
  const wi = round(hi * k);
  const x = clamp(round(centreX - wi / 2), 0, imageWidth - wi);
  return rect(x, signY > 0 ? anchorY : anchorY - hi, x + wi, signY > 0 ? anchorY + hi : anchorY);
}

/** A new rectangle of a fixed shape, lying or standing as the drag is wider or taller, as large as the drag and the image allow. */
function drawnFixed(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  imageWidth: number,
  imageHeight: number,
  aspect: CropAspect,
): CropRectangle | null {
  const startX = clamp(round(x0), 0, imageWidth);
  const startY = clamp(round(y0), 0, imageHeight);
  const endX = clamp(round(x1), 0, imageWidth);
  const endY = clamp(round(y1), 0, imageHeight);
  const acrossX = Math.abs(endX - startX);
  const acrossY = Math.abs(endY - startY);
  const k = ratio(aspect, acrossY > acrossX);
  const growsRight = endX >= startX;
  const growsDown = endY >= startY;
  let w = Math.max(acrossX, acrossY * k);
  w = Math.min(w, growsRight ? imageWidth - startX : startX, (growsDown ? imageHeight - startY : startY) * k);
  const wi = round(w);
  const hi = round(wi / k);
  if (wi < MINIMUM_SIZE || hi < MINIMUM_SIZE) return null;
  return rect(growsRight ? startX : startX - wi, growsDown ? startY : startY - hi, growsRight ? startX + wi : startX, growsDown ? startY + hi : startY);
}

/** "1200 × 900 px": the size of the picture that is written. */
export const sizeText = (r: CropRectangle): string => `${widthOf(r)} × ${heightOf(r)} px`;

/** The rectangle as `save_image` takes it (`CropRect` of the core). */
export const cropToCore = (r: CropRectangle) => ({ x: r.minX, y: r.minY, width: widthOf(r), height: heightOf(r) });
