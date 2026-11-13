<script setup lang="ts">
import { computed } from "vue";
import { autoAlign, closeFunction, isMono } from "../colour";
import { defaultShifts, TONE_LIMITS } from "../logic/settings";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import NumberField from "./NumberField.vue";

// RGB Align card: shift of each channel in pixels. Auto fills in red and blue so that they lie on green.
const tone = settings.wavelet.tone;
const limit = TONE_LIMITS.shift;
const mono = computed(() => isMono());
const channels = [
  { key: "red", label: "Red" },
  { key: "green", label: "Green" },
  { key: "blue", label: "Blue" },
] as const;
</script>

<template>
  <FloatCard class="fixed" title="RGB Align" @close="closeFunction('RGB Align')">
    <template #actions>
      <button type="button" class="link" :disabled="mono" @click="autoAlign">Auto</button>
      <button type="button" class="link" :disabled="mono" @click="tone.shifts = defaultShifts()">Reset</button>
    </template>
    <fieldset class="cardbody" :disabled="mono" :title="mono ? 'Needs a color image' : undefined">
      <div class="line heads"><span /><span>X (px)</span><span>Y (px)</span></div>
      <div v-for="channel in channels" :key="channel.key" class="line">
        <span>{{ channel.label }}</span>
        <NumberField v-model="tone.shifts[channel.key].x" :decimals="2" :step="0.1" :min="-limit" :max="limit" :width="68" :label="`${channel.label} X`" />
        <NumberField v-model="tone.shifts[channel.key].y" :decimals="2" :step="0.1" :min="-limit" :max="limit" :width="68" :label="`${channel.label} Y`" />
      </div>
      <label class="row">
        Max shift (px)
        <NumberField v-model="tone.maxShift" :min="TONE_LIMITS.maxShift.min" :max="TONE_LIMITS.maxShift.max" :width="56" label="Max shift in pixels" />
      </label>
      <p class="hint">Auto moves Red and Blue onto Green</p>
    </fieldset>
  </FloatCard>
</template>

<style scoped>
.line {
  display: grid;
  grid-template-columns: 44px 1fr 1fr;
  gap: 6px;
  align-items: center;
  min-height: 30px;
}
.heads {
  min-height: 18px;
  font-size: var(--fs-s);
  color: var(--muted);
  text-align: right;
}
.line :deep(.num) {
  justify-self: end;
}
.hint {
  margin: 4px 0 0;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
