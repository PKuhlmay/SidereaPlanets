<script setup lang="ts">
import { runAction } from "../actions";
import { atLeast } from "../logic/workflow";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 3 Align: the key fields of Alignment setup, the rest under Advanced.
const align = settings.align;
</script>

<template>
  <StepPanel id="align" :more="4">
    <div class="seg">
      <button type="button" @click="runAction('noAlign')">No Align</button>
      <button type="button" :disabled="!atLeast(session.step, 'aligned')" @click="runAction('scanFrames')">Scan Frames</button>
    </div>
    <label class="row gap">Alignmentbox size <NumberField v-model="align.boxSize" /></label>
    <label class="row">Max Alignpoint movement <NumberField v-model="align.maxMovement" /></label>

    <template #advanced>
      <div class="grp">
        <div class="gh">Alignment setup</div>
        <label class="chk"><input v-model="align.showAlignment" type="checkbox" />Show Alignment</label>
        <!-- Locked as observed in the original; what unlocks them is not known (docs/ui.md) -->
        <label class="chk"><input type="checkbox" disabled />Align by Centre of gravity</label>
        <label class="row indent">Lum. Threshold <NumberField :model-value="0" disabled /></label>
        <label class="chk"><input type="checkbox" disabled />Estimate Rotation</label>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.gap {
  margin-top: 6px;
}
.indent {
  padding-left: 20px;
}
</style>
