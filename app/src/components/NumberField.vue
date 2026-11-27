<script setup lang="ts">
import { ref, watch } from "vue";
import { clampNumber, formatNumber, parseNumber, stepNumber } from "../logic/numbers";

// Number field with arrow keys (docs/design.md, "Zahlenfeld"). Min and max only apply where a
// limit is defined; typed values are otherwise accepted as they are.
const props = withDefaults(
  defineProps<{
    modelValue: number;
    decimals?: number;
    step?: number;
    min?: number;
    max?: number;
    width?: number;
    disabled?: boolean;
    readonly?: boolean;
    label?: string;
  }>(),
  { decimals: 0, step: 1 },
);
const emit = defineEmits<{ "update:modelValue": [value: number] }>();

const text = ref(formatNumber(props.modelValue, props.decimals));
watch(
  () => [props.modelValue, props.decimals] as const,
  ([value, decimals]) => (text.value = formatNumber(value, decimals)),
);

function commit() {
  const parsed = parseNumber(text.value);
  if (parsed === null) {
    text.value = formatNumber(props.modelValue, props.decimals);
    return;
  }
  const value = clampNumber(parsed, props.min, props.max);
  text.value = formatNumber(value, props.decimals);
  if (value !== props.modelValue) emit("update:modelValue", value);
}

function nudge(direction: 1 | -1) {
  if (props.disabled || props.readonly) return;
  emit("update:modelValue", stepNumber(props.modelValue, direction, props.step, props.decimals, props.min, props.max));
}

function onKey(event: KeyboardEvent) {
  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    nudge(event.key === "ArrowUp" ? 1 : -1);
  }
}
</script>

<template>
  <span class="num" :style="width ? { width: `${width}px` } : undefined">
    <input
      v-model="text"
      type="text"
      inputmode="decimal"
      spellcheck="false"
      :disabled="disabled"
      :readonly="readonly"
      :aria-label="label"
      @change="commit"
      @keydown.enter="commit"
      @keydown="onKey"
    />
    <span class="spin">
      <button type="button" tabindex="-1" aria-label="Increase" :disabled="disabled || readonly" @click="nudge(1)" />
      <button type="button" tabindex="-1" aria-label="Decrease" :disabled="disabled || readonly" @click="nudge(-1)" />
    </span>
  </span>
</template>

<style scoped>
.num {
  display: inline-flex;
  flex: none;
  width: 66px;
  height: var(--h-field);
  overflow: hidden;
  background: var(--field);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-s);
}
.num:focus-within {
  border-color: var(--accent);
}
/* Locked: dimmed, with the text greyed as well. A label row does that for its fields, the values in Limit Setup
   stand without one. A panel that is a disabled fieldset locks its fields the same way. */
.num:has(input:disabled),
fieldset:disabled .num {
  color: var(--off);
  opacity: 0.45;
}
input[type="text"] {
  width: 100%;
  min-width: 0;
  height: auto;
  padding: 0 6px;
  font-variant-numeric: tabular-nums;
  text-align: right;
  background: none;
  border: 0;
  border-radius: 0;
  outline: 0;
}
.spin {
  display: grid;
  grid-template-rows: 1fr 1fr;
  width: 16px;
  border-left: 1px solid var(--line);
}
.spin button {
  position: relative;
  padding: 0;
  background: var(--raised);
  border: 0;
  cursor: pointer;
}
.spin button:hover:not(:disabled) {
  background: var(--raised-h);
}
.spin button + button {
  border-top: 1px solid var(--line);
}
.spin button::after {
  position: absolute;
  top: 50%;
  left: 50%;
  content: "";
  border: 3.5px solid transparent;
}
.spin button:first-child::after {
  border-bottom-color: var(--muted);
  transform: translate(-50%, -75%);
}
.spin button:last-child::after {
  border-top-color: var(--muted);
  transform: translate(-50%, -25%);
}
.spin button:disabled {
  cursor: default;
}
</style>
