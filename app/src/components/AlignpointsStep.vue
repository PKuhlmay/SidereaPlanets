<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { setThreshold } from "../align";
import { atLeast } from "../logic/workflow";
import { session, settings, status } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 2 Alignpoints: the Threshold slider moves the weakest points out of `session.alignpoints` through the backend
// (align.ts) and is fixed once Align has run. Draw Region draws the region Set Alignpoints and Align work in.
const align = settings.align;
const hasPoints = computed(() => atLeast(session.step, "points"));
// The region is part of Set Alignpoints and Align, so it only changes before Align.
const canDrawRoi = computed(() => (session.step === "loaded" || session.step === "points") && status.busy === null);
</script>

<template>
  <StepPanel id="alignpoints" :more="5">
    <div class="thr" title="How strong a detail must be to get an alignpoint; left keeps more points, right fewer.">
      <b>{{ hasPoints ? `Threshold · ${session.alignpoints.length} points` : "Threshold" }}</b>
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
      <div class="ends"><span>more points</span><span>fewer points</span></div>
    </div>
    <label class="row gap" title="Smallest distance between two alignpoints, in pixels.">
      <span class="wrap">Min. distance between points (px)</span> <NumberField v-model="align.minDistance" :width="52" />
    </label>
    <label class="row" title="Alignpoints keep at least this distance from the edge of the image or the region, in pixels.">
      <span class="wrap">Min. distance from image edge (px)</span> <NumberField v-model="align.edgeDistance" :width="52" />
    </label>
    <button
      type="button"
      class="btn roi"
      :disabled="!canDrawRoi"
      :aria-pressed="session.roiDrawing"
      title="Region of interest (ROI): drag a rectangle on the image so that only this area is used for placing alignpoints and aligning; press again to remove it."
      @click="runAction('regionOfInterest')"
    >
      Draw Region
    </button>

    <template #advanced>
      <div class="grp">
        <div class="gh">Point Placement</div>
        <div class="sub first" title="How the brightness at a point is measured when alignpoints are placed.">Brightness measure</div>
        <div class="col">
          <label class="chk" title="The centre pixel."><input v-model="align.intensity" type="radio" name="intensity" value="default" />Default</label>
          <label class="chk" title="The mean of the 3×3 pixels around the centre.">
            <input v-model="align.intensity" type="radio" name="intensity" value="area3x3" />3×3 area
          </label>
          <label class="chk" title="The darkest of the 3×3 pixels around the centre.">
            <input v-model="align.intensity" type="radio" name="intensity" value="lowest" />Lowest pixel value
          </label>
        </div>
        <label class="row" title="Only pixels between Min. and Max. brightness (0–255) can get an alignpoint.">
          Min. brightness (0–255) <NumberField v-model="align.lo" :width="52" />
        </label>
        <label class="row" title="Only pixels between Min. and Max. brightness (0–255) can get an alignpoint.">
          Max. brightness (0–255) <NumberField v-model="align.hi" :width="52" />
        </label>
        <label class="row">Number of alignpoints <NumberField :model-value="session.alignpoints.length" readonly :width="52" /></label>
        <div class="inl" title="Not available yet">
          <label class="chk"><input type="checkbox" disabled />Keep points inside the scan frame</label>
          <span class="factor">Not available yet</span>
        </div>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.thr {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.thr b {
  font-size: var(--fs-s);
  font-weight: 600;
  text-align: center;
}
.gap {
  margin-top: 6px;
}
/* a long label takes two lines instead of pushing its field out of the panel */
.wrap {
  white-space: normal;
}
.roi {
  width: 100%;
  margin-top: 6px;
}
.first {
  margin-top: 0;
}
.inl {
  display: flex;
  flex-wrap: wrap;
  gap: 0 6px;
  align-items: center;
  min-height: 29px;
}
.factor {
  margin-left: auto;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
