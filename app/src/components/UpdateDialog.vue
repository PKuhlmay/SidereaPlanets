<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { installBlock, RELEASES_URL } from "../logic/updates";
import { about, status } from "../state";
import { updates } from "../updater";

// The dialog of a new version (PLANETS-13): what is new, then Install and Restart, Later or Skip This Version. The release notes
// are shown as plain text; they come from the release page and are never interpreted as HTML.
const state = updates.state;
const update = computed(() => state.update);
// The About dialog has the window first: a version found while it is open is shown after it closes.
const visible = computed(() => update.value !== null && !about.open);
const block = computed(() => (update.value ? installBlock(status.busy !== null, update.value.canInstall) : null));
const working = computed(() => state.installing);
const progressText = computed(() => {
  if (state.waiting) return "Waiting for the calculation to finish…";
  if (state.progress === null) return "Downloading…";
  return state.progress >= 1 ? "Installing…" : `Downloading… ${Math.round(state.progress * 100)}%`;
});

const root = ref<HTMLElement | null>(null);
const current = __APP_VERSION__;

// While the dialog is open, Escape means Later and Tab stays inside it. The listener is on the window and runs first, so the
// shortcuts behind the dialog do not see these keys.
function onKey(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopPropagation();
    updates.later();
    return;
  }
  if (event.key !== "Tab" || !root.value) return;
  const buttons = [...root.value.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")];
  const first = buttons[0];
  const last = buttons[buttons.length - 1];
  if (!first) return;
  const outside = !root.value.contains(document.activeElement);
  if (outside || (event.shiftKey && document.activeElement === first)) {
    event.preventDefault();
    (event.shiftKey ? last : first).focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

// The focus goes to the first button that works when the dialog opens: Install and Restart, else Later.
watch(visible, async (shown) => {
  if (!shown) {
    window.removeEventListener("keydown", onKey, true);
    return;
  }
  window.addEventListener("keydown", onKey, true);
  await nextTick();
  const buttons = [...(root.value?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
  (buttons.find((button) => button.classList.contains("next")) ?? buttons[0])?.focus();
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));
</script>

<template>
  <div v-if="update && visible" class="backdrop">
    <section ref="root" class="dialog" role="dialog" aria-modal="true" aria-labelledby="update-title">
      <h2 id="update-title">Siderea Planets {{ update.version }} is available</h2>
      <p class="sub">You have version {{ current }}.</p>

      <div class="cap">What’s new</div>
      <div class="notes" tabindex="0" aria-label="Release notes">
        <template v-if="update.notes">{{ update.notes }}</template>
        <span v-else class="none">This release has no notes.</span>
      </div>

      <p v-if="!update.canInstall" class="hint">
        This installation cannot update itself. Download the new version from {{ RELEASES_URL }}.
      </p>
      <p v-if="state.error" class="hint">{{ state.error }}</p>

      <div v-if="working" class="prog" role="status">
        <span>{{ progressText }}</span>
        <div v-if="!state.waiting" class="bar"><i :style="{ width: `${(state.progress ?? 0) * 100}%` }" /></div>
      </div>

      <div class="actions">
        <button type="button" class="btn ghost skip" :disabled="working" title="Do not offer this version again; a newer one will still be offered" @click="updates.skip()">
          Skip This Version
        </button>
        <button type="button" class="btn" :disabled="working" title="Ask again at the next check" @click="updates.later()">Later</button>
        <button
          type="button"
          class="btn next"
          :disabled="working || block !== null"
          :title="block ?? 'Download the update, install it and start Siderea Planets again'"
          @click="updates.install()"
        >
          Install and Restart
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(0, 0, 0, 0.5);
}
.dialog {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 480px;
  max-width: 100%;
  max-height: 100%;
  padding: 16px;
  background: var(--panel);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
}
h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.sub {
  margin: 0;
  color: var(--muted);
}
.cap {
  margin-top: 4px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.notes {
  min-height: 48px;
  max-height: 220px;
  padding: 8px 10px;
  overflow-y: auto;
  line-height: 18px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: var(--field);
  border: 1px solid var(--line);
  border-radius: var(--r);
}
.none {
  color: var(--muted);
}
.hint {
  margin: 0;
  padding: 8px 10px 8px 13px;
  font-size: var(--fs-s);
  line-height: 17px;
  overflow-wrap: anywhere;
  background: linear-gradient(to right, var(--accent) 3px, var(--accent-soft) 3px);
  border-radius: var(--r);
}
.prog {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
}
.bar {
  height: 6px;
  overflow: hidden;
  background: var(--raised);
  border-radius: 3px;
}
.bar i {
  display: block;
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
}
.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 8px;
}
/* a locked Install and Restart must not keep the amber fill */
.btn.next:disabled {
  color: var(--off);
  background: transparent;
  border-color: var(--line);
}
.skip {
  margin-right: auto;
}
</style>
