<script setup lang="ts">
import { computed } from "vue";
import { processingArea } from "../logic/viewport";
import { atLeast } from "../logic/workflow";
import { session, settings } from "../state";

// Marks drawn over the image, in image pixels: the alignpoints (red dots after Set Alignpoints, yellow rings with
// a green shift stroke after Limit) and the Processing Area of the Wavelet tab.
const props = defineProps<{ width: number; height: number }>();

const dots = computed(() => session.tab === "align" && settings.align.showAlignpoints && atLeast(session.step, "points"));
const rings = computed(
  () =>
    atLeast(session.step, "limited") &&
    ((session.tab === "stack" && settings.stack.showAlignSections) || (session.tab === "wavelet" && settings.wavelet.showAlignpoints)),
);
const region = computed(() => (session.tab === "align" && settings.align.showRoi ? session.roi : null));
const area = computed(() =>
  session.tab === "wavelet" && settings.wavelet.showProcessingArea ? processingArea({ w: props.width, h: props.height }, settings.menu.processingArea) : null,
);
// The shift is only a few pixels, so the stroke shows it enlarged to stay visible.
const stroke = (shift: number | undefined) => (shift ?? 0) * 4;
</script>

<template>
  <template v-if="dots">
    <circle v-for="(p, i) in session.alignpoints" :key="i" :cx="p.x" :cy="p.y" r="3.2" fill="var(--ap-point)" />
  </template>
  <template v-if="rings">
    <template v-for="(p, i) in session.alignpoints" :key="i">
      <circle :cx="p.x" :cy="p.y" r="7" fill="none" stroke="var(--ap-ring)" stroke-width="1.4" />
      <line :x1="p.x" :y1="p.y" :x2="p.x + stroke(p.shiftX)" :y2="p.y + stroke(p.shiftY)" stroke="var(--ap-shift)" stroke-width="1.4" />
    </template>
  </template>
  <rect
    v-if="region"
    :x="region.x"
    :y="region.y"
    :width="region.width"
    :height="region.height"
    fill="none"
    stroke="var(--accent)"
    stroke-width="1.5"
    stroke-dasharray="6 5"
    vector-effect="non-scaling-stroke"
  />
  <rect
    v-if="area"
    :x="area.x"
    :y="area.y"
    :width="area.w"
    :height="area.h"
    fill="none"
    stroke="var(--accent)"
    stroke-width="1.5"
    stroke-dasharray="6 5"
    vector-effect="non-scaling-stroke"
  />
</template>
