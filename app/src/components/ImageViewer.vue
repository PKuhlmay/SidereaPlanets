<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { isRegion, rectBetween, type Rect } from "../logic/align";
import { changes, drawRegion, type Drawn } from "../logic/dirty";
import { dragged, drawn, handleAt, isFull, type CropAspect, type CropHandle, type CropRectangle } from "../logic/crop";
import { isMac } from "../logic/platform";
import { actualSize, dragView, fitView, formatZoom, imagePixel, imagePosition, wheelIntent, zoomAt, type Size, type View } from "../logic/viewport";
import { recordDraw } from "../previewTiming";
import { viewerControl } from "../viewerControl";
import CompareOverlay from "./CompareOverlay.vue";
import CropOverlay from "./CropOverlay.vue";

// The image area: mouse wheel zooms around the pointer (on the Mac scrolling moves the image, Option or Cmd and scrolling or the
// pinch of the trackpad zoom), dragging moves the image, the zoom bar at the bottom
// gives -, the scale (a click shows 100 %), + and Fit. Overlays in the `overlay` slot are drawn in image pixels,
// so they zoom with the image.
const props = defineProps<{
  image: ImageData | null;
  /** Shows the Before switch (Wavelets and Save steps). */
  comparable?: boolean;
  /** Text for the empty area. */
  empty: string;
  /** The next drag draws a rectangle (R.o.Interest) instead of moving the image. */
  selecting?: boolean;
  /** Another value (the step) fits the image into the area again, whatever zoom and position it had. */
  fitKey?: string;
  /** View Compare: the stack, shown left of the divider at the same zoom and position. */
  beforeImage?: ImageData | null;
  /** Where the divider sits, as a share of the width of the area. */
  divider?: number;
  /** The crop rectangle in image pixels: the one being edited while `cropEditing`, else the one Done kept. */
  crop?: CropRectangle | null;
  /** The crop tool is open: a drag moves, resizes or draws the rectangle instead of moving the image. */
  cropEditing?: boolean;
  /** The shape the rectangle keeps while it is dragged; null is Free. */
  cropAspect?: CropAspect | null;
}>();
const before = defineModel<boolean>("before", { default: false });
const emit = defineEmits<{
  hover: [pixel: { x: number; y: number } | null];
  /** A finished rectangle in image pixels; a drag too small to be a region is dropped. */
  select: [rect: Rect];
  /** The divider of View Compare was dragged to a share of the width. */
  divider: [share: number];
  /** The crop rectangle was dragged, resized or drawn (image pixels). */
  crop: [rect: CropRectangle];
}>();

const box = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const beforeCanvas = ref<HTMLCanvasElement | null>(null);
const boxSize = ref<Size>({ w: 0, h: 0 });
const view = ref<View>({ scale: 1, x: 0, y: 0 });
// While true the image follows the size of the area; any zoom or drag by the user turns it off until Fit.
const fitted = ref(true);

const size = computed<Size | null>(() => (props.image ? { w: props.image.width, h: props.image.height } : null));

function fit() {
  if (!size.value || boxSize.value.w === 0) return;
  view.value = fitView(boxSize.value, size.value);
  fitted.value = true;
}

// What the canvas shows now. A preview writes into the pixels of the image already on it and says which rectangle it changed
// (logic/dirty.ts); then only that rectangle is put on the canvas, and the canvas is not cleared and sized again.
let onCanvas: Drawn | null = null;

function draw() {
  const image = props.image;
  const target = canvas.value;
  if (!image || !target) return;
  const started = performance.now();
  const now: Drawn = { surface: target, buffer: image.data.buffer, width: image.width, height: image.height };
  const part = drawRegion(onCanvas, now, changes.take(now.buffer));
  if (!onCanvas || onCanvas.surface !== target || onCanvas.width !== image.width || onCanvas.height !== image.height) {
    target.width = image.width;
    target.height = image.height;
  }
  const context = target.getContext("2d");
  if (part) context?.putImageData(image, 0, 0, part.x, part.y, part.width, part.height);
  else context?.putImageData(image, 0, 0);
  onCanvas = now;
  recordDraw(performance.now() - started);
}

