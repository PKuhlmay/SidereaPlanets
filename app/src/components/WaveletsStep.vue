<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { isMono } from "../colour";
import { defaultLayer, formatPreview, LAYER_RANGE } from "../logic/settings";
import { linkedLevels } from "../logic/wavelet";
import { settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 6 Wavelets: Color, Sharpen brightness only and the six layers; the wavelet settings, the schemes and Realign under
// Advanced. The values live in `settings.wavelet`; wavelet.ts calculates the preview whenever one of them changes.
const wavelet = settings.wavelet;
// Grayscale recordings have no color to process: Color and Sharpen brightness only are locked, the latter also without Color.
const mono = computed(() => isMono());

function resetWavelets() {
  wavelet.layers.forEach((layer) => Object.assign(layer, defaultLayer()));
}

// With Move all sliders together a moved slider takes the others along.
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
      <label class="chk" :title="mono ? 'Needs a color image' : 'Processes the channels in color; off gives grayscale.'">
        <input v-model="settings.colour" type="checkbox" :disabled="mono" />Color
      </label>
      <label class="chk" :title="mono ? 'Needs a color image' : 'Sharpens only the brightness of the stack and keeps its colors.'">
        <input v-model="settings.lrgb" type="checkbox" :disabled="mono || !settings.colour" />Sharpen brightness only
      </label>
    </div>
    <div class="grp">
      <div class="gh">Wavelets <button type="button" class="link" @click="resetWavelets">Reset Wavelets</button></div>
      <div class="head">
        <span title="Layer 1 holds the finest detail, higher layers coarser structures; switch a layer off to leave it out.">Layer</span>
        <span title="Smooths the noise in this layer.">Denoise</span>
        <span title="Fine adjustment of this layer; the slider sets the main strength.">Sharpen</span>
        <span>Preview</span>
      </div>
      <div v-for="(layer, i) in wavelet.layers" :key="i" class="layer">
        <label class="chk"><input v-model="layer.enabled" type="checkbox" :aria-label="`Layer ${i + 1}`" />{{ i + 1 }}</label>
        <NumberField v-model="layer.denoise" :decimals="2" :step="0.01" :label="`Denoise layer ${i + 1}`" />
        <NumberField v-model="layer.sharpen" :decimals="3" :step="0.005" :label="`Sharpen layer ${i + 1}`" />
        <button type="button" class="btn small eye" :title="`Preview layer ${i + 1}: Not available yet`" :aria-label="`Preview layer ${i + 1}`" disabled><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M1.5 8s2.4-4.5 6.5-4.5S14.5 8 14.5 8 12.1 12.5 8 12.5 1.5 8 1.5 8z" /><circle cx="8" cy="8" r="2" /></svg></button>
        <input
          :value="layer.level"
          type="range"
          class="level"
          :min="LAYER_RANGE.min"
          :max="LAYER_RANGE.max"
          :aria-label="`Layer ${i + 1}`"
          :title="`Strength of layer ${i + 1}`"
          @input="moveSlider(i, $event)"
        />
        <span class="value">{{ formatPreview(layer.level) }}</span>
      </div>
    </div>

    <template #advanced>
      <div class="grp">
        <div class="gh">Wavelet Settings</div>
        <div class="col">
          <label class="chk" title="Applies the sharpening to the whole image half a second after each change.">
            <input v-model="wavelet.automatic" type="checkbox" />Run Do All automatically
          </label>
          <label class="chk" title="Keeps the layer values when a new stack is made.">
            <input v-model="wavelet.hold" type="checkbox" />Keep layer values for a new stack
          </label>
        </div>
        <div class="sub" title="Linear: the layers grow evenly. Dyadic: each layer doubles in size.">Layer spacing</div>
        <div class="seg">
          <button type="button" :aria-pressed="wavelet.scheme === 'dyadic'" title="Each layer doubles in size." @click="wavelet.scheme = 'dyadic'">
            Dyadic (2^n)
          </button>
          <button type="button" :aria-pressed="wavelet.scheme === 'linear'" title="The layers grow evenly." @click="wavelet.scheme = 'linear'">Linear</button>
        </div>
        <label class="row gap" title="Size of the finest layer.">First layer size <NumberField v-model="wavelet.initialLayer" :width="52" /></label>
        <label class="row" title="How much each further layer grows.">Size step <NumberField v-model="wavelet.stepIncrement" :width="52" /></label>
        <div class="sub">Wavelet filter</div>
        <div class="seg">
          <button type="button" :aria-pressed="wavelet.filter === 'default'" title="The standard filter." @click="wavelet.filter = 'default'">Default</button>
          <button type="button" :aria-pressed="wavelet.filter === 'gaussian'" title="Smoother than Default." @click="wavelet.filter = 'gaussian'">
            Gaussian
          </button>
        </div>
        <label class="chk linked" title="Moving one slider moves all the others by the same amount.">
          <input v-model="wavelet.linked" type="checkbox" />Move all sliders together
        </label>
      </div>

      <div class="grp">
        <div class="gh">Saved Schemes</div>
        <!-- The list of saved schemes has no function yet; Load Scheme… and Save Scheme… work on a file. -->
        <select class="wide" aria-label="Saved schemes" title="Not available yet" disabled><option /></select>
        <div class="grid2 buttons">
          <button type="button" class="btn" title="Loads the layer values from a wavelet scheme file (.rwv)." @click="runAction('loadScheme')">Load Scheme…</button>
          <button type="button" class="btn" title="Saves the layer values as a wavelet scheme file (.rwv)." @click="runAction('saveScheme')">Save Scheme…</button>
        </div>
      </div>

      <div class="grp">
        <div class="col buttons">
          <div class="locked" title="Not available yet">
            <button type="button" class="btn" disabled>Realign with Processed</button>
            <span class="factor">Not available yet</span>
          </div>
          <button type="button" class="btn" title="Not available yet" disabled>Stack Again</button>
        </div>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.colour {
  display: flex;
  gap: 16px;
}
.gap {
  margin-top: 6px;
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
/* the slider takes the first three columns, its value stands right of it under the Preview button */
.layer .level {
  grid-column: 1 / 4;
}
.value {
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
  text-align: right;
}
.eye svg {
  display: block;
  margin: 0 auto;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.3;
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
