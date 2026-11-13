<script setup lang="ts">
import { computed } from "vue";
import { stepShortcut } from "../logic/keys";
import { stepInfo, viewSteps, type ViewStep } from "../logic/workflow";
import { session, status } from "../state";
import { resultOf, states } from "../steps";

// The step bar under the toolbar (docs/design.md, "Schrittleiste"): the seven steps as pills, the details of the
// recording at the right. Done and possible steps open in the left panel; nothing opens during a calculation.
const busy = computed(() => status.busy !== null);

const details = computed(() => {
  const info = session.info;
  if (!info) return "";
  // The codec comes last: it means least to a newcomer (MAC-014).
  return [session.fileName, `${info.width}×${info.height}`, `${info.frameCount} frames`, info.codec].filter(Boolean).join(" · ");
});

// The tooltip of a pill says what the step is, then what is missing (locked) or what it gave (done), then its shortcut.
function title(id: ViewStep, label: string): string {
  const state = states.value[id];
  const lines = [stepInfo(id).explanation];
  const result = resultOf(id);
  if (state.status === "locked") lines.push(state.hint);
  else if (state.status === "done" && result) lines.push(`${label} · ${result}`);
  lines.push(`Shortcut: ${stepShortcut(id)}`);
  return lines.join("\n");
}
</script>

<template>
  <nav class="stepbar" aria-label="Steps">
    <ol class="steps">
      <li v-for="(step, index) in viewSteps" :key="step.id">
        <span v-if="index > 0" class="conn" :class="{ ok: states[viewSteps[index - 1].id].status === 'done' }" aria-hidden="true" />
        <button
          type="button"
          class="pill"
          :class="[states[step.id].status, { shown: session.view === step.id, busy }]"
          :aria-current="session.view === step.id ? 'step' : undefined"
          :aria-disabled="states[step.id].status === 'locked' || busy"
          :title="title(step.id, step.label)"
          @click="states[step.id].status !== 'locked' && !busy && (session.view = step.id)"
        >
          <i>{{ states[step.id].status === "done" ? "✓" : index + 1 }}</i>{{ step.label }}
        </button>
      </li>
    </ol>
    <div v-if="details" class="meta" :title="details">{{ details }}</div>
  </nav>
</template>

<style scoped>
.stepbar {
  display: flex;
  gap: 16px;
  align-items: center;
  height: 40px;
  padding: 0 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
}
.steps {
  display: flex;
  flex: none;
  align-items: center;
  margin: 0;
  padding: 0;
  list-style: none;
}
.steps li {
  display: flex;
  align-items: center;
}
.conn {
  width: 14px;
  height: 1px;
  margin: 0 4px;
  background: var(--line-strong);
}
.conn.ok {
  background: var(--ok);
}
.pill {
  display: flex;
  gap: 6px;
  align-items: center;
  height: 28px;
  padding: 0 10px 0 4px;
  font-size: 13px;
  white-space: nowrap;
  color: var(--text);
  background: none;
  border: 0;
  border-radius: 14px;
  cursor: pointer;
}
.pill i {
  display: grid;
  place-items: center;
  width: 18px;
  height: 18px;
  font-size: 11px;
  font-style: normal;
  font-weight: 600;
  color: var(--muted);
  border: 1px solid var(--line-strong);
  border-radius: 50%;
}
.pill:hover:not(.locked, .busy, .shown) {
  background: var(--raised-h);
}
.pill.done i {
  color: var(--on-accent);
  background: var(--ok);
  border-color: var(--ok);
}
.pill.locked {
  color: var(--off);
  cursor: default;
}
.pill.locked i {
  color: var(--off);
  border-color: var(--line);
}
.pill.busy {
  cursor: default;
}
.pill.shown {
  font-weight: 600;
  color: var(--accent-ink);
  background: var(--accent-soft);
}
.pill.shown:not(.done) i {
  color: var(--on-accent);
  background: var(--accent);
  border-color: var(--accent);
}
.meta {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
