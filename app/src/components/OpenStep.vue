<script setup lang="ts">
import { computed } from "vue";
import { directoryOf } from "../logic/exporting";
import { recentLabel } from "../logic/recent";
import { openRecording, recent } from "../recording";
import { session, status } from "../state";
import StepPanel from "./StepPanel.vue";

// Step 1 Open: the loaded recording and the files opened last; a click on one opens it (like MRU in the toolbar).
const details = computed(() => {
  const info = session.info;
  if (!info) return "";
  return [info.codec, `${info.width}×${info.height}`, `${info.frameCount} frames`].filter(Boolean).join(" · ");
});
</script>

<template>
  <StepPanel id="open">
    <div v-if="session.info" class="loaded">
      <b>{{ session.fileName }}</b>
      <span>{{ details }}</span>
    </div>
    <div class="grp">
      <div class="gh">Recent</div>
      <p v-if="recent.list.length === 0" class="none">No recent files</p>
      <button
        v-for="paths in recent.list"
        :key="paths.join('|')"
        type="button"
        class="file"
        :title="paths.join(', ')"
        :disabled="status.busy !== null"
        @click="openRecording(paths)"
      >
        <span>{{ recentLabel(paths) }}</span>
        <small>{{ directoryOf(paths[0]) }}</small>
      </button>
    </div>
  </StepPanel>
</template>

<style scoped>
.loaded {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-bottom: 8px;
  font-variant-numeric: tabular-nums;
  border-bottom: 1px solid var(--line);
}
.loaded b {
  font-weight: 600;
}
.loaded span,
.none,
.file small {
  font-size: var(--fs-s);
  color: var(--muted);
}
.none {
  margin: 0;
}
.file {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 5px 6px;
  text-align: left;
  background: none;
  border: 0;
  border-radius: var(--r-s);
  cursor: pointer;
}
.file:hover:not(:disabled) {
  background: var(--raised-h);
}
.file:disabled {
  color: var(--off);
  cursor: default;
}
.file span,
.file small {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
