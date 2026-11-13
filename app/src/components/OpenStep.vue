<script setup lang="ts">
import { computed } from "vue";
import { directoryOf } from "../logic/exporting";
import { recentLabel } from "../logic/recent";
import { recordingRows, rowExplanation } from "../logic/recordingDetails";
import { openRecording, recent } from "../recording";
import { session, status } from "../state";
import StepPanel from "./StepPanel.vue";

// Step 1 Open: the loaded recording with its file info and the files opened last; a click on one opens it (like Recent
// Recordings in the toolbar).
const rows = computed(() => (session.info ? recordingRows(session.info) : []));
</script>

<template>
  <StepPanel id="open">
    <div v-if="session.info" class="loaded">
      <b :title="session.fileName">{{ session.fileName }}</b>
      <dl class="info">
        <template v-for="row in rows" :key="row.label">
          <dt :title="rowExplanation(row.label)">{{ row.label }}</dt>
          <dd>{{ row.value }}</dd>
        </template>
      </dl>
    </div>
    <div class="grp">
      <div class="gh">Recent Recordings</div>
      <p v-if="recent.list.length === 0" class="none">No recent recordings</p>
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
  gap: 6px;
  padding-bottom: 10px;
  font-variant-numeric: tabular-nums;
  border-bottom: 1px solid var(--line);
}
.loaded b {
  overflow: hidden;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}
/* label and value side by side, 12 px: the label muted, the value in the text colour */
.info {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 3px 10px;
  margin: 0;
  font-size: var(--fs-s);
}
.info dt {
  color: var(--muted);
}
.info dd {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
  user-select: text;
}
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
