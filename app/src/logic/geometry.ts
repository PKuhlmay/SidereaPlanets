// The pure parts of the cards Flip and Rotate and Resize Image (SP-022, after mac/App/Model/GeometryLogic.swift): the values with their
// defaults and limits, the sizes they give, what Save Image sends and the section `geometry` of the project file. The picture itself is
// made in the Rust core (crates/core/src/geometry.rs; order: refine, colors, flip and rotate, crop, resize). `orientedSize` and
// `resizeSize` follow `Orientation::output_size` and `Resize::output_size` there, with the values of its tests in geometry.test.ts, so
// the cards can tell the size at once while a slider moves.

export interface Size {
  width: number;
  height: number;
}

export type RotationMethod = "bilinear" | "bicubic";
export type ResizeMethod = "lanczos3" | "bicubic" | "bilinear" | "nearest";

export interface OrientationValues {
  flipHorizontal: boolean;
  flipVertical: boolean;
  /** Clockwise in degrees, -180 … 180 in tenths. Multiples of 90 turn exactly, anything between is resampled. */
  angle: number;
  /** Cuts a freely turned picture to the largest part of the original shape without black corners. */
  cropToFit: boolean;
  method: RotationMethod;
}

export interface ResizeValues {
  mode: "percent" | "pixels";
  percent: number;
  /** In pixels; 0 means "not typed yet": the picture keeps its size. */
  width: number;
  height: number;
  /** With the lock the side typed last (`driver`) decides and the other follows. */
  lockAspect: boolean;
  driver: "width" | "height";
  method: ResizeMethod;
}

export interface GeometryValues {
  orientation: OrientationValues;
  resize: ResizeValues;
}

export const ANGLE_LIMITS = { min: -180, max: 180, step: 0.1 };
export const RESIZE_LIMITS = { percent: { min: 10, max: 400 }, side: { min: 1, max: 16_384 } };

export const ROTATION_METHODS: { id: RotationMethod; title: string }[] = [
  { id: "bilinear", title: "Bilinear" },
  { id: "bicubic", title: "Bicubic" },
];
export const RESIZE_METHODS: { id: ResizeMethod; title: string }[] = [
  { id: "lanczos3", title: "Lanczos3" },
  { id: "bicubic", title: "Bicubic" },
  { id: "bilinear", title: "Bilinear" },
  { id: "nearest", title: "Nearest Neighbor" },
];

export const defaultOrientation = (): OrientationValues => ({
  flipHorizontal: false,
  flipVertical: false,
  angle: 0,
  cropToFit: false,
  method: "bicubic",
});

export const defaultResize = (): ResizeValues => ({
  mode: "percent",
  percent: 100,
  width: 0,
  height: 0,
  lockAspect: true,
  driver: "width",
  method: "lanczos3",
});

export const defaultGeometry = (): GeometryValues => ({ orientation: defaultOrientation(), resize: defaultResize() });

/** Rounds half away from zero, like Swift's `rounded()` and Rust's `round()`. */
const round = (v: number): number => Math.sign(v) * Math.round(Math.abs(v));
const clamp = (v: number, low: number, high: number): number => Math.min(high, Math.max(low, v));

// MARK: Flip and Rotate

/** An angle in tenths of a degree, wrapped into -180 … 180 (180 stays 180). */
export function wrapAngle(degrees: number): number {
  let value = round(degrees * 10) / 10;
  while (value > 180) value -= 360;
  while (value <= -180) value += 360;
  return value === 0 ? 0 : Math.round(value * 10) / 10;
}

export function setAngle(o: OrientationValues, degrees: number): void {
  o.angle = wrapAngle(clamp(degrees, ANGLE_LIMITS.min, ANGLE_LIMITS.max));
}

/** Turns the picture on screen by `degrees` clockwise. */
export function rotate(o: OrientationValues, degrees: number): void {
  o.angle = wrapAngle(o.angle + degrees);
}

/** Mirrors what is on screen: the core mirrors first in the frame of the picture, so the angle changes its sign. */
export function flipHorizontally(o: OrientationValues): void {
  o.flipHorizontal = !o.flipHorizontal;
  o.angle = wrapAngle(-o.angle);
}

export function flipVertically(o: OrientationValues): void {
  o.flipVertical = !o.flipVertical;
  o.angle = wrapAngle(-o.angle);
}

export const isOrientationNeutral = (o: OrientationValues): boolean => !o.flipHorizontal && !o.flipVertical && Math.abs(o.angle) < 0.001;

