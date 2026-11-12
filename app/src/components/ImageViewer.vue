<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { isRegion, rectBetween, type Rect } from "../logic/align";
import { actualSize, dragView, fitView, formatZoom, imagePixel, imagePosition, zoomAt, type Size, type View } from "../logic/viewport";

// The image area: mouse wheel zooms around the pointer, dragging moves the image, the zoom bar at the bottom
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
}>();
const before = defineModel<boolean>("before", { default: false });
const emit = defineEmits<{
  hover: [pixel: { x: number; y: number } | null];
  /** A finished rectangle in image pixels; a drag too small to be a region is dropped. */
  select: [rect: Rect];
}>();

const box = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
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

function draw() {
  const image = props.image;
  const target = canvas.value;
  if (!image || !target) return;
  target.width = image.width;
  target.height = image.height;
  target.getContext("2d")?.putImageData(image, 0, 0);
}

// A new image size (another recording, a resampled stack) starts fitted; frames of the same size keep the view.
watch(size, (now, was) => {
  if (now && (!was || now.w !== was.w || now.h !== was.h)) fit();
});
watch(() => props.image, draw, { flush: "post" });

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
  draw();
});
onBeforeUnmount(() => observer?.disconnect());

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
  zoom(event.deltaY < 0 ? 1.15 : 1 / 1.15, point(event));
}

let drag: { x: number; y: number } | null = null;
const dragging = ref(false);
// The rectangle that is being drawn: from the corner where the drag started to the pointer.
let corner: { x: number; y: number } | null = null;
const draft = ref<Rect | null>(null);

function onDown(event: PointerEvent) {
  if (!size.value || (event.target as HTMLElement).closest(".floats > *, .zoom, .topbar button, .topbar .menu")) return;
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
      <svg class="overlay" :viewBox="`0 0 ${size.w} ${size.h}`" aria-hidden="true">
        <rect x="0.5" y="0.5" :width="size.w - 1" :height="size.h - 1" class="edge" />
        <slot name="overlay" />
        <rect v-if="draft" :x="draft.x" :y="draft.y" :width="draft.width" :height="draft.height" class="draft" stroke-dasharray="6 5" />
      </svg>
    </div>
    <p v-else class="empty">{{ empty }}</p>

    <slot name="top" />
    <div class="floats"><slot name="floats" /></div>

    <div v-if="size" class="zoom">
      <button type="button" aria-label="Zoom out" @click="zoom(1 / 1.25)">−</button>
      <button type="button" title="Show at 100 %" @click="showActualSize">{{ formatZoom(view.scale) }}</button>
      <button type="button" aria-label="Zoom in" @click="zoom(1.25)">+</button>
      <span class="d" />
      <button type="button" @click="fit">Fit</button>
      <template v-if="comparable">
        <span class="d" />
        <button type="button" :aria-pressed="before" title="Show the unprocessed stack" @click="before = !before">Before</button>
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
