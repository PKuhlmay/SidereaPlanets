<script setup lang="ts">
import { formatDuration } from "../logic/workflow";
import { status } from "../state";

// Five fields: progress, current action, result, position under the pointer (Wavelets and Save steps), time.
</script>

<template>
  <footer class="statusbar">
    <div class="prog">
      <div class="bar"><i :style="{ width: `${status.progress * 100}%` }" /></div>
      <span>{{ Math.round(status.progress * 100) }}%</span>
    </div>
    <div class="msg">{{ status.action }}</div>
    <div class="msg" :title="status.result">{{ status.result }}</div>
    <div>{{ status.position }}</div>
    <div>Time used {{ formatDuration(status.timeUsed) }} · Time left {{ formatDuration(status.timeLeft) }}</div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: grid;
  grid-template-columns: 210px 1fr 1fr auto auto;
  align-items: center;
  height: 28px;
  font-size: var(--fs-s);
  font-variant-numeric: tabular-nums;
  color: var(--muted);
  background: var(--panel);
  border-top: 1px solid var(--line);
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
