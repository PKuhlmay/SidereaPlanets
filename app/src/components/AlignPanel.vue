<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { setThreshold } from "../align";
import { atLeast } from "../logic/workflow";
import { session, settings, status } from "../state";
import NumberField from "./NumberField.vue";

// Left panel of the Align tab. Everything is locked until a recording is open. The Threshold slider moves the
// weakest points out of `session.alignpoints` through the backend (align.ts) and is fixed once Align has run.
const align = settings.align;
const hasPoints = computed(() => atLeast(session.step, "points"));
// The region is part of Set Alignpoints and Align, so it only changes before Align.
const canDrawRoi = computed(() => (session.step === "loaded" || session.step === "points") && status.busy === null);
const countMode = computed(() => align.limitMode === "framesPerPoint" || align.limitMode === "bestCount");
</script>

<template>
  <fieldset class="side" :disabled="session.step === 'empty'" aria-label="Alignment">
    <div class="grp">
      <div class="gh">Set Alignpoint parameters</div>
      <label class="row">Minimum distance between <NumberField v-model="align.minDistance" /></label>
      <label class="row">Min distance from edge <NumberField v-model="align.edgeDistance" /></label>
      <div class="sub">Intensity selection</div>
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
      <label class="row">Number of Alignpoints <NumberField :model-value="session.alignpoints.length" readonly /></label>
      <label class="chk"><input type="checkbox" disabled />Keep Alignpoints inside ScanFrame</label>
    </div>

    <div class="grp">
      <div class="gh">
        Alignment setup
        <label class="chk plain"><input v-model="align.showAlignment" type="checkbox" />Show Alignment</label>
      </div>
      <div class="seg">
        <button type="button" @click="runAction('noAlign')">No Align</button>
        <button
          type="button"
          :disabled="!canDrawRoi"
          :aria-pressed="session.roiDrawing"
          title="Draw the region that alignment is based on; press again to drop it"
          @click="runAction('regionOfInterest')"
        >
          R.o.Interest
        </button>
        <button type="button" :disabled="!atLeast(session.step, 'aligned')" @click="runAction('scanFrames')">Scan Frames</button>
      </div>
      <label class="row first"><span>Alignmentbox size</span> <NumberField v-model="align.boxSize" /></label>
      <label class="row">Max Alignpoint movement <NumberField v-model="align.maxMovement" /></label>
      <!-- Locked as observed in the original; what unlocks them is not known (docs/ui.md) -->
      <label class="chk"><input type="checkbox" disabled />Align by Centre of gravity</label>
      <label class="row indent">Lum. Threshold <NumberField :model-value="0" disabled /></label>
      <label class="chk"><input type="checkbox" disabled />Estimate Rotation</label>
    </div>

    <div class="grp">
      <div class="gh">Limit Setup</div>
      <div class="two">
        <div class="col">
          <label class="chk"><input v-model="align.limitMode" type="radio" name="limit" value="lowestQuality" />Lowest Quality (%)</label>
          <label class="chk"><input v-model="align.limitMode" type="radio" name="limit" value="bestPercent" />Best Frames (%)</label>
          <label class="chk"><input v-model="align.limitMode" type="radio" name="limit" value="framesPerPoint" />Frames/Apoint</label>
          <label class="chk"><input v-model="align.limitMode" type="radio" name="limit" value="bestCount" />Best Frames</label>
        </div>
        <div class="col values">
          <NumberField v-model="align.limitPercent" :disabled="countMode" label="Limit percent" />
          <NumberField v-model="align.limitCount" :disabled="!countMode" label="Limit frame count" />
        </div>
      </div>
    </div>
  </fieldset>
</template>

<style scoped>
.lohi {
  gap: 6px;
}
.lohi .row {
  min-height: 0;
}
.ends {
  margin-top: 10px;
}
.plain {
  font-weight: 400;
}
.first {
  margin-top: 6px;
}
.indent {
  padding-left: 20px;
}
.values {
  gap: 22px;
}
</style>
