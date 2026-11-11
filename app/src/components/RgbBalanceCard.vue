<script setup lang="ts">
import { computed } from "vue";
import { closeFunction, isMono } from "../colour";
import { TONE_LIMITS } from "../logic/settings";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import NumberField from "./NumberField.vue";

// RGB Balance card: a slider and a field per channel, in percent. The channels are told apart by their labels.
const tone = settings.wavelet.tone;
const { min, max } = TONE_LIMITS.balance;
const mono = computed(() => isMono());
const channels = [
  { key: "red", label: "Red" },
  { key: "green", label: "Green" },
  { key: "blue", label: "Blue" },
] as const;

function reset() {
  tone.red = tone.green = tone.blue = 100;
}
</script>

<template>
  <FloatCard class="fixed" title="RGB Balance" @close="closeFunction('RGB Balance')">
    <template #actions>
      <button type="button" class="link" :disabled="mono" @click="reset">Reset</button>
    </template>
    <fieldset class="cardbody" :disabled="mono" :title="mono ? 'Needs a colour image' : undefined">
      <div v-for="channel in channels" :key="channel.key" class="line">
        <span>{{ channel.label }}</span>
        <input v-model.number="tone[channel.key]" type="range" :min="min" :max="max" :aria-label="channel.label" />
        <NumberField v-model="tone[channel.key]" :min="min" :max="max" :width="56" :label="`${channel.label} percent`" />
      </div>
    </fieldset>
  </FloatCard>
</template>

<style scoped>
.line {
  display: grid;
  grid-template-columns: 40px 1fr auto;
  gap: 8px;
  align-items: center;
  min-height: 30px;
}
</style>
