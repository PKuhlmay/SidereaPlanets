<script setup lang="ts">
import { runAction } from "../actions";
import { defaultLayer, formatPreview, LAYER_RANGE } from "../logic/settings";
import { linkedLevels } from "../logic/wavelet";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";

// Left panel of the Wavelet tab. The values live in `settings.wavelet`; wavelet.ts calculates the preview whenever
// one of them changes.
const wavelet = settings.wavelet;

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
  <fieldset class="side" :disabled="session.step === 'empty'" aria-label="Wavelets">
    <div class="grp">
      <div class="gh">Wavelets <button type="button" class="link" @click="resetWavelets">Reset Wavelets</button></div>
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

      <div class="head"><span>Layer</span><span>Denoise</span><span>Sharpen</span><span>Preview</span></div>
      <div v-for="(layer, i) in wavelet.layers" :key="i" class="layer">
        <label class="chk"><input v-model="layer.enabled" type="checkbox" />{{ i + 1 }}</label>
        <NumberField v-model="layer.denoise" :decimals="2" :step="0.01" :label="`Denoise layer ${i + 1}`" />
        <NumberField v-model="layer.sharpen" :decimals="3" :step="0.005" :label="`Sharpen layer ${i + 1}`" />
        <button type="button" class="btn small" :title="`Preview layer ${i + 1}`" @click="runAction('previewLayer')">{{ formatPreview(layer.level) }}</button>
        <input :value="layer.level" type="range" :min="LAYER_RANGE.min" :max="LAYER_RANGE.max" :aria-label="`Layer ${i + 1}`" @input="moveSlider(i, $event)" />
      </div>
    </div>

    <div class="grp">
      <div class="gh">Available schemes</div>
      <select class="wide" aria-label="Available schemes"><option /></select>
      <div class="grid2 buttons">
        <button type="button" class="btn" @click="runAction('loadScheme')">Load Scheme</button>
        <button type="button" class="btn" @click="runAction('saveScheme')">Save Scheme</button>
      </div>
    </div>
  </fieldset>
</template>

<style scoped>
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
  margin-top: 6px;
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
  margin-top: 8px;
}
</style>
