<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";

// A toolbar menu: borderless button with an arrow, the list opens below it. Escape and a click outside close it.
defineProps<{ label: string; wide?: boolean }>();

const open = ref(false);
const root = ref<HTMLElement | null>(null);

const close = () => (open.value = false);

function onDocumentPointer(event: PointerEvent) {
  if (open.value && root.value && !root.value.contains(event.target as Node)) close();
}
function onKey(event: KeyboardEvent) {
  if (event.key === "Escape" && open.value) {
    event.stopPropagation();
    close();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocumentPointer);
  window.addEventListener("keydown", onKey, true);
});
onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onDocumentPointer);
  window.removeEventListener("keydown", onKey, true);
});
</script>

<template>
  <div ref="root" class="mw">
    <button type="button" class="btn ghost caret" aria-haspopup="menu" :aria-expanded="open" @click="open = !open">{{ label }}</button>
    <div v-if="open" class="menu" :class="{ wide }" role="menu">
      <slot :close="close" />
    </div>
  </div>
</template>

<style scoped>
.mw {
  position: relative;
}
.menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 30;
  min-width: 270px;
  padding: 4px;
  background: var(--raised);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
}
.menu.wide {
  min-width: 310px;
}
</style>
