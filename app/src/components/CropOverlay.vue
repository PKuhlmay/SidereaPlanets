<script setup lang="ts">
import { computed } from "vue";
import { centreCrossArm, thirdsLines, type CropRectangle } from "../logic/crop";
import type { View } from "../logic/viewport";

// The crop rectangle over the image (SP-021, after the earlier SwiftUI app and Siderea): everything outside is dimmed, a light border
// over a dark one, a rule-of-thirds grid, a small cross at the centre, eight square grips and a "width × height" plate. The rectangle Done
// kept shows only the dimmed outside, the border and the plate. Drawn in screen pixels from the view, so every line stays thin; never in
// the way of the pointer (ImageViewer drags it). The image area is dark in both themes, so the colors are fixed white over black.
const props = defineProps<{ rectangle: CropRectangle; view: View; editing: boolean; width: number; height: number }>();

const CROSS_ARM = 10;
const GRIP = 8;

const frame = computed(() => {
  const { scale, x, y } = props.view;
  const r = props.rectangle;
  return { x: r.minX * scale + x, y: r.minY * scale + y, w: (r.maxX - r.minX) * scale, h: (r.maxY - r.minY) * scale };
});

/** The whole area with the rectangle as a hole (even-odd). */
const dim = computed(() => {
  const f = frame.value;
  return `M0 0H${props.width}V${props.height}H0Z M${f.x} ${f.y}H${f.x + f.w}V${f.y + f.h}H${f.x}Z`;
});

const grid = computed(() => {
  const { scale, x, y } = props.view;
  const f = frame.value;
  const lines = thirdsLines(props.rectangle);
  const vertical = lines.xs.map((ix) => `M${ix * scale + x} ${f.y}V${f.y + f.h}`);
  const horizontal = lines.ys.map((iy) => `M${f.x} ${iy * scale + y}H${f.x + f.w}`);
  return [...vertical, ...horizontal].join(" ");
});

const cross = computed(() => {
  const f = frame.value;
  const arm = centreCrossArm(props.rectangle, CROSS_ARM / props.view.scale) * props.view.scale;
  const cx = f.x + f.w / 2;
  const cy = f.y + f.h / 2;
  return `M${cx - arm} ${cy}H${cx + arm} M${cx} ${cy - arm}V${cy + arm}`;
});

const grips = computed(() => {
  const f = frame.value;
  const xs = [f.x, f.x + f.w / 2, f.x + f.w];
  const ys = [f.y, f.y + f.h / 2, f.y + f.h];
  return ys.flatMap((gy, row) => xs.filter((_, col) => !(row === 1 && col === 1)).map((gx) => ({ x: gx - GRIP / 2, y: gy - GRIP / 2 })));
});

const label = computed(() => `${props.rectangle.maxX - props.rectangle.minX} × ${props.rectangle.maxY - props.rectangle.minY}`);
</script>

<template>
  <svg class="crop" :viewBox="`0 0 ${width} ${height}`" :width="width" :height="height" aria-hidden="true">
    <path :d="dim" fill-rule="evenodd" :fill-opacity="editing ? 0.55 : 0.4" fill="#000" />
    <rect :x="frame.x" :y="frame.y" :width="frame.w" :height="frame.h" class="under" />
    <rect :x="frame.x" :y="frame.y" :width="frame.w" :height="frame.h" class="border" />
    <template v-if="editing">
      <path :d="grid" class="grid-under" />
      <path :d="grid" class="grid" />
      <path :d="cross" class="under square" />
      <path :d="cross" class="border square" />
      <rect v-for="(g, i) in grips" :key="i" :x="g.x" :y="g.y" :width="GRIP" :height="GRIP" class="grip" />
    </template>
    <g :transform="`translate(${frame.x + 10} ${frame.y + 10})`">
      <rect :width="label.length * 6.6 + 8" height="17" rx="3" fill="#000" fill-opacity="0.6" />
      <text x="4" y="12.5" class="plate">{{ label }}</text>
    </g>
  </svg>
</template>

<style scoped>
.crop {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}
.under {
  fill: none;
  stroke: #000;
  stroke-opacity: 0.6;
  stroke-width: 3;
}
.border {
  fill: none;
  stroke: #fff;
  stroke-width: 1;
}
.square {
  stroke-linecap: square;
}
.grid-under {
  fill: none;
  stroke: #000;
  stroke-opacity: 0.35;
  stroke-width: 2;
}
.grid {
  fill: none;
  stroke: #fff;
  stroke-opacity: 0.55;
  stroke-width: 0.5;
}
.grip {
  fill: #fff;
  stroke: #000;
  stroke-opacity: 0.7;
  stroke-width: 1;
}
.plate {
  font-size: 11px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  fill: #fff;
}
</style>
