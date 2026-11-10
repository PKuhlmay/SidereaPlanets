<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import type { ResampleMethod } from "../logic/settings";
import { atLeast, stepButtons, type StepId } from "../logic/workflow";
import { session, settings, status } from "../state";
import NumberField from "./NumberField.vue";

// The action row under the tabs: step buttons at the left (above the left panel), the header check boxes next to them.
const busy = computed(() => status.busy !== null);
const steps = computed(() => stepButtons(session.step, busy.value));
const stacked = computed(() => atLeast(session.step, "stacked"));
const methods: { value: ResampleMethod; label: string }[] = [
  { value: "bell", label: "Bell" },
  { value: "bspline", label: "Bspline" },
  { value: "lanczos", label: "Lanczos" },
  { value: "mitchell", label: "Mitchell" },
];

const classes = (id: StepId) => ({ next: steps.value[id].next, done: steps.value[id].done });
</script>

<template>
  <div class="actions">
    <template v-if="session.tab === 'align'">
      <div class="flow">
        <button type="button" class="btn" :class="classes('setAlignpoints')" :disabled="steps.setAlignpoints.disabled" @click="runAction('setAlignpoints')">Set Alignpoints</button>
        <button type="button" class="btn" :class="classes('align')" :disabled="steps.align.disabled" @click="runAction('align')">Align</button>
        <button type="button" class="btn" :class="classes('limit')" :disabled="steps.limit.disabled" @click="runAction('limit')">Limit</button>
      </div>
      <div class="opts">
        <label class="chk"><input v-model="settings.colour" type="checkbox" />Colour</label>
        <label class="chk"><input v-model="settings.lrgb" type="checkbox" />LRGB</label>
        <label class="chk"><input v-model="settings.align.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="chk"><input v-model="settings.align.showRoi" type="checkbox" :disabled="!session.roi" />Show ROI</label>
        <label class="chk"><input v-model="settings.align.showAlignpoints" type="checkbox" />Show Alignpoints</label>
        <label class="chk"><input v-model="settings.align.showAligndata" type="checkbox" />Show Aligndata</label>
        <label class="chk"><input v-model="settings.align.showFramelist" type="checkbox" />Show Framelist</label>
        <label class="chk"><input v-model="settings.align.showRegistrationgraph" type="checkbox" />Show Registrationgraph</label>
        <label class="chk"><input v-model="settings.align.showPrefilter" type="checkbox" />Show Prefilter</label>
      </div>
    </template>

    <template v-else-if="session.tab === 'stack'">
      <div class="flow">
        <button type="button" class="btn" :class="classes('stack')" :disabled="steps.stack.disabled" @click="runAction('stack')">Stack</button>
        <button type="button" class="btn" :disabled="steps.saveImage.disabled" @click="runAction('saveImage')">Save image</button>
      </div>
      <div class="opts">
        <label class="chk"><input v-model="settings.colour" type="checkbox" />Colour</label>
        <label class="chk"><input v-model="settings.lrgb" type="checkbox" />LRGB</label>
        <label class="chk"><input v-model="settings.stack.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="chk"><input v-model="settings.stack.showAlignSections" type="checkbox" />Show AlignSections</label>
        <label class="chk"><input v-model="settings.stack.showStackgraph" type="checkbox" />Show Stackgraph</label>
        <label class="chk"><input v-model="settings.stack.showStacklist" type="checkbox" />Show Stacklist</label>
        <label class="chk"><input v-model="settings.stack.resampling" type="checkbox" />Resampling</label>
        <label class="chk"><input v-model="settings.stack.drizzling" type="checkbox" />Drizzling</label>
        <!-- The extra fields appear next to their check box, in the same row -->
        <div class="inl">
          <template v-if="settings.stack.resampling">
            <select v-model="settings.stack.resampleMethod" aria-label="Resampling method">
              <option v-for="m in methods" :key="m.value" :value="m.value">{{ m.label }}</option>
            </select>
            Factor <NumberField v-model="settings.stack.resampleFactor" :decimals="1" :step="0.1" :width="56" label="Resampling factor" />
          </template>
        </div>
        <div class="inl">
          <template v-if="settings.stack.drizzling">
            Factor <NumberField v-model="settings.stack.drizzleFactor" :decimals="1" :step="0.1" :width="56" label="Drizzling factor" />
          </template>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="flow">
        <button type="button" class="btn" :disabled="busy || !stacked" @click="runAction('process')">Process</button>
        <button type="button" class="btn" :class="{ next: !busy && stacked }" :disabled="busy || !stacked" @click="runAction('doAll')">Do All</button>
        <button type="button" class="btn" :disabled="steps.saveImage.disabled" @click="runAction('saveImage')">Save image</button>
        <span class="gap" />
        <button type="button" class="btn" :disabled="busy || !stacked" @click="runAction('realign')">Realign with Processed</button>
        <button type="button" class="btn" disabled>Stack Again</button>
      </div>
      <div class="opts">
        <label class="chk"><input v-model="settings.wavelet.showFullImage" type="checkbox" />Show Full Image</label>
        <label class="chk"><input v-model="settings.wavelet.showAlignpoints" type="checkbox" />Show AlignPoints</label>
        <label class="chk"><input v-model="settings.wavelet.showProcessingArea" type="checkbox" />Show Processing Area</label>
      </div>
      <button
        type="button"
        class="btn fx"
        :aria-pressed="settings.wavelet.functionsOpen"
        title="Show or hide the Functions panel"
        @click="settings.wavelet.functionsOpen = !settings.wavelet.functionsOpen"
      >
        Functions »
      </button>
    </template>
  </div>
</template>

<style scoped>
.actions {
  display: flex;
  gap: 20px;
  align-items: center;
  min-height: 62px;
  padding: 6px 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
}
.flow {
  display: flex;
  gap: 6px;
  align-items: center;
  min-width: calc(var(--side) - 12px);
}
.gap {
  width: 10px;
}
.opts {
  display: grid;
  grid-auto-flow: column;
  grid-template-rows: repeat(2, 24px);
  column-gap: 24px;
  row-gap: 1px;
  justify-content: start;
  align-items: center;
}
.inl {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: var(--fs-s);
  color: var(--muted);
}
.fx {
  margin-left: auto;
}
</style>
