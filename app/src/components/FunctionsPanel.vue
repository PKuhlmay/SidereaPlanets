<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import {
  BRIGHTNESS_RANGE,
  CONTRAST_RANGE,
  contrastField,
  contrastSlider,
  functionNames,
} from "../logic/settings";
import { settings } from "../state";
import NumberField from "./NumberField.vue";

// Right panel of the Wavelet tab. Every function button is a switch for its own panel; those panels are not built
// yet (RK-008), so the buttons only keep their pressed state.
const wavelet = settings.wavelet;

function toggle(name: string) {
  const at = wavelet.functions.indexOf(name);
  if (at >= 0) wavelet.functions.splice(at, 1);
  else wavelet.functions.push(name);
}

const contrast = computed({
  get: () => contrastField(wavelet.contrast),
  set: (field: number) => (wavelet.contrast = contrastSlider(field)),
});

function resetContrast() {
  wavelet.contrast = 250;
  wavelet.brightness = 0;
}
</script>

<template>
  <aside class="side right" aria-label="Functions">
    <div class="grp">
      <div class="gh">Functions</div>
      <div class="fx">
        <button
          v-for="name in functionNames"
          :key="name"
          type="button"
          class="btn wrap"
          :aria-pressed="wavelet.functions.includes(name)"
          @click="toggle(name)"
        >
          {{ name }}
        </button>
      </div>
    </div>

    <div class="grp">
      <div class="gh">
        Contrast/Brightness
        <span>
          <button type="button" class="link" :aria-pressed="wavelet.holdContrast" @click="wavelet.holdContrast = !wavelet.holdContrast">Hold</button>
          <button type="button" class="link" @click="resetContrast">Reset</button>
        </span>
      </div>
      <input v-model.number="wavelet.contrast" type="range" :min="CONTRAST_RANGE.min" :max="CONTRAST_RANGE.max" aria-label="Contrast" />
      <input v-model.number="wavelet.brightness" type="range" :min="BRIGHTNESS_RANGE.min" :max="BRIGHTNESS_RANGE.max" aria-label="Brightness" />
      <div class="grid2 fields">
        <label class="col">Contrast <NumberField v-model="contrast" /></label>
        <label class="col">Brightness <NumberField v-model="wavelet.brightness" /></label>
      </div>
    </div>

    <div class="grp">
      <div class="seg">
        <button type="button" @click="runAction('copyTo')">Copy To</button>
        <button type="button" @click="runAction('loadTo')">Load to</button>
        <button type="button" disabled @click="runAction('difference')">Difference</button>
      </div>
      <div class="sub">Toggle</div>
      <div class="col">
        <label class="chk"><input type="radio" name="toggle" checked />Current Image</label>
        <label class="chk"><input type="radio" name="toggle" disabled />Clipboard Image</label>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.fx {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
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
.fields {
  margin-top: 4px;
}
.fields .col {
  gap: 3px;
}
.fields :deep(.num) {
  width: 100%;
}
</style>
