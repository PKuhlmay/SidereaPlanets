<script setup lang="ts">
import { computed } from "vue";
import { showItems } from "../logic/menu";
import { imageKind } from "../logic/workflow";
import { session, settings } from "../state";
import MenuButton from "./MenuButton.vue";

// The row at the top of the image area (docs/design.md, "Show am Bild"): what the image shows at the left; Adjust ›
// (Wavelets) and the Show menu with the check boxes of the former action row at the right. The check marks that change
// nothing on screen yet are dimmed under "Not available yet".
defineProps<{ caption: string }>();

const kind = computed(() => imageKind(session.view));
const items = computed(() => showItems(kind.value));
const available = computed(() => items.value.filter((item) => item.available));
const locked = computed(() => items.value.filter((item) => !item.available));
// The check marks of a kind of image live in their own settings group; the keys come from `showItems`.
const group = computed(
  () => (kind.value === "frame" ? settings.align : kind.value === "stack" ? settings.stack : settings.wavelet) as unknown as Record<string, boolean>,
);
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
      title="Show or hide the panel with color, tone and other image adjustments"
      @click="settings.wavelet.functionsOpen = !settings.wavelet.functionsOpen"
    >
      Adjust ›
    </button>
    <MenuButton v-if="session.info && session.view !== 'open'" class="show" label="Show" end title="What the image and the cards show">
      <label v-for="item in available" :key="item.key" class="mi">
        <input v-model="group[item.key]" type="checkbox" :disabled="item.key === 'showRoi' && !session.roi" />{{ item.title }}
      </label>
      <template v-if="locked.length > 0">
        <hr />
        <div class="cap">Not available yet</div>
        <label v-for="item in locked" :key="item.key" class="mi" title="Not available yet">
          <input type="checkbox" :checked="false" disabled />{{ item.title }}
        </label>
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
hr {
  margin: 4px 0;
  border: 0;
  border-top: 1px solid var(--line);
}
.cap {
  padding: 4px 8px 2px;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
