<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { isMono } from "../colour";
import { defaultLayer, formatPreview, LAYER_RANGE } from "../logic/settings";
import { linkedLevels } from "../logic/wavelet";
import { settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 6 Wavelets: Colour, LRGB and the six layers; the scheme settings, the schemes and Realign under Advanced.
// The values live in `settings.wavelet`; wavelet.ts calculates the preview whenever one of them changes.
const wavelet = settings.wavelet;
// Grayscale recordings have no colour to process: Colour and LRGB are locked, LRGB also without Colour.
const mono = computed(() => isMono());

function resetWavelets() {
  wavelet.layers.forEach((layer) => Object.assign(layer, defaultLayer()));
}

// With Use Linked Wavelets a moved slider takes the others along.
function moveSlider(index: number, event: Event) {
  const next = linkedLevels(
    wavelet.layers.map((l) => l.level),
    index,
    Number((event.target as HTMLInputElement).value),
    wavelet.linked,
  );
  next.forEach((level, i) => (wavelet.layers[i].level = level));
}
</script>

<template>
  <StepPanel id="wavelets" :more="8">
    <div class="colour">
      <label class="chk"><input v-model="settings.colour" type="checkbox" :disabled="mono" />Colour</label>
      <label class="chk"><input v-model="settings.lrgb" type="checkbox" :disabled="mono || !settings.colour" />LRGB</label>
    </div>
    <div class="grp">
      <div class="gh">Wavelets <button type="button" class="link" @click="resetWavelets">Reset Wavelets</button></div>
      <div class="head"><span>Layer</span><span>Denoise</span><span>Sharpen</span><span>Preview</span></div>
      <div v-for="(layer, i) in wavelet.layers" :key="i" class="layer">
        <label class="chk"><input v-model="layer.enabled" type="checkbox" />{{ i + 1 }}</label>
        <NumberField v-model="layer.denoise" :decimals="2" :step="0.01" :label="`Denoise layer ${i + 1}`" />
        <NumberField v-model="layer.sharpen" :decimals="3" :step="0.005" :label="`Sharpen layer ${i + 1}`" />
        <button type="button" class="btn small" :title="`Preview layer ${i + 1}: Not available yet`" disabled>{{ formatPreview(layer.level) }}</button>
        <input :value="layer.level" type="range" :min="LAYER_RANGE.min" :max="LAYER_RANGE.max" :aria-label="`Layer ${i + 1}`" @input="moveSlider(i, $event)" />
      </div>
    </div>

    <template #advanced>
      <div class="grp">
        <div class="gh">Wavelets</div>
        <div class="col">
          <label class="chk"><input v-model="wavelet.automatic" type="checkbox" />Automatic</label>
          <label class="chk"><input v-model="wavelet.hold" type="checkbox" />Hold Wavelet Setting</label>
        </div>
        <div class="sub">Waveletscheme</div>
        <div class="seg">
          <button type="button" :aria-pressed="wavelet.scheme === 'dyadic'" @click="wavelet.scheme = 'dyadic'">Dyadic (2^n)</button>
          <button type="button" :aria-pressed="wavelet.scheme === 'linear'" @click="wavelet.scheme = 'linear'">Linear</button>
        </div>
        <div class="row nowrap">
          <label class="row">Initial Layer <NumberField v-model="wavelet.initialLayer" :width="46" /></label>
          <label class="row" title="Step Increment">Step Incr. <NumberField v-model="wavelet.stepIncrement" :width="46" /></label>
        </div>
        <div class="sub">Wavelet filter</div>
        <div class="seg">
          <button type="button" :aria-pressed="wavelet.filter === 'default'" @click="wavelet.filter = 'default'">Default</button>
          <button type="button" :aria-pressed="wavelet.filter === 'gaussian'" @click="wavelet.filter = 'gaussian'">Gaussian</button>
        </div>
        <label class="chk linked"><input v-model="wavelet.linked" type="checkbox" />Use Linked Wavelets</label>
      </div>

      <div class="grp">
        <div class="gh">Available schemes</div>
        <select class="wide" aria-label="Available schemes"><option /></select>
        <div class="grid2 buttons">
          <button type="button" class="btn" @click="runAction('loadScheme')">Load Scheme</button>
          <button type="button" class="btn" @click="runAction('saveScheme')">Save Scheme</button>
        </div>
      </div>

      <div class="grp">
        <div class="col buttons">
          <div class="locked" title="Not available yet">
            <button type="button" class="btn" disabled>Realign with Processed</button>
            <span class="factor">Not available yet</span>
          </div>
          <button type="button" class="btn" disabled>Stack Again</button>
        </div>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.colour {
  display: flex;
  gap: 24px;
}
.nowrap {
  margin-top: 6px;
  white-space: nowrap;
}
.linked {
  margin-top: 6px;
}
.head,
.layer {
  display: grid;
  grid-template-columns: 30px 1fr 1fr 42px;
  gap: 2px 8px;
  align-items: center;
}
.head {
  font-size: var(--fs-s);
  color: var(--muted);
}
.layer {
  padding: 3px 0 2px;
}
.layer :deep(.num) {
  width: auto;
}
.layer input[type="range"] {
  grid-column: 1 / -1;
}
.wide {
  width: 100%;
}
.buttons {
  gap: 6px;
  margin-top: 8px;
}
.locked {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.factor {
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
