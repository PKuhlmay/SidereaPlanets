<script setup lang="ts">
import { ref, watch } from "vue";
import { runAction } from "../actions";
import { exportFormats, saveChoice, type ExportFormatInfo, type ImageSource } from "../saving";
import MenuButton from "./MenuButton.vue";

// Save Image: the format is chosen first (the Stack step offers seven, the Save step five), then the file dialog opens.
// In the step panel the button takes the full width and the list opens above it; `filled` makes it the step's action.
const props = defineProps<{ source: ImageSource; disabled: boolean; filled?: boolean }>();

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
  <MenuButton
    v-slot="{ close }"
    label="Save Image"
    plain
    :filled="filled"
    block
    up
    :disabled="disabled"
    title="Saves the image as a file; choose the file type first."
  >
    <button v-for="format in formats" :key="format.id" type="button" class="mi" @click="choose(format, close)">
      {{ format.label }} · .{{ format.extension }}…
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
</style>
