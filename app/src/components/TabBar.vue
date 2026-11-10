<script setup lang="ts">
import { computed } from "vue";
import { tabLocked, tabs } from "../logic/workflow";
import { session } from "../state";

// The three tabs as numbered steps; the details of the recording sit at the right (the Memory display of the
// original needs a backend command and is not built).
const details = computed(() => {
  const info = session.info;
  if (!info) return null;
  return [info.codec, `${info.width}×${info.height}`, `${info.frameCount} frames`].filter(Boolean).join(" · ");
});
</script>

<template>
  <div class="tabrow">
    <div class="tabs" role="tablist">
      <template v-for="(tab, index) in tabs" :key="tab.id">
        <span v-if="index > 0" class="sep" aria-hidden="true">›</span>
        <button
          type="button"
          role="tab"
          class="tab"
          :aria-selected="session.tab === tab.id"
          :disabled="tabLocked(session.step, tab.id) && session.tab !== tab.id"
          :title="tabLocked(session.step, tab.id) ? tab.hint : undefined"
          @click="session.tab = tab.id"
        >
          <i>{{ index + 1 }}</i
          >{{ tab.label }}
        </button>
      </template>
    </div>
    <div v-if="details" class="meta">
      <b>{{ session.fileName }}</b> · {{ details }}
    </div>
  </div>
</template>

<style scoped>
.tabrow {
  display: flex;
  align-items: stretch;
  padding: 0 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
}
.tabs {
  display: flex;
  align-items: stretch;
}
.tab {
  display: flex;
  gap: 8px;
  align-items: center;
  height: 36px;
  padding: 0 14px 0 6px;
  font-weight: 600;
  color: var(--muted);
  background: none;
  border: 0;
  border-bottom: 2px solid transparent;
  cursor: pointer;
}
.tab i {
  display: grid;
  place-items: center;
  width: 18px;
  height: 18px;
  font-size: 11px;
  font-style: normal;
  background: var(--raised);
  border-radius: 50%;
}
.tab[aria-selected="true"] {
  color: var(--text);
  border-bottom-color: var(--accent);
}
.tab[aria-selected="true"] i {
  color: var(--on-accent);
  background: var(--accent);
}
.tab:disabled {
  color: var(--off);
  cursor: default;
}
.sep {
  align-self: center;
  margin: 0 2px;
  color: var(--off);
}
.meta {
  display: flex;
  gap: 18px;
  align-items: center;
  margin-left: auto;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
}
.meta b {
  font-weight: 600;
  color: var(--text);
}
</style>