// A new image size (another recording, a resampled stack) starts fitted; frames of the same size keep the view.
watch(size, (now, was) => {
  if (now && (!was || now.w !== was.w || now.h !== was.h)) fit();
});
watch(() => props.image, draw, { flush: "post" });

function drawBefore() {
  const image = props.beforeImage;
  const target = beforeCanvas.value;
  if (!image || !target) return;
  target.width = image.width;
  target.height = image.height;
  target.getContext("2d")?.putImageData(image, 0, 0);
}
watch(() => props.beforeImage, drawBefore, { flush: "post" });

// The stack shows left of the divider: its canvas is cut at the divider, in image pixels (the stage is scaled).
const beforeClip = computed(() => {
  if (!size.value) return "none";
  const imageX = ((props.divider ?? 0.5) * boxSize.value.w - view.value.x) / view.value.scale;
  return `inset(0 ${Math.max(0, size.value.w - imageX)}px 0 0)`;
});

/** How far from an edge or corner of the crop rectangle the pointer still takes hold of it, in screen pixels. */
const GRAB_DISTANCE = 7;

// The crop rectangle that can be grabbed: none while it is the whole image, so the next drag draws a new one.
const editableCrop = computed(() => {
  const crop = props.crop;
  return crop && size.value && !isFull(crop, size.value.w, size.value.h) ? crop : null;
});
const cropCursor = ref("crosshair");
const CURSORS: Record<CropHandle, string> = {
  topLeft: "nwse-resize",
  bottomRight: "nwse-resize",
  topRight: "nesw-resize",
  bottomLeft: "nesw-resize",
  top: "ns-resize",
  bottom: "ns-resize",
  left: "ew-resize",
  right: "ew-resize",
  body: "move",
};
// The crop drag in progress: what was grabbed (null = a new rectangle), where, and the rectangle then.
let cropDrag: { handle: CropHandle | null; start: { x: number; y: number }; rect: CropRectangle | null } | null = null;

function cropHandleAt(px: number, py: number): CropHandle | null {
  const rect = editableCrop.value;
  if (!rect) return null;
  const at = imagePosition(view.value, px, py);
  return handleAt(rect, at.x, at.y, GRAB_DISTANCE / Math.max(view.value.scale, 0.01));
}

// Another step brings another layout around the image (frame bar, right panel). Measure the area after that layout
// is in place and fit again, so that the image neither keeps the zoom of the other layout nor sits off-centre.
watch(
  () => props.fitKey,
  async () => {
    fitted.value = true;
    await nextTick();
    const rect = box.value?.getBoundingClientRect();
    if (rect) boxSize.value = { w: rect.width, h: rect.height };
    fit();
  },
);

let observer: ResizeObserver | undefined;
onMounted(() => {
  observer = new ResizeObserver(([entry]) => {
    boxSize.value = { w: entry.contentRect.width, h: entry.contentRect.height };
    if (fitted.value) fit();
  });
  if (box.value) observer.observe(box.value);
  for (const name of pinchEvents) box.value?.addEventListener(name, onPinch);
  viewerControl.zoom = zoom;
  viewerControl.fit = fit;
  viewerControl.actualSize = showActualSize;
  draw();
});
onBeforeUnmount(() => {
  observer?.disconnect();
  for (const name of pinchEvents) box.value?.removeEventListener(name, onPinch);
  viewerControl.zoom = viewerControl.fit = viewerControl.actualSize = null;
});

const stageStyle = computed(() => ({
  width: `${size.value?.w ?? 0}px`,
  height: `${size.value?.h ?? 0}px`,
  transform: `translate(${view.value.x}px, ${view.value.y}px) scale(${view.value.scale})`,
  imageRendering: view.value.scale >= 2 ? ("pixelated" as const) : ("auto" as const),
}));

function point(event: { clientX: number; clientY: number }) {
  const rect = box.value?.getBoundingClientRect();
  return { px: event.clientX - (rect?.left ?? 0), py: event.clientY - (rect?.top ?? 0) };
}