/** What decides the size and place of the pixels: when it changes, a crop rectangle set on the old picture no longer fits. */
export const orientationLayout = (o: OrientationValues): number[] => [o.flipHorizontal ? 1 : 0, o.flipVertical ? 1 : 0, o.angle, o.cropToFit ? 1 : 0];

/** "turned 15.0° clockwise, flipped horizontally"; null while nothing changes. */
export function orientationSummary(o: OrientationValues): string | null {
  if (isOrientationNeutral(o)) return null;
  const parts: string[] = [];
  if (Math.abs(o.angle) >= 0.001) parts.push(`turned ${Math.abs(o.angle).toFixed(1)}° ${o.angle > 0 ? "clockwise" : "counterclockwise"}`);
  if (o.flipHorizontal) parts.push("flipped horizontally");
  if (o.flipVertical) parts.push("flipped vertically");
  return parts.join(", ");
}

/** Quarter turns clockwise (0..3) and the free rest in degrees (0 if there is none), as `split_angle` of the core. */
function splitAngle(angle: number): { turns: number; rest: number } {
  const a = Number.isFinite(angle) ? angle : 0;
  const quarters = round(a / 90);
  const rest = a - quarters * 90;
  return { turns: ((quarters % 4) + 4) % 4, rest: Math.abs(rest) < 1e-3 ? 0 : rest };
}

/** The canvas of a picture turned by a free angle, keeping the parity of the source (`free_size` of the core). */
function freeSize(width: number, height: number, degrees: number, cropToFit: boolean): Size {
  const radians = (degrees * Math.PI) / 180;
  const s = Math.abs(Math.sin(radians));
  const c = Math.abs(Math.cos(radians));
  let outW: number;
  let outH: number;
  if (cropToFit) {
    const a = Math.min(width / (width * c + height * s), height / (width * s + height * c), 1);
    outW = Math.max(Math.floor(width * a - 0.01), 1);
    outH = Math.max(Math.floor(height * a - 0.01), 1);
  } else {
    outW = Math.ceil(width * c + height * s - 1e-3);
    outH = Math.ceil(width * s + height * c - 1e-3);
  }
  const matchParity = (out: number, src: number) => (out % 2 === src % 2 ? out : cropToFit && out > 1 ? out - 1 : out + 1);
  return { width: matchParity(outW, width), height: matchParity(outH, height) };
}

/** The size of the flipped and turned picture for a source of `width` × `height`. */
export function orientedSize(width: number, height: number, o: OrientationValues): Size {
  const { turns, rest } = splitAngle(o.angle);
  const w = turns % 2 === 1 ? height : width;
  const h = turns % 2 === 1 ? width : height;
  return rest === 0 ? { width: w, height: h } : freeSize(w, h, rest, o.cropToFit);
}

// MARK: Resize Image

export type ResizeTarget = { percent: number } | { pixels: { width: number | null; height: number | null } };

/** The request for the core; with the lock only the side typed last is sent, so the other follows the shape. */
export function resizeTarget(r: ResizeValues): ResizeTarget {
  if (r.mode === "percent") return { percent: r.percent };
  const wide = r.width > 0 ? r.width : null;
  const high = r.height > 0 ? r.height : null;
  if (!r.lockAspect) return { pixels: { width: wide, height: high } };
  if ((r.driver === "width" && wide !== null) || (wide !== null && high === null)) return { pixels: { width: wide, height: null } };
  if ((r.driver === "height" && high !== null) || (wide === null && high !== null)) return { pixels: { width: null, height: high } };
  return { pixels: { width: null, height: null } };
}

/** The size the picture gets from a source of the given size (`Resize::output_size` of the core). */
export function resizeSize(source: Size, r: ResizeValues): Size {
  const side = (v: number) => clamp(round(v), 1, RESIZE_LIMITS.side.max);
  const fixed = (v: number) => clamp(v, 1, RESIZE_LIMITS.side.max);
  const target = resizeTarget(r);
  if ("percent" in target) {
    const share = clamp(target.percent, RESIZE_LIMITS.percent.min, RESIZE_LIMITS.percent.max) / 100;
    return { width: side(source.width * share), height: side(source.height * share) };
  }
  const { width, height } = target.pixels;
  if (width !== null && height !== null) return { width: fixed(width), height: fixed(height) };
  if (width !== null) return { width: fixed(width), height: side((source.height * fixed(width)) / Math.max(source.width, 1)) };
  if (height !== null) return { width: side((source.width * fixed(height)) / Math.max(source.height, 1)), height: fixed(height) };
  return { ...source };
}

export const isResizeNeutral = (source: Size, r: ResizeValues): boolean => {
  const size = resizeSize(source, r);
  return size.width === source.width && size.height === source.height;
};

