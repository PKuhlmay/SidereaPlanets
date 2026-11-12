<script setup lang="ts">
import { computed } from "vue";
import { imageKind } from "../logic/workflow";
import { session, settings } from "../state";
import MenuButton from "./MenuButton.vue";

// The row at the top of the image area (docs/design.md, "Show am Bild"): what the image shows at the left; Functions »
// (Wavelets) and the Show menu with the check boxes of the former action row at the right.
defineProps<{ caption: string }>();

const kind = computed(() => imageKind(session.view));
</script>

<template>
  <div class="topbar">
    <span v-if="caption" class="chip">{{ caption }}</span>
    <span class="gap" />
    <button
      v-if="session.info && session.view === 'wavelets'"
      type="button"
      class="chip ctl"
      :aria-pressed="settings.wavelet.functionsOpen"
      title="Show or hide the Functions panel"
      @click="settings.wavelet.functionsOpen = !settings.wavelet.functionsOpen"
    >
      Functions »
    </button>
    <MenuButton v-if="session.info && session.view !== 'open'" class="show" label="Show" end>
      <template v-if="kind === 'frame'">
        <label class="mi"><input v-model="settings.align.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="mi"><input v-model="settings.align.showRoi" type="checkbox" :disabled="!session.roi" />Show ROI</label>
        <label class="mi"><input v-model="settings.align.showAlignpoints" type="checkbox" />Show Alignpoints</label>
        <label class="mi"><input v-model="settings.align.showAligndata" type="checkbox" />Show Aligndata</label>
        <label class="mi"><input v-model="settings.align.showFramelist" type="checkbox" />Show Framelist</label>
        <label class="mi"><input v-model="settings.align.showRegistrationgraph" type="checkbox" />Show Registrationgraph</label>
        <label class="mi"><input v-model="settings.align.showPrefilter" type="checkbox" />Show Prefilter</label>
      </template>
      <template v-else-if="kind === 'stack'">
        <label class="mi"><input v-model="settings.stack.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="mi"><input v-model="settings.stack.showAlignSections" type="checkbox" />Show AlignSections</label>
        <label class="mi"><input v-model="settings.stack.showStackgraph" type="checkbox" />Show Stackgraph</label>
        <label class="mi"><input v-model="settings.stack.showStacklist" type="checkbox" />Show Stacklist</label>
      </template>
      <template v-else>
        <label class="mi"><input v-model="settings.wavelet.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="mi"><input v-model="settings.wavelet.showAlignpoints" type="checkbox" />Show AlignPoints</label>
        <label class="mi"><input v-model="settings.wavelet.showProcessingArea" type="checkbox" />Show Processing Area</label>
      </template>
    </MenuButton>
  </div>
</template>

<style scoped>
.topbar {
  position: absolute;
  top: 10px;
  right: 10px;
  left: 10px;
  z-index: 2;
  display: flex;
  gap: 6px;
  align-items: center;
  pointer-events: none;
}
.gap {
  flex: 1;
}
.chip,
.show :deep(.btn) {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 10px;
  font-size: var(--fs-s);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--on-canvas);
  white-space: nowrap;
  background: var(--canvas-chrome);
  border: 1px solid var(--canvas-line);
  border-radius: var(--r-l);
  backdrop-filter: blur(10px);
}
.ctl,
.show {
  pointer-events: auto;
}
.ctl {
  cursor: pointer;
}
/* on the always dark image area the accent itself reads in both themes */
.ctl[aria-pressed="true"] {
  color: var(--accent);
  background: var(--accent-soft);
  border-color: var(--accent);
}
.show :deep(.btn:hover),
.ctl:not([aria-pressed="true"]):hover {
  background: var(--canvas-line);
}
.mi {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  border-radius: var(--r-s);
  cursor: pointer;
}
.mi:hover:not(:has(input:disabled)) {
  background: var(--accent-soft);
}
.mi:has(input:disabled) {
  color: var(--off);
  cursor: default;
}
</style>
