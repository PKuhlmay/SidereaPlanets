<script setup lang="ts">
import { computed } from "vue";
import { closeFunction } from "../colour";
import { clampWeight, mixPresets } from "../logic/colour";
import { IDENTITY_MIX, TONE_LIMITS } from "../logic/settings";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import MenuButton from "./MenuButton.vue";
import NumberField from "./NumberField.vue";

// Color Mixing card: every output channel is the sum of red, green and blue times three weights, in percent. Neutral is
// 100 on the diagonal. It works on gray images too, where all three input channels are equal.
const mix = settings.wavelet.tone.mix;
const { min, max } = TONE_LIMITS.mix;
const channels = ["Red", "Green", "Blue"] as const;
const isIdentity = computed(() => mix.matrix.every((w, i) => w === IDENTITY_MIX[i]));

const at = (output: number, input: number) => output * 3 + input;
const setWeight = (output: number, input: number, value: number) => (mix.matrix[at(output, input)] = clampWeight(value));

function choose(matrix: number[], close: () => void) {
  close();
  mix.matrix = [...matrix];
}
</script>

<template>
  <FloatCard class="fixed" title="Color Mixing" @close="closeFunction('Color Mixing')">
    <template #actions>
      <button type="button" class="link" :disabled="isIdentity" @click="mix.matrix = [...IDENTITY_MIX]">Reset</button>
    </template>
    <div class="cardbody">
      <div class="row">
        Preset
        <MenuButton v-slot="{ close }" label="Choose" plain end title="Sets all nine weights at once">
          <button v-for="preset in mixPresets" :key="preset.name" type="button" class="mi" @click="choose(preset.matrix, close)">
            {{ preset.name }}
          </button>
        </MenuButton>
      </div>
      <div v-for="(output, o) in channels" :key="output">
        <div class="sub">{{ output }} output</div>
        <div v-for="(input, i) in channels" :key="input" class="line">
          <span>{{ input }}</span>
          <input
            :value="mix.matrix[at(o, i)]"
            type="range"
            :min="min"
            :max="max"
            :aria-label="`${input} in ${output.toLowerCase()} output`"
            @input="setWeight(o, i, Number(($event.target as HTMLInputElement).value))"
          />
          <span class="pct">
            <NumberField
              :model-value="mix.matrix[at(o, i)]"
              :min="min"
              :max="max"
              :width="56"
              :label="`${input} in ${output.toLowerCase()} output percent`"
              @update:model-value="setWeight(o, i, $event)"
            />%
          </span>
        </div>
      </div>
      <label class="chk keep" title="Keeps the brightness of every pixel while the colors change.">
        <input v-model="mix.preserveBrightness" type="checkbox" />Preserve brightness
      </label>
    </div>
  </FloatCard>
</template>

<style scoped>
.sub {
  margin: 6px 0 0;
}
.line {
  display: grid;
  grid-template-columns: 40px 1fr auto;
  gap: 8px;
  align-items: center;
  min-height: 28px;
}
.pct {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  font-size: var(--fs-s);
  color: var(--muted);
}
.keep {
  margin-top: 6px;
}
.mi {
  display: flex;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  text-align: left;
  white-space: nowrap;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: var(--r-s);
}
.mi:hover {
  background: var(--accent-soft);
}
</style>
