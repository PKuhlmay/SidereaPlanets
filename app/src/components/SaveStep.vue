<script setup lang="ts">
import { computed } from "vue";
import { canCrop, removeCrop, startCrop } from "../cropTool";
import { sizeText } from "../logic/crop";
import { resultSize } from "../geometryView";
import { orientationSummary, sizeLabel } from "../logic/geometry";
import { session, settings, status } from "../state";
import { progress } from "../steps";
import StepPanel from "./StepPanel.vue";

// Step 7 Save: what Save Image writes, the group Crop that opens the crop tool (SP-021) and the group Size (SP-022). The action with the menu of file types is
// in the foot of the panel.
// The group Size: what Save Image writes after Flip and Rotate, Crop and Resize Image (SP-022).
const result = computed(() => resultSize());
const turned = computed(() => orientationSummary(settings.geometry.orientation));
const summary = computed(() => (session.crop.applied ? `Only ${sizeText(session.crop.applied)} is saved.` : "The whole picture is saved."));
</script>

<template>
  <StepPanel id="save">
    <p class="note">Saves the sharpened stack with your color and tone settings.</p>
    <p v-if="!progress.waveletsDone" class="note">Your latest changes are applied to the whole image first (Do All).</p>
    <div class="grp">
      <div class="gh" title="Cuts the picture to a rectangle when it is saved.">Crop</div>
      <p class="note">{{ summary }}</p>
      <div class="line">
        <button
          type="button"
          class="btn"
          :disabled="!canCrop() || session.crop.editing"
          title="Draws the part of the picture to keep, as Free, 1:1, 3:2 or 4:3."
          @click="startCrop"
        >
          Crop…
        </button>
        <button
          v-if="session.crop.applied"
          type="button"
          class="link"
          :disabled="status.busy !== null"
          title="Saves the whole picture again."
          @click="removeCrop"
        >
          Remove Crop
        </button>
      </div>
    </div>
    <div v-if="result" class="grp">
      <div class="gh" title="The size of the file that Save Image writes.">Size</div>
      <p class="note">The picture is saved at {{ sizeLabel(result) }}.</p>
      <p v-if="turned" class="note">It is {{ turned }}.</p>
      <p class="note">Flip and Rotate and Resize Image are in the Adjustments of the step Wavelets.</p>
    </div>
  </StepPanel>
</template>

<style scoped>
.note {
  margin: 0 0 4px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.grp {
  margin-top: 8px;
  border-top: 1px solid var(--line);
}
.line {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-top: 4px;
}
</style>
