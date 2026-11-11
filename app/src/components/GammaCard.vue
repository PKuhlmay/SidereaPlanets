<script setup lang="ts">
import { computed } from "vue";
import { closeFunction } from "../colour";
import { TONE_LIMITS } from "../logic/settings";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import NumberField from "./NumberField.vue";

// Gamma card: one slider (0.01 steps) and one field. Above 1.00 the midtones get brighter.
const tone = settings.wavelet.tone;
const { min, max } = TONE_LIMITS.gamma;

const slider = computed({
  get: () => Math.round(tone.gamma * 100),
  set: (position: number) => (tone.gamma = position / 100),
});
</script>

<template>
  <FloatCard class="fixed" title="Gamma" @close="closeFunction('Gamma')">
    <template #actions>
      <button type="button" class="link" @click="tone.gamma = 1">Reset</button>
    </template>
    <div class="cardbody">
      <div class="ends"><span>darker</span><span>brighter</span></div>
      <input v-model.number="slider" type="range" :min="min * 100" :max="max * 100" aria-label="Gamma" />
      <label class="row">Gamma <NumberField v-model="tone.gamma" :decimals="2" :step="0.01" :min="min" :max="max" label="Gamma value" /></label>
    </div>
  </FloatCard>
</template>
