<script setup lang="ts">
import { computed } from "vue";
import { showFrame, showNextFrame } from "../recording";
import { atLeast } from "../logic/workflow";
import { session } from "../state";
import NumberField from "./NumberField.vue";

// Frame bar of the Alignpoints, Align and Best Frames steps: status, Go to frame, previous and next frame (← and →), a slider over
// all frames and the position. Empty before a recording is open.
const count = computed(() => session.info?.frameCount ?? 0);
const aligned = computed(() => atLeast(session.step, "aligned"));

// Go to frame counts from 1, the slider and the backend from 0.
const goto = computed({
  get: () => session.frame + 1,
  set: (frame: number) => void showFrame(frame - 1),
});
</script>

<template>
  <div class="framebar">
    <template v-if="session.info">
      <span class="state" :class="{ on: aligned }"><i />{{ aligned ? "Aligned" : "Loaded" }}</span>
      <label class="goto">Go to frame <NumberField v-model="goto" :min="1" :max="count" label="Go to frame" /></label>
      <span class="steps">
        <button type="button" class="nav" :disabled="session.frame <= 0" title="Previous frame (←)" aria-label="Previous frame" @click="showNextFrame(-1)">
          <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M10 3.5 5.5 8l4.5 4.5" /></svg>
        </button>
        <button
          type="button"
          class="nav"
          :disabled="session.frame >= count - 1"
          title="Next frame (→)"
          aria-label="Next frame"
          @click="showNextFrame(1)"
        >
          <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M6 3.5 10.5 8 6 12.5" /></svg>
        </button>
      </span>
      <input
        type="range"
        min="0"
        :max="Math.max(0, count - 1)"
        :value="session.frame"
        aria-label="Frame"
        @input="showFrame(Number(($event.target as HTMLInputElement).value))"
      />
      <span class="position">Frame {{ session.frame + 1 }} of {{ count }}</span>
    </template>
  </div>
</template>

<style scoped>
.framebar {
  display: flex;
  gap: 14px;
  align-items: center;
  height: 40px;
  padding: 0 12px;
  background: var(--panel);
  border-top: 1px solid var(--line);
}
.state {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  min-width: 76px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.state i {
  width: 9px;
  height: 9px;
  background: var(--ok);
  border-radius: 50%;
  opacity: 0.45;
}
.state.on i {
  opacity: 1;
}
.state.on {
  font-weight: 600;
  color: var(--ok);
}
.goto {
  display: flex;
  gap: 8px;
  align-items: center;
  white-space: nowrap;
}
.steps {
  display: flex;
  gap: 2px;
}
.nav {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  padding: 0;
  color: var(--text);
  background: none;
  border: 0;
  border-radius: var(--r-s);
  cursor: pointer;
}
.nav:hover:not(:disabled) {
  background: var(--raised-h);
}
.nav:disabled {
  color: var(--off);
  cursor: default;
}
.nav svg {
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.position {
  min-width: 118px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
