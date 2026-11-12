<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { chooseFiles } from "../recording";
import { againDiscards, stepAfter, stepBefore, stepInfo, stepNumber, type ViewStep } from "../logic/workflow";
import { session, status } from "../state";
import { advancedOpen, nextOpen, progress, resultOf, states, toggleAdvanced } from "../steps";
import SaveImageMenu from "./SaveImageMenu.vue";

// The left panel for one step (docs/design.md, "Schrittpanel"): the result of the step before, the header, the key
// settings (default slot), the rest under Advanced (slot `advanced`, `more` settings) and the foot with the action.
const props = defineProps<{ id: ViewStep; more?: number }>();

const info = computed(() => stepInfo(props.id));
const busy = computed(() => status.busy !== null);
const done = computed(() => states.value[props.id].status === "done");
const isNext = computed(() => nextOpen.value === props.id);
// After Save image everything is done and Save stays open with its result.
const finished = computed(() => props.id === "save" && done.value && isNext.value);

const before = computed(() => stepBefore(props.id));
const after = computed(() => stepAfter(props.id));
const advanced = computed(() => advancedOpen[props.id] === true);

function run(id: ViewStep) {
  switch (id) {
    case "open":
      void chooseFiles();
      break;
    case "alignpoints":
      void runAction("setAlignpoints");
      break;
    case "align":
      void runAction("align");
      break;
    case "limit":
      void runAction("limit");
      break;
    case "stack":
      void runAction("stack");
      break;
    case "wavelets":
      void runAction("doAll");
      break;
    case "save":
      break;
  }
}
</script>

<template>
  <aside class="steppanel" :aria-label="info.label">
    <div class="scroll">
      <p v-if="isNext && before" class="prev">✓ {{ stepInfo(before).label }} · {{ resultOf(before) }}</p>
      <header class="head">
        <div class="count">Step {{ stepNumber(id) }} of 7</div>
        <h2>{{ info.label }}</h2>
        <p>{{ info.sentence }}</p>
      </header>

      <slot />

      <template v-if="more">
        <button type="button" class="adv" :aria-expanded="advanced" @click="toggleAdvanced(id)">
          <span>{{ advanced ? "▾" : "▸" }} Advanced</span><span class="n">{{ more }} more settings</span>
        </button>
        <div v-if="advanced" class="advanced"><slot name="advanced" /></div>
      </template>
    </div>

    <footer class="foot">
      <!-- After Save image: the file name, then the way to the next recording -->
      <template v-if="finished">
        <p class="result">✓ {{ resultOf(id) }}</p>
        <button type="button" class="btn next wide" :disabled="busy" @click="chooseFiles">Open another recording</button>
      </template>

      <!-- A done step opened again: its result, the action as a standard button, and the way on -->
      <template v-else-if="done">
        <p class="result">✓ {{ resultOf(id) }}</p>
        <div class="line">
          <button v-if="id === 'wavelets'" type="button" class="btn" :disabled="busy" @click="runAction('process')">Process</button>
          <button type="button" class="btn wide" :disabled="busy" @click="run(id)">{{ info.action }} again</button>
        </div>
        <p v-if="againDiscards(id, progress)" class="then">Running it again discards the steps after it.</p>
        <SaveImageMenu v-if="id === 'stack'" source="stack" :disabled="busy" />
        <button type="button" class="btn next wide" :disabled="busy" @click="session.view = nextOpen">
          Continue: {{ stepInfo(nextOpen).label }} ›
        </button>
      </template>

      <!-- The open step: its action filled, and what comes then -->
      <template v-else>
        <div class="line">
          <button v-if="id === 'wavelets'" type="button" class="btn" :disabled="busy" @click="runAction('process')">Process</button>
          <SaveImageMenu v-if="id === 'save'" source="wavelet" filled :disabled="busy" />
          <button v-else type="button" class="btn next wide" :disabled="busy" @click="run(id)">{{ info.action }}</button>
        </div>
        <p v-if="after" class="then">Then: {{ stepInfo(after).label }}</p>
      </template>
    </footer>
  </aside>
</template>

<style scoped>
.steppanel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--panel);
  border-right: 1px solid var(--line);
}
.scroll {
  flex: 1;
  min-height: 0;
  padding: 12px 12px 12px;
  overflow: auto;
}
/* a label never wraps: the panel is fixed at 272 px and the rows are built for one line */
.scroll :deep(.row) {
  white-space: nowrap;
}
.prev,
.result {
  margin: 0 0 10px;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--ok);
}
.result {
  margin-bottom: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.head {
  margin-bottom: 10px;
}
.count {
  font-size: var(--fs-s);
  color: var(--muted);
}
.head h2 {
  margin: 2px 0 4px;
  font-size: 15px;
  font-weight: 600;
}
.head p {
  margin: 0;
  font-size: var(--fs-s);
  line-height: 17px;
  color: var(--muted);
}
.adv {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  min-height: 29px;
  margin-top: 10px;
  padding: 0;
  font-weight: 500;
  text-align: left;
  background: none;
  border: 0;
  border-top: 1px solid var(--line);
  cursor: pointer;
}
.adv .n {
  font-size: var(--fs-s);
  font-weight: 400;
  color: var(--muted);
}
.adv:hover span:first-child {
  color: var(--accent-ink);
}
.foot {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  border-top: 1px solid var(--line);
}
.foot .result {
  margin: 0;
}
.line {
  display: flex;
  gap: 6px;
}
.wide {
  flex: 1;
  width: 100%;
}
.line > :deep(.mw) {
  flex: 1;
}
.then {
  margin: 0;
  font-size: var(--fs-s);
  color: var(--muted);
}
</style>