function zoom(factor: number, at?: { px: number; py: number }) {
  if (!size.value) return;
  const { px, py } = at ?? { px: boxSize.value.w / 2, py: boxSize.value.h / 2 };
  view.value = zoomAt(view.value, factor, px, py);
  fitted.value = false;
}

function showActualSize() {
  view.value = actualSize(view.value, boxSize.value);
  fitted.value = false;
}

function onWheel(event: WheelEvent) {
  if (!size.value) return;
  const intent = wheelIntent(event, isMac);
  if (intent.kind === "zoom") {
    zoom(intent.factor, point(event));
  } else {
    view.value = { ...view.value, x: view.value.x + intent.dx, y: view.value.y + intent.dy };
    fitted.value = false;
  }
}

// The pinch of the trackpad reaches the web view of the Mac as gesture events (Safari's own, not in lib.dom): `scale` is the
// factor since the start of the pinch, so every change zooms by the step since the one before, around the fingers.
interface PinchEvent extends Event {
  scale: number;
  clientX: number;
  clientY: number;
}
let pinchScale = 1;
function onPinch(event: Event) {
  event.preventDefault();
  const pinch = event as PinchEvent;
  if (event.type === "gesturestart") pinchScale = 1;
  else {
    zoom(pinch.scale / pinchScale, point(pinch));
    pinchScale = pinch.scale;
  }
}
const pinchEvents = ["gesturestart", "gesturechange", "gestureend"];

let drag: { x: number; y: number } | null = null;
const dragging = ref(false);
// The rectangle that is being drawn: from the corner where the drag started to the pointer.
let corner: { x: number; y: number } | null = null;
const draft = ref<Rect | null>(null);

function onDown(event: PointerEvent) {
  if (!size.value || (event.target as HTMLElement).closest(".floats > *, .zoom, .topbar button, .topbar .menu")) return;
  if (props.cropEditing) {
    const { px, py } = point(event);
    cropDrag = { handle: cropHandleAt(px, py), start: imagePosition(view.value, px, py), rect: editableCrop.value };
    box.value?.setPointerCapture(event.pointerId);
    return;
  }
  if (props.selecting) {
    const { px, py } = point(event);
    corner = imagePosition(view.value, px, py);
    draft.value = rectBetween(corner, corner, size.value);
    box.value?.setPointerCapture(event.pointerId);
    return;
  }
  drag = { x: event.clientX - view.value.x, y: event.clientY - view.value.y };
  dragging.value = true;
  box.value?.setPointerCapture(event.pointerId);
}

function onMove(event: PointerEvent) {
  if (props.cropEditing && size.value) {
    const { px, py } = point(event);
    const at = imagePosition(view.value, px, py);
    if (cropDrag) {
      const { handle, start, rect } = cropDrag;
      const next =
        handle && rect
          ? dragged(rect, handle, at.x - start.x, at.y - start.y, size.value.w, size.value.h, props.cropAspect)
          : drawn(start.x, start.y, at.x, at.y, size.value.w, size.value.h, props.cropAspect);
      if (next) emit("crop", next);
    } else {
      const handle = cropHandleAt(px, py);
      cropCursor.value = handle ? CURSORS[handle] : "crosshair";
    }
  }
  if (corner && size.value) {
    const { px, py } = point(event);
    draft.value = rectBetween(corner, imagePosition(view.value, px, py), size.value);
  }
  if (drag) {
    // A click without moving must not count as moving the image, or it would stop following the area.
    const moved = dragView(view.value, drag, { x: event.clientX, y: event.clientY });
    if (moved) {
      view.value = moved;
      fitted.value = false;
    }
  }
  if (!size.value) return;
  const { px, py } = point(event);
  emit("hover", imagePixel(view.value, size.value, px, py));
}

function endDrag() {
  cropDrag = null;
  if (draft.value && isRegion(draft.value)) emit("select", draft.value);
  corner = null;
  draft.value = null;
  drag = null;
  dragging.value = false;
}
</script>

