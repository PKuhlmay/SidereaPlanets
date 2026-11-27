<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { CAMERA_ANGLE, derotationSummary, localInputValue, parseLocalInput, PLANETS, utcText } from "../logic/derotation";
import { codecs } from "../logic/settings";
import { atLeast } from "../logic/workflow";
import { session, settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 5 Stack: the key Stacking options; the others and Create AVI under Advanced. Options the backend ignores
// (Output size other than Default, Stretch histogram, Resampling, Drizzling, and in Create AVI Codec,
// Compression quality, the frame sizes other than Source size, Use prefilters and Compressed) are locked and marked
// "Not available yet"; their values stay in `settings.stack` (logic/stack.ts), where nothing reads them.
// De-rotate images opens its settings below the check box; derotation.ts asks the core for the line at the bottom.
const stack = settings.stack;

const hasTimestamps = computed(() => session.info?.hasTimestamps ?? false);
const startTimeIsGuess = computed(() => session.info?.startTimeFromFileDate ?? false);
const summary = computed(() => (stack.derotate && session.derotation ? derotationSummary(session.derotation) : null));

function onStartTime(event: Event) {
  const value = parseLocalInput((event.target as HTMLInputElement).value);
  if (value !== null) stack.startTime = value;
}
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
          <label
            class="chk"
            title="Turns every frame back to the middle of the recording by the rotation of the planet, so that details at the edge of the disc stay sharp. Needs the date and time of the recording."
          ><input v-model="stack.derotate" type="checkbox" />De-rotate images</label>
          <div v-if="stack.derotate" class="derot">
            <div class="label">Planet</div>
            <div class="seg">
              <button
                v-for="p in PLANETS"
                :key="p.value"
                type="button"
                :aria-pressed="stack.planet === p.value"
                title="The planet in the recording: it decides how fast it turns and how flattened it is."
                @click="stack.planet = p.value"
              >
                {{ p.title }}
              </button>
            </div>
            <div v-if="!hasTimestamps" class="starttime">
              <label class="row" title="When the first frame was taken, in the time zone of this computer. The file has no time for its frames.">
                Start time
                <input class="when" type="datetime-local" :value="localInputValue(stack.startTime)" @change="onStartTime" />
              </label>
              <div v-if="stack.startTime !== null" class="note">{{ utcText(stack.startTime) }}{{ startTimeIsGuess ? " · taken from the file date, check it" : "" }}</div>
              <div v-else class="note">Enter the time of the first frame</div>
            </div>
            <label
              class="row"
              title="How far the camera was turned, in degrees counter-clockwise on the screen. 0 means celestial north is up in the picture."
            >
              Camera angle (°)
              <NumberField v-model="stack.cameraAngle" :decimals="1" :min="CAMERA_ANGLE.min" :max="CAMERA_ANGLE.max" label="Camera angle" />
            </label>
            <label class="chk" title="The picture is a mirror image, as a single mirror in the light path makes it: east is on the right when north is up."><input v-model="stack.mirrored" type="checkbox" />Mirrored image</label>
            <div v-if="summary" class="summary">
              <div :class="summary.isProblem ? 'problem' : 'line'">{{ summary.text }}</div>
              <div v-if="summary.hint" class="note">{{ summary.hint }}</div>
            </div>
            <div v-if="hasTimestamps" class="note">The file has a time for every frame.</div>
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
        <!-- Only the size of the source frames is written (docs/formats.md, "Create AVI"), so it is always the one that is on. -->
        <div class="col">
          <label class="chk"><input type="radio" name="alignedsize" value="source" checked />Source size</label>
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
/* De-rotate images: settings below the check box, indented (docs/design.md, "Derotation"). */
.derot {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-left: 18px;
  margin: 2px 0 4px;
}
.starttime {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.label,
.note {
  font-size: var(--fs-s);
  color: var(--muted);
}
.line {
  font-size: var(--fs-s);
  color: var(--text);
}
.problem {
  font-size: var(--fs-s);
  color: var(--accent-ink);
}
.summary {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.when {
  width: 150px;
  min-width: 0;
  height: var(--h-field);
  padding: 0 4px;
  background: var(--field);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-s);
}
.when:focus {
  border-color: var(--accent);
  outline: 0;
}
</style>
