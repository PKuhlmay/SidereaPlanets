<script setup lang="ts">
import { computed } from "vue";
import { discOverlay } from "../logic/derotation";
import { orientationApplies } from "../geometryView";
import { processingArea } from "../logic/viewport";
import { atLeast, imageKind } from "../logic/workflow";
import { session, settings } from "../state";

// Marks drawn over the image, in image pixels: the alignpoints (red dots after Set Alignpoints, yellow rings with
// a green shift stroke after Limit) and the Processing Area of the Wavelets and Save steps.
const props = defineProps<{ width: number; height: number }>();

const kind = computed(() => imageKind(session.view));
const dots = computed(() => kind.value === "frame" && settings.align.showAlignpoints && atLeast(session.step, "points"));
// The marks of the stack are not drawn on a flipped or turned picture: they would sit in the wrong place.
const turned = computed(() => kind.value === "wavelet" && orientationApplies());
const rings = computed(
  () =>
    !turned.value &&
    atLeast(session.step, "limited") &&
    ((kind.value === "stack" && settings.stack.showAlignSections) || (kind.value === "wavelet" && settings.wavelet.showAlignpoints)),
);
const region = computed(() => (kind.value === "frame" && settings.align.showRoi ? session.roi : null));
const area = computed(() =>
  kind.value === "wavelet" && !turned.value && settings.wavelet.showProcessingArea ? processingArea({ w: props.width, h: props.height }, settings.menu.processingArea) : null,
);
// De-rotate images: the disc measured in the reference frame, Saturn's ring edge and N over the north pole.
const disc = computed(() =>
  kind.value === "stack" && settings.stack.derotate && session.derotation?.disc ? discOverlay(session.derotation.disc) : null,
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
  <g v-if="disc" class="disc">
    <ellipse v-bind="disc.globe" fill="none" stroke="var(--accent)" stroke-width="1.5" vector-effect="non-scaling-stroke" />
    <ellipse
      v-if="disc.ring"
      v-bind="disc.ring"
      fill="none"
      stroke="var(--accent)"
      stroke-opacity="0.7"
      stroke-width="1"
      stroke-dasharray="5 4"
      vector-effect="non-scaling-stroke"
    />
    <line v-bind="disc.axis" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round" vector-effect="non-scaling-stroke" />
    <text :x="disc.label.x" :y="disc.label.y" fill="var(--accent)" font-size="12" font-weight="600" text-anchor="middle" dominant-baseline="central">N</text>
  </g>
</template>
