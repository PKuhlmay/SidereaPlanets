<script setup lang="ts">
import { computed } from "vue";
import { settings } from "../state";
import NumberField from "./NumberField.vue";
import QualityChart from "./QualityChart.vue";
import StepPanel from "./StepPanel.vue";

// Step 4 Limit: the whole Limit Setup and the quality of the frames; nothing under Advanced.
const align = settings.align;
const countMode = computed(() => align.limitMode === "framesPerPoint" || align.limitMode === "bestCount");
</script>

<template>
  <StepPanel id="limit">
    <!-- The one group of the step, Limit Setup, without its heading (docs/design.md, "Schrittpanel") -->
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
    <QualityChart />
  </StepPanel>
</template>

<style scoped>
.values {
  gap: 22px;
}
</style>
