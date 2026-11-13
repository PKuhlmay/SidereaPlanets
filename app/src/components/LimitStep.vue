<script setup lang="ts">
import { computed } from "vue";
import { settings } from "../state";
import NumberField from "./NumberField.vue";
import QualityChart from "./QualityChart.vue";
import StepPanel from "./StepPanel.vue";

// Step 4 Best Frames: the whole Limit Setup and the quality of the frames; nothing under Advanced.
const align = settings.align;
const countMode = computed(() => align.limitMode === "framesPerPoint" || align.limitMode === "bestCount");
</script>

<template>
  <StepPanel id="limit">
    <!-- The one group of the step, Limit Setup, without its heading (docs/design.md, "Schrittpanel"). The field of each kind of
         mode sits in the row of its first mode and is locked while the other kind is chosen. -->
    <div class="mode">
      <label class="chk" title="Keep every frame whose quality is at least this share of the best frame.">
        <input v-model="align.limitMode" type="radio" name="limit" value="lowestQuality" />At least % of the best quality
      </label>
      <NumberField v-model="align.limitPercent" :disabled="countMode" :width="52" label="Limit percent" />
    </div>
    <div class="mode">
      <label class="chk" title="Keep this share of all frames, the sharpest ones.">
        <input v-model="align.limitMode" type="radio" name="limit" value="bestPercent" />Best % of frames
      </label>
    </div>
    <div class="mode">
      <label class="chk" title="Each alignpoint uses its own sharpest frames; the field sets how many.">
        <input v-model="align.limitMode" type="radio" name="limit" value="framesPerPoint" />Best frames per point
      </label>
      <NumberField v-model="align.limitCount" :disabled="!countMode" :width="52" label="Limit frame count" />
    </div>
    <div class="mode">
      <label class="chk" title="Keep exactly this many frames, the sharpest ones.">
        <input v-model="align.limitMode" type="radio" name="limit" value="bestCount" />Best number of frames
      </label>
    </div>
    <QualityChart />
  </StepPanel>
</template>

<style scoped>
.mode {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  min-height: 29px;
}
</style>
