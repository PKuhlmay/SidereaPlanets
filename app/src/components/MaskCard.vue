<script setup lang="ts">
import { closeFunction } from "../colour";
import { defaultRefine, REFINE_LIMITS } from "../logic/settings";
import { session, settings } from "../state";
import FloatCard from "./FloatCard.vue";
import RefineSlider from "./RefineSlider.vue";

// Masking card: limits the sharpening to the bright part of the stack (the planet), with a soft edge. Show mask shows the
// mask in the image area while the card is open (colour.ts); it is only a view and never saved.
const refine = settings.refine;

function reset() {
  const d = defaultRefine();
  Object.assign(refine, { maskOn: d.maskOn, threshold: d.threshold, feather: d.feather, invert: d.invert });
  session.showMask = false;
}
</script>

<template>
  <FloatCard class="fixed" title="Masking" @close="closeFunction('Masking')">
    <template #actions>
      <button type="button" class="link" @click="reset">Reset</button>
    </template>
    <div class="cardbody">
      <label class="chk" title="Sharpens only where the stack is brighter than the threshold. Everywhere else the unsharpened stack stays.">
        <input v-model="refine.maskOn" type="checkbox" />Use mask
      </label>
      <RefineSlider
        v-model="refine.threshold"
        title="Threshold"
        label="Threshold"
        v-bind="REFINE_LIMITS.threshold"
        help="Brightness of the stack, from 0 (black) to 255 (white), above which the image is sharpened."
      />
      <RefineSlider
        v-model="refine.feather"
        title="Feather (px)"
        label="Feather in pixels"
        v-bind="REFINE_LIMITS.feather"
        help="Width of the soft edge between the sharpened and the unsharpened part."
      />
      <div class="checks">
        <label class="chk" title="Sharpens the dark part instead of the bright part."><input v-model="refine.invert" type="checkbox" />Invert</label>
        <label class="chk" title="Tints the part that is sharpened blue. This only changes what you see here, not the image.">
          <input v-model="session.showMask" type="checkbox" />Show mask
        </label>
      </div>
    </div>
  </FloatCard>
</template>

<style scoped>
.checks {
  display: flex;
  gap: 14px;
  margin-top: 6px;
}
</style>
