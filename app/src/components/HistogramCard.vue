<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { autoLevels, closeFunction, requestHistogram } from "../colour";
import { failureText } from "../jobs";
import {
  colourPayload,
  histogramKey,
  histogramMax,
  histogramPath,
  levelToX,
  moveBlack,
  moveWhite,
  xToLevel,
  type HistogramData,
} from "../logic/colour";
import { TONE_LIMITS } from "../logic/settings";
import { session, settings, status } from "../state";
import FloatCard from "./FloatCard.vue";
import NumberField from "./NumberField.vue";

// Histogram card: the histogram of the image and the black and white point of the stretch (markers below the graph).
const tone = settings.wavelet.tone;
const W = 212;
const H = 84;
const MARKER = 12;

const data = ref<HistogramData | null>(null);
const max = computed(() => (data.value ? histogramMax(data.value) : 0));
const area = computed(() => (data.value ? histogramPath(data.value.luma, max.value, W, H, true) : ""));
const lines = computed(() => {
  const channels = data.value?.channels ?? [];
  const colours = ["var(--ch-r)", "var(--ch-g)", "var(--ch-b)"];
  // A grayscale image has one channel; it is drawn like the luminance, in the text colour.
  return channels.map((counts, i) => ({
    colour: channels.length === 3 ? colours[i] : "var(--muted)",
    path: histogramPath(counts, max.value, W, H),
  }));
});

// One request at a time, the newest answer wins; moving a field or marker waits a moment before asking.
let latest = 0;
let timer: number | undefined;
function load() {
  window.clearTimeout(timer);
  timer = window.setTimeout(async () => {
    const request = ++latest;
    try {
      const result = await requestHistogram(tone.histogramView === "result");
      if (request === latest) data.value = result;
    } catch (e) {
      if (request === latest) {
        data.value = null;
        status.result = failureText("Histogram", e);
      }
    }
  }, 80);
}
onBeforeUnmount(() => window.clearTimeout(timer));

watch(
  () => [histogramKey(colourPayload(settings.wavelet, settings.refine), tone.histogramView === "result"), session.processedImage, session.stackImage],
  load,
  { immediate: true },
);

// Dragging a marker: the pointer position on the graph gives the level.
const svg = ref<SVGSVGElement | null>(null);
const dragged = ref<"black" | "white" | null>(null);

function start(which: "black" | "white", event: PointerEvent) {
  dragged.value = which;
  (event.currentTarget as Element).setPointerCapture(event.pointerId);
}

function move(event: PointerEvent) {
  if (!dragged.value || !svg.value) return;
  const rect = svg.value.getBoundingClientRect();
  const level = xToLevel(((event.clientX - rect.left) / rect.width) * W, W);
  if (dragged.value === "black") tone.black = moveBlack(level, tone.white);
  else tone.white = moveWhite(level, tone.black);
}

const triangle = (level: number) => {
  const x = levelToX(level, W);
  return `${x - 6},${H + MARKER} ${x + 6},${H + MARKER} ${x},${H + 2}`;
};

function reset() {
  tone.black = TONE_LIMITS.level.min;
  tone.white = TONE_LIMITS.level.max;
}
</script>

<template>
  <FloatCard class="fixed" title="Histogram" @close="closeFunction('Histogram')">
    <template #actions>
      <button type="button" class="link" @click="reset">Reset</button>
    </template>
    <div class="cardbody">
      <svg
        ref="svg"
        :viewBox="`0 0 ${W} ${H + MARKER}`"
        width="100%"
        role="img"
        aria-label="Histogram"
        title="Drag the triangles to set the black and white point"
        @pointermove="move"
        @pointerup="dragged = null"
        @pointercancel="dragged = null"
      >
        <rect x="0" y="0" :width="W" :height="H" class="plot" />
        <path :d="area" class="luma" />
        <path v-for="(line, i) in lines" :key="i" :d="line.path" fill="none" :stroke="line.colour" stroke-width="1" />
        <polygon :points="triangle(tone.black)" class="marker" @pointerdown="start('black', $event)">
          <title>Black point</title>
        </polygon>
        <polygon :points="triangle(tone.white)" class="marker" @pointerdown="start('white', $event)">
          <title>White point</title>
        </polygon>
      </svg>
      <div class="seg">
        <button type="button" :aria-pressed="tone.histogramView === 'input'" @click="tone.histogramView = 'input'">Input</button>
        <button type="button" :aria-pressed="tone.histogramView === 'result'" @click="tone.histogramView = 'result'">Result</button>
      </div>
      <div class="grid2">
        <label class="col sm">Black point <NumberField v-model="tone.black" :min="0" :max="tone.white - 1" label="Black point" /></label>
        <label class="col sm">White point <NumberField v-model="tone.white" :min="tone.black + 1" :max="255" label="White point" /></label>
      </div>
      <div class="row">
        <button type="button" class="btn" @click="autoLevels">Auto</button>
        <label class="row clip" title="Share of the darkest and brightest pixels that Auto may cut off.">Clip (%) <NumberField v-model="tone.clip" :decimals="1" :step="0.1" :min="TONE_LIMITS.clip.min" :max="TONE_LIMITS.clip.max" :width="56" label="Clip percent" /></label>
      </div>
    </div>
  </FloatCard>
</template>

<style scoped>
.cardbody {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.plot {
  fill: var(--field);
  stroke: var(--line);
}
.luma {
  fill: var(--line-strong);
  stroke: none;
}
.marker {
  fill: var(--text);
  cursor: ew-resize;
}
.sm {
  gap: 3px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.sm :deep(.num) {
  width: 100%;
}
.clip {
  gap: 6px;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
