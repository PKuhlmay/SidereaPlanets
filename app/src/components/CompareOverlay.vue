<script setup lang="ts">
import { compareLabelFits } from "../cropTool";

// The divider and the labels of View Compare over the image area (RK-021, docs/design.md "Bildbereich"); the two images are drawn by
// ImageViewer. Only the strip around the line takes the pointer, the rest of the area keeps moving the image. The arrow keys move the
// divider too.
const props = defineProps<{ share: number; width: number }>();
const emit = defineEmits<{ move: [share: number] }>();

/** What a label needs between the divider and the edge: its own width and the margin. */
const LABEL_ROOM = 84;

let dragging = false;

function down(event: PointerEvent) {
  dragging = true;
  (event.currentTarget as Element).setPointerCapture(event.pointerId);
}

function move(event: PointerEvent) {
  if (!dragging || props.width <= 0) return;
  const area = (event.currentTarget as HTMLElement).parentElement?.getBoundingClientRect();
  if (area) emit("move", (event.clientX - area.left) / props.width);
}

function key(event: KeyboardEvent) {
  if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
  event.preventDefault();
  event.stopPropagation();
  emit("move", props.share + (event.key === "ArrowRight" ? 0.05 : -0.05));
}
</script>

<template>
  <div class="compare">
    <span v-if="compareLabelFits(share, width, LABEL_ROOM, 'before')" class="side before">Before</span>
    <span v-if="compareLabelFits(share, width, LABEL_ROOM, 'after')" class="side after">After</span>
    <div
      class="divider"
      :style="{ left: `${share * 100}%` }"
      role="slider"
      tabindex="0"
      aria-label="Compare divider"
      :aria-valuenow="Math.round(share * 100)"
      aria-valuemin="2"
      aria-valuemax="98"
      title="Drag to compare the stack with the result"
      @pointerdown.stop="down"
      @pointermove="move"
      @pointerup="dragging = false"
      @pointercancel="dragging = false"
      @keydown="key"
    >
      <span class="line" />
      <span class="grip" aria-hidden="true">‹ ›</span>
    </div>
  </div>
</template>

<style scoped>
.compare {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.side {
  position: absolute;
  bottom: 12px;
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  font-size: var(--fs-s);
  font-weight: 500;
  color: var(--on-canvas);
  background: var(--canvas-chrome);
  border: 1px solid var(--canvas-line);
  border-radius: var(--r-l);
}
.before {
  left: 12px;
}
.after {
  right: 12px;
}
.divider {
  position: absolute;
  top: 0;
  bottom: 0;
  display: grid;
  place-items: center;
  width: 30px;
  cursor: ew-resize;
  pointer-events: auto;
  transform: translateX(-50%);
}
.divider:focus-visible {
  outline: none;
}
.divider:focus-visible .grip {
  box-shadow: 0 0 0 2px var(--accent);
}
.line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background: color-mix(in srgb, var(--on-canvas) 90%, transparent);
  box-shadow: 0 0 2px rgba(0, 0, 0, 0.5);
}
/* the image area stays dark in both themes, so the grip uses the colors of the other canvas controls */
.grip {
  position: relative;
  display: grid;
  place-items: center;
  width: 22px;
  height: 28px;
  font-size: 11px;
  font-weight: 700;
  color: var(--accent);
  background: rgba(0, 0, 0, 0.75);
  border: 1px solid var(--accent);
  border-radius: 14px;
}
</style>
