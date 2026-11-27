<script setup lang="ts">
import { computed } from "vue";
import { closeFunction } from "../colour";
import { resizeSource, resultSize } from "../geometryView";
import {
  defaultResize,
  RESIZE_LIMITS,
  RESIZE_METHODS,
  setResizeHeight,
  setResizeLock,
  setResizeWidth,
  sizeLabel,
  switchToPixels,
} from "../logic/geometry";
import { settings } from "../state";
import FloatCard from "./FloatCard.vue";
import MenuButton from "./MenuButton.vue";
import NumberField from "./NumberField.vue";

// Resize Image card (SP-022): scales the picture by a percentage or to a size in pixels when it is saved. It does not change the
// preview; the card tells the size that is written.
const r = settings.geometry.resize;
const isDefault = computed(() => JSON.stringify(r) === JSON.stringify(defaultResize()));
const result = computed(() => resultSize());
const methodTitle = computed(() => RESIZE_METHODS.find((m) => m.id === r.method)?.title ?? "");

function setMode(mode: "percent" | "pixels") {
  const source = resizeSource();
  if (mode === "pixels" && source) switchToPixels(r, source);
  else r.mode = "percent";
}

const percent = computed({
  get: () => r.percent,
  set: (value: number) => (r.percent = Math.min(RESIZE_LIMITS.percent.max, Math.max(RESIZE_LIMITS.percent.min, Math.round(value)))),
});
const width = computed({
  get: () => result.value?.width ?? 0,
  set: (value: number) => {
    const source = resizeSource();
    if (source) setResizeWidth(r, value, source);
  },
});
const height = computed({
  get: () => result.value?.height ?? 0,
  set: (value: number) => {
    const source = resizeSource();
    if (source) setResizeHeight(r, value, source);
  },
});
function setLock(on: boolean) {
  const source = resizeSource();
  if (source) setResizeLock(r, on, source);
}
</script>

<template>
  <FloatCard class="fixed" title="Resize Image" @close="closeFunction('Resize Image')">
    <template #actions>
      <button type="button" class="link" :disabled="isDefault" title="The picture keeps its size." @click="Object.assign(r, defaultResize())">Reset</button>
    </template>
    <div class="cardbody">
      <div class="seg">
        <button type="button" :aria-pressed="r.mode === 'percent'" title="Scales by a share of the size." @click="setMode('percent')">Percent</button>
        <button type="button" :aria-pressed="r.mode === 'pixels'" title="Scales to a width and a height in pixels." @click="setMode('pixels')">Pixels</button>
      </div>
      <template v-if="r.mode === 'percent'">
        <label class="row gap">
          Size (%) <NumberField v-model="percent" :min="RESIZE_LIMITS.percent.min" :max="RESIZE_LIMITS.percent.max" :width="56" label="Size percent" />
        </label>
        <input v-model.number="percent" type="range" :min="RESIZE_LIMITS.percent.min" :max="RESIZE_LIMITS.percent.max" aria-label="Size" />
      </template>
      <template v-else>
        <label class="row gap">
          Width (px) <NumberField v-model="width" :min="RESIZE_LIMITS.side.min" :max="RESIZE_LIMITS.side.max" :width="64" label="Width in pixels" />
        </label>
        <label class="row">
          Height (px) <NumberField v-model="height" :min="RESIZE_LIMITS.side.min" :max="RESIZE_LIMITS.side.max" :width="64" label="Height in pixels" />
        </label>
        <label class="chk" title="Keeps the shape of the picture: the other side follows the one you type.">
          <input type="checkbox" :checked="r.lockAspect" @change="setLock(($event.target as HTMLInputElement).checked)" />Lock aspect ratio
        </label>
      </template>
      <div class="row" title="How the new pixels are calculated. Lanczos3 keeps the most detail.">
        Method
        <MenuButton v-slot="{ close }" :label="methodTitle" plain end title="How the new pixels are calculated">
          <button v-for="method in RESIZE_METHODS" :key="method.id" type="button" class="mi" @click="close(), (r.method = method.id)">
            {{ method.title }}
          </button>
        </MenuButton>
      </div>
      <div class="row"><span class="muted">Result</span><span aria-label="Size of the saved picture">{{ result ? sizeLabel(result) : "" }}</span></div>
      <p class="muted note">The picture is scaled when it is saved, after flipping, turning and cropping.</p>
    </div>
  </FloatCard>
</template>

<style scoped>
.gap {
  margin-top: 6px;
}
.muted {
  font-size: var(--fs-s);
  color: var(--muted);
}
.note {
  margin: 2px 0 0;
}
.mi {
  display: flex;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  text-align: left;
  white-space: nowrap;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: var(--r-s);
}
.mi:hover {
  background: var(--accent-soft);
}
</style>