<template>
  <section
    ref="box"
    class="viewer"
    :class="{ drag: dragging, selecting }"
    :style="cropEditing ? { cursor: cropCursor } : undefined"
    aria-label="Image"
    @wheel.prevent="onWheel"
    @pointerdown="onDown"
    @pointermove="onMove"
    @pointerup="endDrag"
    @pointercancel="endDrag"
    @pointerleave="emit('hover', null)"
  >
    <div v-if="size" class="stage" :style="stageStyle">
      <canvas ref="canvas" />
      <canvas v-if="beforeImage" ref="beforeCanvas" :style="{ clipPath: beforeClip }" />
      <svg class="overlay" :viewBox="`0 0 ${size.w} ${size.h}`" aria-hidden="true">
        <rect x="0.5" y="0.5" :width="size.w - 1" :height="size.h - 1" class="edge" />
        <slot name="overlay" />
        <rect v-if="draft" :x="draft.x" :y="draft.y" :width="draft.width" :height="draft.height" class="draft" stroke-dasharray="6 5" />
      </svg>
    </div>
    <p v-else class="empty">{{ empty }}</p>

    <CropOverlay v-if="size && crop" :rectangle="crop" :view="view" :editing="!!cropEditing" :width="boxSize.w" :height="boxSize.h" />
    <CompareOverlay v-if="size && beforeImage" :share="divider ?? 0.5" :width="boxSize.w" @move="emit('divider', $event)" />

    <slot name="top" />
    <div class="floats"><slot name="floats" /></div>

    <div v-if="size" class="zoom">
      <button type="button" aria-label="Zoom out" @click="zoom(1 / 1.25)">−</button>
      <button type="button" title="Show at 100%" @click="showActualSize">{{ formatZoom(view.scale) }}</button>
      <button type="button" aria-label="Zoom in" @click="zoom(1.25)">+</button>
      <span class="d" />
      <button type="button" title="Fit the image into the area" @click="fit">Fit</button>
      <template v-if="comparable">
        <span class="d" />
        <button type="button" :aria-pressed="before" title="Switch between the stack without any sharpening and the sharpened image" @click="before = !before">
          Before/After
        </button>
      </template>
    </div>
  </section>
</template>

<style scoped>
.viewer {
  position: relative;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  cursor: grab;
  touch-action: none;
  background: var(--canvas);
}
.viewer.drag {
  cursor: grabbing;
}
.viewer.selecting {
  cursor: crosshair;
}
.draft {
  fill: none;
  stroke: var(--accent);
  stroke-width: 1.5;
  vector-effect: non-scaling-stroke;
}
.stage {
  position: absolute;
  top: 0;
  left: 0;
  transform-origin: 0 0;
}
canvas,
.overlay {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
}
.overlay {
  pointer-events: none;
}
.edge {
  fill: none;
  stroke: rgba(255, 255, 255, 0.07);
  vector-effect: non-scaling-stroke;
}
.empty {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  margin: 0;
  color: var(--on-canvas);
  opacity: 0.6;
}
/* the cards start below the row with the caption and Show (slot "top") */
.floats {
  position: absolute;
  top: 48px;
  right: 12px;
  bottom: 56px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 236px;
  overflow-y: auto;
  pointer-events: none;
}
.zoom {
  position: absolute;
  bottom: 12px;
  left: 50%;
  display: flex;
  gap: 2px;
  align-items: center;
  padding: 3px;
  font-variant-numeric: tabular-nums;
  color: var(--on-canvas);
  cursor: default;
  background: var(--canvas-chrome);
  border: 1px solid var(--canvas-line);
  border-radius: var(--r-l);
  backdrop-filter: blur(10px);
  transform: translateX(-50%);
}
.zoom button {
  min-width: 26px;
  height: 24px;
  padding: 0 8px;
  color: inherit;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: 5px;
}
.zoom button:hover {
  background: rgba(255, 255, 255, 0.1);
}
.zoom button[aria-pressed="true"] {
  font-weight: 600;
  color: var(--on-accent);
  background: var(--accent);
}
.d {
  width: 1px;
  height: 16px;
  margin: 0 3px;
  background: var(--canvas-line);
}
</style>
