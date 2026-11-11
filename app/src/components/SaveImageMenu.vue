<script setup lang="ts">
import { ref, watch } from "vue";
import { runAction } from "../actions";
import { exportFormats, saveChoice, type ExportFormatInfo, type ImageSource } from "../saving";
import MenuButton from "./MenuButton.vue";

// Save image: the format is chosen first (the Stack tab offers seven, the Wavelet tab five), then the file dialog opens.
const props = defineProps<{ source: ImageSource; disabled: boolean }>();

const formats = ref<ExportFormatInfo[]>([]);

watch(
  () => props.source,
  async (source) => {
    try {
      formats.value = await exportFormats(source);
    } catch {
      formats.value = [];
    }
  },
  { immediate: true },
);

function choose(format: ExportFormatInfo, close: () => void) {
  saveChoice.format = format.id;
  close();
  void runAction("saveImage");
}
</script>

<template>
  <MenuButton v-slot="{ close }" label="Save image" plain :disabled="disabled">
    <button v-for="format in formats" :key="format.id" type="button" class="mi" @click="choose(format, close)">
      {{ format.label }}<span class="ext">.{{ format.extension }}</span>
    </button>
  </MenuButton>
</template>

<style scoped>
.mi {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  text-align: left;
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: var(--r-s);
}
.mi:hover {
  background: var(--accent-soft);
}
.ext {
  margin-left: auto;
  padding-left: 16px;
  color: var(--muted);
}
</style>
