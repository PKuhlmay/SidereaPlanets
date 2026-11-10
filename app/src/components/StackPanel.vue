<script setup lang="ts">
import { runAction } from "../actions";
import { codecs } from "../logic/settings";
import { atLeast } from "../logic/workflow";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";

// Left panel of the Stack tab: Stacking options and Create AVI. The values are collected here; the Stack ticket
// (RK-006) reads them from `settings.stack`.
const stack = settings.stack;
</script>

<template>
  <fieldset class="side" :disabled="session.step === 'empty'" aria-label="Stacking">
    <div class="grp">
      <div class="gh">Stacking options</div>
      <div class="sub first">StackSize</div>
      <div class="grid2">
        <label class="chk"><input v-model="stack.stackSize" type="radio" name="stacksize" value="default" />Default</label>
        <label class="chk"><input v-model="stack.stackSize" type="radio" name="stacksize" value="minimized" />Minimized</label>
        <label class="chk"><input v-model="stack.stackSize" type="radio" name="stacksize" value="maximized" />Maximized</label>
        <label class="chk"><input v-model="stack.stackSize" type="radio" name="stacksize" value="roi" />R.of Interest</label>
      </div>
      <label class="row gap-top">Use Nearestby Alignpoints <NumberField v-model="stack.nearestPoints" /></label>
      <div class="col">
        <label class="chk"><input v-model="stack.normalize" type="checkbox" />Normalization of frame intensity</label>
        <label class="chk"><input v-model="stack.correctGeometry" type="checkbox" />Correct geometry</label>
        <label class="chk"><input v-model="stack.stretchHistogram" type="checkbox" />Stretch histogram after stacking</label>
        <label class="chk"><input v-model="stack.derotate" type="checkbox" />De-rotate images</label>
      </div>
      <label class="row"><span>Do not stack pixels<br />at the frameedge</span> <NumberField v-model="stack.edgePixels" /></label>
    </div>

    <div class="grp">
      <div class="gh">Create AVI</div>
      <label class="row">
        Codec
        <select v-model="stack.codec" class="codec">
          <option v-for="c in codecs" :key="c" :value="c">{{ c }}</option>
        </select>
      </label>
      <label class="row">Compress Qual <NumberField v-model="stack.compressQuality" :step="100" :width="84" /></label>
      <label class="row">Framerate <NumberField v-model="stack.framerate" :width="84" /></label>
      <div class="sub">Create aligned Sequence</div>
      <div class="two save">
        <button type="button" class="btn" :disabled="!atLeast(session.step, 'aligned')" @click="runAction('saveAligned')">Save Aligned</button>
        <div class="col">
          <label class="chk"><input v-model="stack.alignedSize" type="radio" name="alignedsize" value="original" />Original image size</label>
          <label class="chk"><input v-model="stack.alignedSize" type="radio" name="alignedsize" value="minimum" />Minimum area</label>
          <label class="chk"><input v-model="stack.alignedSize" type="radio" name="alignedsize" value="maximum" />Maximum area</label>
          <label class="chk"><input v-model="stack.alignedSize" type="radio" name="alignedsize" value="roi" />Region of Interest</label>
        </div>
      </div>
      <div class="grid2">
        <label class="chk"><input v-model="stack.sortOnQuality" type="checkbox" />Sort On Quality</label>
        <label class="chk"><input v-model="stack.usePrefilters" type="checkbox" />Use Prefilters</label>
      </div>
      <div class="sub">Create integrated sequence</div>
      <div class="two save">
        <button type="button" class="btn" :disabled="!atLeast(session.step, 'aligned')" @click="runAction('saveIntegrated')">Save Integrated</button>
        <div class="col">
          <label class="chk"><input v-model="stack.integratedMethod" type="radio" name="integrated" value="moving" />Moving average</label>
          <label class="chk"><input type="radio" name="integrated" value="compressed" disabled />Compressed</label>
        </div>
      </div>
      <label class="row start">Average of <NumberField v-model="stack.averageFrames" :width="56" /> Frames</label>
    </div>
  </fieldset>
</template>

<style scoped>
.first {
  margin-top: 0;
}
.gap-top {
  margin-top: 6px;
}
.codec {
  width: 120px;
}
.save {
  grid-template-columns: auto 1fr;
}
.start {
  justify-content: flex-start;
}
</style>