const clampSide = (v: number) => clamp(Math.round(v), RESIZE_LIMITS.side.min, RESIZE_LIMITS.side.max);

/** Without the lock both sides are fixed numbers: the side not typed yet takes the size it has now. */
function pinSides(r: ResizeValues, source: Size): void {
  if (r.lockAspect) return;
  const now = resizeSize(source, r);
  if (r.width === 0) r.width = now.width;
  if (r.height === 0) r.height = now.height;
}

export function setResizeWidth(r: ResizeValues, value: number, source: Size): void {
  pinSides(r, source);
  r.width = clampSide(value);
  r.driver = "width";
}

export function setResizeHeight(r: ResizeValues, value: number, source: Size): void {
  pinSides(r, source);
  r.height = clampSide(value);
  r.driver = "height";
}

/** The pixel fields start from the size the picture has now. */
export function switchToPixels(r: ResizeValues, source: Size): void {
  const now = resizeSize(source, r);
  Object.assign(r, { mode: "pixels", width: now.width, height: now.height, driver: "width" });
}

/** Turning the lock off fixes both sides at the size they show now. */
export function setResizeLock(r: ResizeValues, on: boolean, source: Size): void {
  if (!on && r.mode === "pixels") {
    const now = resizeSize(source, r);
    r.width = now.width;
    r.height = now.height;
  }
  r.lockAspect = on;
}

// MARK: What Save Image sends

export interface GeometryPayload {
  orientation: OrientationValues;
  resize: { target: ResizeTarget; method: ResizeMethod } | null;
}

/** The geometry for the file Save Image writes, for a picture of `resizeSource` (after the crop); null while it changes nothing. */
export function geometryPayload(g: GeometryValues, resizeSource: Size): GeometryPayload | null {
  const resize = isResizeNeutral(resizeSource, g.resize) ? null : { target: resizeTarget(g.resize), method: g.resize.method };
  if (isOrientationNeutral(g.orientation) && resize === null) return null;
  return { orientation: { ...g.orientation }, resize };
}

/** "1200 × 900 px". */
export const sizeLabel = (size: Size): string => `${size.width} × ${size.height} px`;

// MARK: Project file

/** The section `geometry`, with the keys of the Mac app. */
export function geometrySection(g: GeometryValues): Record<string, unknown> {
  const { orientation: o, resize: r } = g;
  return {
    flipHorizontal: o.flipHorizontal,
    flipVertical: o.flipVertical,
    angle: o.angle,
    cropToFit: o.cropToFit,
    rotationMethod: o.method,
    resize: { mode: r.mode, percent: r.percent, width: r.width, height: r.height, lockAspect: r.lockAspect, driver: r.driver, method: r.method },
  };
}

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);
const bool = (v: unknown, keep: boolean) => (typeof v === "boolean" ? v : keep);
const num = (v: unknown, keep: number) => (typeof v === "number" && Number.isFinite(v) ? v : keep);
const choice = <T extends string>(v: unknown, options: readonly T[], keep: T): T => (options.includes(v as T) ? (v as T) : keep);

/** Reads the section back: a missing or wrong key keeps the value, a number outside the limits is cut to them. */
export function applyGeometrySection(section: unknown, g: GeometryValues): void {
  if (!isRecord(section)) return;
  const o = g.orientation;
  o.flipHorizontal = bool(section.flipHorizontal, o.flipHorizontal);
  o.flipVertical = bool(section.flipVertical, o.flipVertical);
  setAngle(o, num(section.angle, o.angle));
  o.cropToFit = bool(section.cropToFit, o.cropToFit);
  o.method = choice(section.rotationMethod, ["bilinear", "bicubic"] as const, o.method);
  const resize = section.resize;
  if (!isRecord(resize)) return;
  const r = g.resize;
  r.mode = choice(resize.mode, ["percent", "pixels"] as const, r.mode);
  r.percent = clamp(Math.round(num(resize.percent, r.percent)), RESIZE_LIMITS.percent.min, RESIZE_LIMITS.percent.max);
  r.width = clamp(Math.round(num(resize.width, r.width)), 0, RESIZE_LIMITS.side.max);
  r.height = clamp(Math.round(num(resize.height, r.height)), 0, RESIZE_LIMITS.side.max);
  r.lockAspect = bool(resize.lockAspect, r.lockAspect);
  r.driver = choice(resize.driver, ["width", "height"] as const, r.driver);
  r.method = choice(resize.method, ["lanczos3", "bicubic", "bilinear", "nearest"] as const, r.method);
}
