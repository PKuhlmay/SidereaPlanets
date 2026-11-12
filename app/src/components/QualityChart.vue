<script setup lang="ts">
import { computed } from "vue";
import { qualityChart } from "../logic/qualityChart";
import { session, settings } from "../state";

// Quality of the frames in the Limit step: one bar per frame, best first, kept frames in the accent colour, the limit
// as a dashed line. Follows every change of the Limit Setup at once (docs/design.md, "4 Limit").
const W = 248;
const H = 76;
const pad = { left: 3, right: 3, top: 12, bottom: 3 };
const plotW = W - pad.left - pad.right;
const plotH = H - pad.top - pad.bottom;

const chart = computed(() => qualityChart(session.frames, settings.align, plotW));

const bars = computed(() => {
  const list = chart.value.bars;
  const step = list.length > 0 ? plotW / list.length : 0;
  const gap = step >= 3 ? 1 : 0;
  return list.map((bar, i) => {
    const h = Math.max(1, bar.height * plotH);
    return { x: pad.left + i * step, y: pad.top + plotH - h, w: Math.max(0.5, step - gap), h, kept: bar.kept };
  });
});

const line = computed(() => {
  const l = chart.value.line;
  if (!l) return null;
  if (l.kind === "level") {
    const y = pad.top + plotH * (1 - l.at);
    return { x1: pad.left, y1: y, x2: W - pad.right, y2: y, tx: W - pad.right, ty: y - 3, anchor: "end" };
  }
  const x = pad.left + plotW * l.at;
  const right = x < W - 40;
  return { x1: x, y1: pad.top - 2, x2: x, y2: H - pad.bottom, tx: right ? x + 4 : x - 4, ty: pad.top - 3, anchor: right ? "start" : "end" };
});
</script>

<template>
  <div class="quality">
    <svg :viewBox="`0 0 ${W} ${H}`" width="100%" role="img" :aria-label="chart.label">
      <rect
        v-for="(bar, i) in bars"
        :key="i"
        :x="bar.x"
        :y="bar.y"
        :width="bar.w"
        :height="bar.h"
        :class="bar.kept ? 'kept' : 'dropped'"
      />
      <template v-if="line">
        <line :x1="line.x1" :y1="line.y1" :x2="line.x2" :y2="line.y2" class="limit" stroke-dasharray="3 3" />
        <text :x="line.tx" :y="line.ty" :text-anchor="line.anchor" class="value">{{ chart.value }}</text>
      </template>
    </svg>
    <div class="ends"><span>best frame</span><span>weakest</span></div>
  </div>
</template>

<style scoped>
.quality {
  margin-top: 8px;
}
svg {
  display: block;
  background: var(--field);
  border: 1px solid var(--line);
  border-radius: var(--r-s);
}
.kept {
  fill: var(--accent);
}
.dropped {
  fill: var(--line-strong);
}
.limit {
  stroke: var(--text);
  stroke-width: 1;
}
.value {
  font-size: 10px;
  font-variant-numeric: tabular-nums;
  fill: var(--muted);
}
.ends {
  margin-top: 2px;
  font-size: 10px;
}
</style>
