<script setup lang="ts">
import { computed } from "vue";
import { cancelCrop, cropImageSize, finishCrop, resetCrop, setCropFormat } from "../cropTool";
import { CROP_FORMATS, isFull, sizeText } from "../logic/crop";
import { session } from "../state";
import FloatCard from "./FloatCard.vue";

// The card of the crop tool (docs/design.md "Zuschneiden"): Format, the size of the rectangle, Reset, Cancel and Done. Closing
// the card is Cancel. The rectangle itself is drawn by CropOverlay and dragged in ImageViewer.
const wholeImage = computed(() => {
  const draft = session.crop.draft;
  const size = cropImageSize();
  return !draft || !size || isFull(draft, size.width, size.height);
});
const size = computed(() => (session.crop.draft ? sizeText(session.crop.draft) : ""));
</script>

<template>
  <FloatCard class="fixed" title="Crop" @close="cancelCrop">
    <template #actions>
      <button type="button" class="link" title="Back to the whole image." @click="resetCrop">Reset</button>
    </template>
    <div class="cardbody">
      <div class="sub first">Format</div>
      <div class="seg">
        <button
          v-for="format in CROP_FORMATS"
          :key="format.id"
          type="button"
          :aria-pressed="session.crop.format === format.id"
          :title="format.aspect ? `Keeps the shape ${format.title} while the rectangle is dragged.` : 'Any shape.'"
          @click="setCropFormat(format.id)"
        >
          {{ format.title }}
        </button>
      </div>
      <div class="row"><span class="muted">Size</span><span aria-label="Size of the crop">{{ size }}</span></div>
      <p class="hint">
        {{
          wholeImage
            ? "Drag on the image to draw the part to keep."
            : "Drag the rectangle to move it and its handles to resize it. Save Image writes only what is inside."
        }}
      </p>
      <div class="buttons">
        <button type="button" class="btn" title="Closes the crop tool and drops the changes." @click="cancelCrop">Cancel</button>
        <button type="button" class="btn next" title="Keeps the rectangle: Save Image writes only what is inside." @click="finishCrop">Done</button>
      </div>
    </div>
  </FloatCard>
</template>

<style scoped>
.first {
  margin-top: 0;
}
.muted,
.hint {
  font-size: var(--fs-s);
  color: var(--muted);
}
.hint {
  margin: 2px 0 8px;
}
.buttons {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
</style>
