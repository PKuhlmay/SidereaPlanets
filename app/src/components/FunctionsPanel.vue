<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { toggleFunction } from "../colour";
import { functionHelp, isFunctionBuilt } from "../logic/colour";
import {
  BRIGHTNESS_RANGE,
  CONTRAST_RANGE,
  contrastField,
  contrastSlider,
  functionNames,
} from "../logic/settings";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";

// Right panel of the Wavelets step (Adjustments). Every button is a switch for its own card over the image
// (FunctionCards.vue); the buttons whose card does not exist yet are dimmed and locked. Contrast/Brightness and
// Remember · Restore · Difference live here.
const wavelet = settings.wavelet;
const hasImage = computed(() => (session.processedImage ?? session.stackImage) !== null);
const hasCopy = computed(() => session.clipboardImage !== null);

const contrast = computed({
  get: () => contrastField(wavelet.contrast),
  set: (field: number) => (wavelet.contrast = contrastSlider(field)),
});

// A name with a slash may wrap after it ("Denoise/" over "Deringing"); the button is a third of the panel wide.
const breakable = (name: string) => name.replace("/", "/\u200b");

function resetContrast() {
  wavelet.contrast = 250;
  wavelet.brightness = 0;
}
</script>

<template>
  <aside class="side right" aria-label="Adjustments">
    <div class="grp">
      <div class="gh">Adjustments</div>
      <div class="fx">
        <button
          v-for="name in functionNames"
          :key="name"
          type="button"
          class="btn wrap"
          :aria-pressed="name === 'Crop' ? session.crop.editing : wavelet.functions.includes(name)"
          :disabled="!isFunctionBuilt(name)"
          :title="functionHelp(name)"
          @click="toggleFunction(name)"
        >
          {{ breakable(name) }}
        </button>
      </div>
    </div>

    <div class="grp">
      <div class="gh">
        Contrast/Brightness
        <span>
          <!-- Keep has no effect yet (the state is only stored), so it is locked. -->
          <button type="button" class="link" disabled title="Not available yet">Keep</button>
          <button type="button" class="link" @click="resetContrast">Reset</button>
        </span>
      </div>
      <!-- Each slider has its label and its field right above it. -->
      <label class="row">Contrast <NumberField v-model="contrast" :width="62" /></label>
      <input v-model.number="wavelet.contrast" type="range" :min="CONTRAST_RANGE.min" :max="CONTRAST_RANGE.max" aria-label="Contrast" />
      <label class="row gap">Brightness <NumberField v-model="wavelet.brightness" :width="62" /></label>
      <input v-model.number="wavelet.brightness" type="range" :min="BRIGHTNESS_RANGE.min" :max="BRIGHTNESS_RANGE.max" aria-label="Brightness" />
    </div>

    <div class="grp">
      <div class="seg">
        <button type="button" :disabled="!hasImage" title="Remembers the current image so that it can be compared later." @click="runAction('copyTo')">
          Remember
        </button>
        <button type="button" :disabled="!hasCopy" title="Brings the remembered image back." @click="runAction('loadTo')">Restore</button>
        <button
          type="button"
          :disabled="!hasCopy"
          :aria-pressed="session.toneView === 'difference'"
          title="Shows where the current image differs from the remembered one, amplified."
          @click="runAction('difference')"
        >
          Difference
        </button>
      </div>
      <div class="sub">Show:</div>
      <div class="col">
        <label class="chk"><input v-model="session.toneView" type="radio" name="toggle" value="current" />Current image</label>
        <label class="chk"><input v-model="session.toneView" type="radio" name="toggle" value="clipboard" :disabled="!hasCopy" />Remembered image</label>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.fx {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 6px;
}
.wrap {
  height: auto;
  min-height: 38px;
  padding: 3px 4px;
  font-size: var(--fs-s);
  line-height: 1.2;
  white-space: normal;
}
.gap {
  margin-top: 4px;
}
</style>
