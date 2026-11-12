<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { seriesPoints } from "../logic/chart";
import { atLeast, imageKind } from "../logic/workflow";
import { session, settings } from "../state";
import FloatCard from "./FloatCard.vue";

// Framelist, Registrationgraph (steps up to Limit) and Alignmentpoints (Stack step) as cards over the image. A card is
// visible while its check mark in the Show menu is set.
const kind = computed(() => imageKind(session.view));
const showFramelist = computed(() => kind.value === "frame" && settings.align.showFramelist && session.step !== "empty");
const showGraph = computed(() => kind.value === "frame" && settings.align.showRegistrationgraph && session.step !== "empty");
const showPoints = computed(() => kind.value === "stack" && settings.stack.showAlignSections && atLeast(session.step, "limited"));

const rated = computed(() => session.frames.some((f) => f.quality !== null));

// After Align the best frame comes first.
const listed = computed(() => {
  const frames = [...session.frames];
  return rated.value ? frames.sort((a, b) => (b.quality ?? -1) - (a.quality ?? -1)) : frames;
});

const frameName = (index: number) => `Frame${String(index + 1).padStart(5, "0")}`;
const setAll = (checked: boolean) => session.frames.forEach((f) => (f.checked = checked));

const chart = { left: 8, top: 10, width: 198, height: 78 };
const quality = computed(() => seriesPoints(listed.value.map((f) => f.quality ?? 0), chart));
const difference = computed(() => seriesPoints(listed.value.map((f) => f.difference ?? 0), chart));
</script>

<template>
  <FloatCard v-if="showFramelist" class="grow" title="Framelist" @close="settings.align.showFramelist = false">
    <div class="tools">
      <button type="button" class="btn" @click="setAll(true)">Check all</button>
      <button type="button" class="btn" @click="setAll(false)">Check none</button>
      <button type="button" class="btn" @click="runAction('emptyList')">Empty List</button>
    </div>
    <div class="list">
      <label v-for="frame in listed" :key="frame.index" class="chk">
        <input v-model="frame.checked" type="checkbox" />{{ frameName(frame.index) }}
        <span v-if="frame.quality !== null" class="q">{{ frame.quality.toFixed(1) }}%</span>
      </label>
    </div>
  </FloatCard>

  <FloatCard v-if="showGraph" title="Registrationgraph" @close="settings.align.showRegistrationgraph = false">
    <div class="body">
      <svg viewBox="0 0 214 96" width="100%" role="img" aria-label="Quality and difference per frame">
        <text v-if="!rated" x="107" y="52" text-anchor="middle" font-size="11" fill="currentColor" opacity="0.6">Run Align to see the graph</text>
        <template v-else>
          <path d="M8 10V88H206" fill="none" stroke="currentColor" opacity="0.25" />
          <polyline :points="difference" fill="none" stroke="var(--info)" stroke-width="2" stroke-linejoin="round" />
          <polyline :points="quality" fill="none" stroke="var(--accent)" stroke-width="2" stroke-linejoin="round" />
        </template>
      </svg>
      <div class="legend">
        <span><i style="background: var(--accent)" />Quality</span>
        <span><i style="background: var(--info)" />Difference</span>
      </div>
    </div>
  </FloatCard>

  <FloatCard v-if="showPoints" class="grow" title="Alignmentpoints" @close="settings.stack.showAlignSections = false">
    <div class="list">
      <label v-for="(point, i) in session.alignpoints" :key="i" class="chk"><input v-model="point.checked" type="checkbox" />Point {{ i + 1 }}</label>
    </div>
  </FloatCard>
</template>

<style scoped>
.grow {
  flex: 0 1 auto;
}
.tools {
  display: flex;
  gap: 4px;
  padding: 6px 8px 2px;
}
.tools .btn {
  flex: 1;
  height: 24px;
  padding: 0 4px;
  font-size: var(--fs-s);
}
.list {
  padding: 4px 10px 8px;
  overflow: auto;
  font-variant-numeric: tabular-nums;
}
.list .chk {
  display: flex;
}
.q {
  margin-left: auto;
  color: var(--muted);
}
.body {
  padding: 8px 10px 10px;
}
.legend {
  display: flex;
  gap: 14px;
  margin-top: 4px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.legend i {
  display: inline-block;
  width: 14px;
  height: 2px;
  margin-right: 6px;
  vertical-align: middle;
  border-radius: 1px;
}
</style>
