<script setup lang="ts">
import { computed } from "vue";
import { cpu } from "../compute";
import { appBuildLabel } from "../logic/buildInfo";
import { byteText, cpuReading, isHigh } from "../logic/processUsage";
import { formatDuration } from "../logic/workflow";
import { resourceSettings, usage } from "../resource";
import { status } from "../state";

// Five fields: progress, current action, result, position under the pointer (Wavelets and Save steps), time. In front of them,
// while it is switched on in Settings, the CPU load and the memory of the app (SP-019).
const cores = computed(() => cpu.max);
const cpuText = computed(() => cpuReading(usage.cpuPercent, cores.value) ?? "–");
const ramText = computed(() => (usage.memoryBytes === null ? "–" : byteText(usage.memoryBytes)));
const high = computed(() => isHigh(usage.cpuPercent, cores.value));
// Version, commit and build time, as in Siderea (SP-025).
const build = appBuildLabel();
</script>

<template>
  <footer class="statusbar" :class="{ usage: resourceSettings.shown }">
    <div v-if="resourceSettings.shown" class="usage-field">
      <!-- The number of cores in brackets says what 100 % stands for. -->
      <span
        :title="`CPU load of Siderea Planets as a share of all ${cores} processor cores. 100 % means every core is fully busy.`"
        >CPU ({{ cores }}) <b :class="{ high }">{{ cpuText }}</b></span
      >
      <span class="ram" title="Memory Siderea Planets currently holds.">RAM <b>{{ ramText }}</b></span>
    </div>
    <div class="prog">
      <div class="bar"><i :style="{ width: `${status.progress * 100}%` }" /></div>
      <span>{{ Math.round(status.progress * 100) }}%</span>
    </div>
    <div class="msg">{{ status.action }}</div>
    <div class="msg" :title="status.result">{{ status.result }}</div>
    <div>{{ status.position }}</div>
    <div>Time used {{ formatDuration(status.timeUsed) }} · Time left {{ formatDuration(status.timeLeft) }}</div>
    <div class="build" title="Version of Siderea Planets and the time this build was made.">{{ build }}</div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: grid;
  grid-template-columns: 210px 1fr 1fr auto auto auto;
  align-items: center;
  height: 28px;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
  background: var(--panel);
  border-top: 1px solid var(--line);
}
/* with the readout in front: it takes what it needs, the progress keeps its width */
.statusbar.usage {
  grid-template-columns: auto 210px 1fr 1fr auto auto auto;
  container-type: inline-size;
}
.usage-field {
  display: flex;
  gap: 14px;
}
.usage-field b {
  display: inline-block;
  min-width: 50px;
  font-weight: 400;
  color: var(--text);
  text-align: right;
  white-space: pre;
}
/* a high load is marked twice: in the accent colour and bold */
.usage-field b.high {
  font-weight: 700;
  color: var(--accent-ink);
}
/* when the bar gets narrow the memory goes first; nothing wraps */
@container (max-width: 1000px) {
  .ram {
    display: none;
  }
}
.statusbar > * {
  padding: 0 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.statusbar > * + * {
  border-left: 1px solid var(--line);
}
.msg {
  color: var(--text);
}
.prog {
  display: flex;
  gap: 8px;
  align-items: center;
}
.bar {
  flex: 1;
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
.prog span {
  width: 34px;
  color: var(--text);
  text-align: right;
}
</style>
