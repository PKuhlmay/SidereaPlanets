<script setup lang="ts">
import { progress } from "../steps";
import StepPanel from "./StepPanel.vue";

// Step 7 Save: what Save image writes; the action with the menu of file types is in the foot of the panel.
</script>

<template>
  <StepPanel id="save">
    <p class="note">The sharpened stack with the colour and tone settings.</p>
    <p v-if="!progress.waveletsDone" class="note">Do All runs first with the current values.</p>
  </StepPanel>
</template>

<style scoped>
.note {
  margin: 0 0 4px;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
