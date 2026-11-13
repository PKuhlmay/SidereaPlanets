<script setup lang="ts">
import { progress } from "../steps";
import StepPanel from "./StepPanel.vue";

// Step 7 Save: what Save Image writes; the action with the menu of file types is in the foot of the panel.
</script>

<template>
  <StepPanel id="save">
    <p class="note">Saves the sharpened stack with your color and tone settings.</p>
    <p v-if="!progress.waveletsDone" class="note">Your latest changes are applied to the whole image first (Do All).</p>
  </StepPanel>
</template>

<style scoped>
.note {
  margin: 0 0 4px;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
