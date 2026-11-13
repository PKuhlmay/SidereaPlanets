<script setup lang="ts">
import NumberField from "./NumberField.vue";

// A row of the cards Denoise/Deringing and Masking: label and field above a slider, whole numbers.
defineProps<{ title: string; label: string; min: number; max: number; help: string }>();
const value = defineModel<number>({ required: true });
</script>

<template>
  <div class="refine" :title="help">
    <label class="row">{{ title }} <NumberField v-model="value" :min="min" :max="max" :width="56" :label="label" /></label>
    <input v-model.number="value" type="range" :min="min" :max="max" :aria-label="title" />
  </div>
</template>

<style scoped>
.refine + .refine {
  margin-top: 4px;
}
</style>
