<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { setThreshold } from "../align";
import { atLeast } from "../logic/workflow";
import { session, settings, status } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 2 Alignpoints: the Threshold slider moves the weakest points out of `session.alignpoints` through the backend
// (align.ts) and is fixed once Align has run. R.o.Interest draws the region Set Alignpoints and Align work in.
const align = settings.align;
const hasPoints = computed(() => atLeast(session.step, "points"));
// The region is part of Set Alignpoints and Align, so it only changes before Align.
const canDrawRoi = computed(() => (session.step === "loaded" || session.step === "points") && status.busy === null);
</script>

<template>
  <StepPanel id="alignpoints" :more="5">
    <div class="ends">
      <span>weakest</span><b>{{ hasPoints ? `Alignpoints: ${session.alignpoints.length}` : "Threshold" }}</b><span>strongest</span>
    </div>
    <!-- No range before Set Alignpoints: the slider stays at 0 and looks locked -->
    <input
      type="range"
      min="0"
      :max="session.alignpointTotal"
      :value="align.threshold"
      :disabled="session.alignpointTotal === 0 || session.step !== 'points' || status.busy !== null"
      aria-label="Threshold"
      @input="setThreshold(Number(($event.target as HTMLInputElement).value))"
    />
    <label class="row gap">Minimum distance between <NumberField v-model="align.minDistance" /></label>
    <label class="row">Min distance from edge <NumberField v-model="align.edgeDistance" /></label>
    <button
      type="button"
      class="btn roi"
      :disabled="!canDrawRoi"
      :aria-pressed="session.roiDrawing"
      title="Draw the region that alignment is based on; press again to drop it"
      @click="runAction('regionOfInterest')"
    >
      R.o.Interest
    </button>

    <template #advanced>
      <div class="grp">
        <div class="gh">Set Alignpoint parameters</div>
        <div class="sub first">Intensity selection</div>
        <div class="two">
          <div class="col">
            <label class="chk"><input v-model="align.intensity" type="radio" name="intensity" value="default" />Default</label>
            <label class="chk"><input v-model="align.intensity" type="radio" name="intensity" value="area3x3" />3x3 area</label>
            <label class="chk"><input v-model="align.intensity" type="radio" name="intensity" value="lowest" />Lowest pixelvalue</label>
          </div>
          <div class="col lohi">
            <label class="row">Lo <NumberField v-model="align.lo" /></label>
            <label class="row">Hi <NumberField v-model="align.hi" /></label>
          </div>
        </div>
        <label class="row">Number of Alignpoints <NumberField :model-value="session.alignpoints.length" readonly /></label>
        <label class="chk"><input type="checkbox" disabled />Keep Alignpoints inside ScanFrame</label>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.gap {
  margin-top: 6px;
}
.roi {
  width: 100%;
  margin-top: 6px;
}
.first {
  margin-top: 0;
}
.lohi {
  gap: 6px;
}
.lohi .row {
  min-height: 0;
}
</style>
