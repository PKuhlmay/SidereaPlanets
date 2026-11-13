<script setup lang="ts">
import { closeFunction } from "../colour";
import { defaultRefine, REFINE_LIMITS } from "../logic/settings";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import RefineSlider from "./RefineSlider.vue";

// Denoise/Deringing card (MAC-018): edge-preserving smoothing, and the dark ring next to bright edges lifted back to the level of the
// stack. Both start from the sharpened image and the stack it was made from (crates/core/src/refine.rs).
const refine = settings.refine;

function reset() {
  const d = defaultRefine();
  Object.assign(refine, { denoise: d.denoise, deringing: d.deringing, radius: d.radius });
}
</script>

<template>
  <FloatCard class="fixed" title="Denoise/Deringing" @close="closeFunction('Denoise/Deringing')">
    <template #actions>
      <button type="button" class="link" @click="reset">Reset</button>
    </template>
    <div class="cardbody">
      <RefineSlider
        v-model="refine.denoise"
        title="Denoise (%)"
        label="Denoise percent"
        v-bind="REFINE_LIMITS.percent"
        help="How strongly noise is smoothed. Edges, such as the edge of the planet, stay sharp."
      />
      <RefineSlider
        v-model="refine.deringing"
        title="Deringing (%)"
        label="Deringing percent"
        v-bind="REFINE_LIMITS.percent"
        help="How much of the dark ring next to bright edges is lifted back to the level of the original stack."
      />
      <RefineSlider
        v-model="refine.radius"
        title="Radius (px)"
        label="Deringing radius in pixels"
        v-bind="REFINE_LIMITS.radius"
        help="How far from an edge Deringing looks for the dark ring."
      />
      <p class="hint">Deringing compares the sharpened image with the stack</p>
    </div>
  </FloatCard>
</template>

<style scoped>
.hint {
  margin: 6px 0 0;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
