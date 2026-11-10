<script setup lang="ts">
import { computed } from "vue";
import { showFrame } from "../recording";
import { atLeast } from "../logic/workflow";
import { session } from "../state";
import NumberField from "./NumberField.vue";

// Frame bar of the Align tab: status, Goto Frame, a slider over all frames and the position. Empty before a
// recording is open.
const count = computed(() => session.info?.frameCount ?? 0);
const aligned = computed(() => atLeast(session.step, "aligned"));

// Goto Frame counts from 1, the slider and the backend from 0.
const goto = computed({
  get: () => session.frame + 1,
  set: (frame: number) => void showFrame(frame - 1),
});
</script>

<template>
  <div class="framebar">
    <template v-if="session.info">
      <span class="state" :class="{ on: aligned }"><i />{{ aligned ? "Aligned" : "Loaded" }}</span>
      <label class="goto">Goto Frame <NumberField v-model="goto" :min="1" :max="count" label="Goto Frame" /></label>
      <input
        type="range"
        min="0"
        :max="Math.max(0, count - 1)"
        :value="session.frame"
        aria-label="Frame"
        @input="showFrame(Number(($event.target as HTMLInputElement).value))"
      />
      <span class="position">Frame ({{ session.frame + 1 }}): {{ session.frame + 1 }}/{{ count }}</span>
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
.position {
  min-width: 118px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
