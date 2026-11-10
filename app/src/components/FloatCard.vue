<script setup lang="ts">
// A floating card over the image (Framelist, Registrationgraph, Alignmentpoints); the cross removes its check mark.
defineProps<{ title: string }>();
const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <section class="float" :aria-label="title">
    <header>
      {{ title }}
      <button type="button" aria-label="Close" @click="emit('close')">×</button>
    </header>
    <slot />
  </section>
</template>

<style scoped>
.float {
  display: flex;
  flex-direction: column;
  min-height: 0;
  cursor: default;
  background: color-mix(in srgb, var(--panel) 94%, transparent);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(10px);
  pointer-events: auto;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 6px 6px 10px;
  font-weight: 600;
  border-bottom: 1px solid var(--line);
}
header button {
  width: 22px;
  height: 22px;
  font-size: 15px;
  line-height: 1;
  color: var(--muted);
  cursor: pointer;
  background: none;
  border: 0;
  border-radius: var(--r-s);
}
header button:hover {
  color: var(--text);
  background: var(--raised-h);
}
</style>
