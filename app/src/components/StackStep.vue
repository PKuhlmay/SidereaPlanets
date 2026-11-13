<script setup lang="ts">
import { runAction } from "../actions";
import { codecs } from "../logic/settings";
import { atLeast } from "../logic/workflow";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 5 Stack: the key Stacking options; the others and Create AVI under Advanced. Options the backend ignores
// (Output size other than Default, Stretch histogram, De-rotate, Resampling, Drizzling, and in Create AVI Codec,
// Compression quality, the frame sizes other than Original size, Use prefilters and Compressed) are locked and marked
// "Not available yet"; their values stay in `settings.stack` (logic/stack.ts), where nothing reads them.
const stack = settings.stack;
</script>

<template>
  <StepPanel id="stack" :more="14">
    <div class="sub first" title="Size of the stacked image.">Output size</div>
    <label class="chk"><input v-model="stack.stackSize" type="radio" name="stacksize" value="default" />Default</label>
    <div class="inl" title="Not available yet. Only the area that every frame covers.">
      <label class="chk"><input type="radio" name="stacksize" value="minimized" disabled />Smallest common area</label>
      <span class="factor">Not available yet</span>
    </div>
    <div class="inl" title="Not available yet. Everything that any frame covers.">
      <label class="chk"><input type="radio" name="stacksize" value="maximized" disabled />Largest area</label>
      <span class="factor">Not available yet</span>
    </div>
    <div class="inl" title="Not available yet. Only the region of interest.">
      <label class="chk"><input type="radio" name="stacksize" value="roi" disabled />Region</label>
      <span class="factor">Not available yet</span>
    </div>
    <label
      class="row gap"
      title="How many of the nearest alignpoints decide how a pixel is shifted: more is smoother, fewer is more local."
    >
      Use nearest points <NumberField v-model="stack.nearestPoints" />
    </label>
    <label class="chk" title="Shifts each part of the image by its own movement; off moves every frame as a whole."><input v-model="stack.correctGeometry" type="checkbox" />Correct geometry</label>

    <template #advanced>
      <div class="grp">
        <div class="gh">Stacking Options</div>
        <div class="col">
          <label class="chk" title="Levels the brightness of the frames before stacking; helps when clouds pass by."><input v-model="stack.normalize" type="checkbox" />Even out frame brightness</label>
          <div class="inl" title="Not available yet. Spreads the brightness over the full range after stacking.">
            <label class="chk"><input type="checkbox" :checked="false" disabled />Stretch histogram after stacking</label>
            <span class="factor">Not available yet</span>
          </div>
          <div class="inl" title="Not available yet. Turns the frames back against the rotation of the planet.">
            <label class="chk"><input type="checkbox" :checked="false" disabled />De-rotate images</label>
            <span class="factor">Not available yet</span>
          </div>
        </div>
        <label class="row" title="Pixels at the edge of each frame that are left out of the stack, in pixels.">
          Ignore edge pixels (px) <NumberField v-model="stack.edgePixels" />
        </label>
        <!-- Locked: the stack does not enlarge the result yet (docs/stacking.md). The method and factor fields appear
             with the check box in the design, so they stay out until the check box can be switched on. -->
        <div class="inl" title="Not available yet. Enlarges the result.">
          <label class="chk"><input type="checkbox" :checked="false" disabled />Resampling</label>
          <span class="factor">Not available yet</span>
        </div>
        <div class="inl" title="Not available yet. Enlarges the result by combining shifted frames.">
          <label class="chk"><input type="checkbox" :checked="false" disabled />Drizzling</label>
          <span class="factor">Not available yet</span>
        </div>
      </div>

      <div class="grp">
        <div class="gh" title="Saves the aligned frames as a video, for example for animations or other software.">Create AVI</div>
        <!-- Codec and Compression quality are locked: the video is always written uncompressed (docs/formats.md, "Create AVI"). -->
        <label class="row" title="Not available yet. The video is always uncompressed.">
          Codec
          <select v-model="stack.codec" class="codec" disabled>
            <option v-for="c in codecs" :key="c" :value="c">{{ c }}</option>
          </select>
        </label>
        <label class="row" title="Not available yet. The video is always uncompressed.">
          Compression quality <NumberField v-model="stack.compressQuality" :step="100" :width="84" disabled />
        </label>
        <label class="row">Frame rate (fps) <NumberField v-model="stack.framerate" :width="84" /></label>
        <div class="sub">Aligned video</div>
        <button
          type="button"
          class="btn"
          :disabled="!atLeast(session.step, 'aligned')"
          title="Saves the kept frames, aligned, as an AVI video."
          @click="runAction('saveAligned')"
        >
          Save Aligned Video…
        </button>
        <div class="sub">Frame size</div>
        <!-- Only the original size is written (docs/formats.md, "Create AVI"), so it is always the one that is on. -->
        <div class="col">
          <label class="chk"><input type="radio" name="alignedsize" value="original" checked />Original size</label>
          <div class="inl" title="Not available yet">
            <label class="chk"><input type="radio" name="alignedsize" value="minimum" disabled />Smallest common area</label>
            <span class="factor">Not available yet</span>
          </div>
          <div class="inl" title="Not available yet">
            <label class="chk"><input type="radio" name="alignedsize" value="maximum" disabled />Largest area</label>
            <span class="factor">Not available yet</span>
          </div>
          <div class="inl" title="Not available yet">
            <label class="chk"><input type="radio" name="alignedsize" value="roi" disabled />Region</label>
            <span class="factor">Not available yet</span>
          </div>
        </div>
        <div class="grid2">
          <label class="chk" title="Puts the sharpest frames first in the video."><input v-model="stack.sortOnQuality" type="checkbox" />Sort by quality</label>
          <label class="chk" title="Not available yet"><input type="checkbox" :checked="false" disabled />Use prefilters</label>
        </div>
        <div class="sub">Averaged video</div>
        <button
          type="button"
          class="btn"
          :disabled="!atLeast(session.step, 'aligned')"
          title="Saves a video in which each frame is the average of several aligned frames in a row; the number is set below."
          @click="runAction('saveIntegrated')"
        >
          Save Averaged Video…
        </button>
        <div class="col">
          <label class="chk" title="Each video frame is the average of the next n aligned frames."><input v-model="stack.integratedMethod" type="radio" name="integrated" value="moving" />Moving average</label>
          <div class="inl" title="Not available yet">
            <label class="chk"><input type="radio" name="integrated" value="compressed" disabled />Compressed</label>
            <span class="factor">Not available yet</span>
          </div>
        </div>
        <label class="row start">Average of <NumberField v-model="stack.averageFrames" :width="56" label="Average of n frames" /> frames</label>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.first {
  margin-top: 0;
}
.gap {
  margin-top: 6px;
}
.inl {
  display: flex;
  flex-wrap: wrap;
  gap: 0 6px;
  align-items: center;
  min-height: 29px;
}
.factor {
  margin-left: auto;
  font-size: var(--fs-s);
  color: var(--muted);
}
.codec {
  width: 120px;
}
.row:has(select:disabled) {
  color: var(--off);
}
.codec:disabled {
  opacity: 0.45;
}
.start {
  justify-content: flex-start;
}
</style>
