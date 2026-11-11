<script setup lang="ts">
// A floating card over the image (Framelist, Registrationgraph, Alignmentpoints, the Functions cards); the cross
// removes its check mark. Actions such as Reset go into the `actions` slot, at the left of the cross.
defineProps<{ title: string }>();
const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <section class="float" :aria-label="title" @wheel.stop>
    <header>
      {{ title }}
      <span class="acts">
        <slot name="actions" />
        <button type="button" aria-label="Close" @click="emit('close')">×</button>
      </span>
    </header>
    <slot />
  </section>
</template>

<style scoped>
.float.fixed {
  flex: none;
}
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
.acts {
  display: flex;
  gap: 2px;
  align-items: center;
}
header button.link {
  width: auto;
  height: auto;
  padding: 2px 4px;
  font-size: var(--fs);
  color: var(--accent-ink);
}
header button.link:hover {
  background: var(--accent-soft);
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
