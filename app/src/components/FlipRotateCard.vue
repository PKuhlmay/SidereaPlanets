<script setup lang="ts">
import { computed } from "vue";
import { closeFunction } from "../colour";
import { editOrientation, pictureSize } from "../geometryView";
import {
  ANGLE_LIMITS,
  defaultOrientation,
  flipHorizontally,
  flipVertically,
  rotate,
  ROTATION_METHODS,
  setAngle,
  sizeLabel,
  type RotationMethod,
} from "../logic/geometry";
import { session, settings } from "../state";
import FloatCard from "./FloatCard.vue";
import NumberField from "./NumberField.vue";

// Flip and Rotate card (RK-022, after the Mac's GeometryCards.swift): mirrors the picture, turns it by 90° or by any angle, and chooses
// how the pixels in between are calculated. The image area shows the result; while the crop tool is open nothing here can change.
const o = settings.geometry.orientation;
const canEdit = computed(() => !session.crop.editing);
const isDefault = computed(() => JSON.stringify(o) === JSON.stringify(defaultOrientation()));
const size = computed(() => {
  const s = pictureSize();
  return s ? sizeLabel(s) : "";
});

const angle = computed({
  get: () => o.angle,
  set: (degrees: number) => editOrientation((v) => setAngle(v, degrees)),
});
const setMethod = (method: RotationMethod) => editOrientation((v) => (v.method = method));
const setCropToFit = (on: boolean) => editOrientation((v) => (v.cropToFit = on));
const reset = () => editOrientation((v) => Object.assign(v, defaultOrientation()));
</script>

<template>
  <FloatCard class="fixed" title="Flip and Rotate" @close="closeFunction('Flip and Rotate')">
    <template #actions>
      <button type="button" class="link" :disabled="!canEdit || isDefault" title="No flip, no turn." @click="reset">Reset</button>
    </template>
    <div class="cardbody">
      <div class="sub first">Flip</div>
      <div class="seg">
        <button
          type="button"
          :aria-pressed="o.flipHorizontal"
          :disabled="!canEdit"
          title="Mirrors the picture from left to right."
          @click="editOrientation(flipHorizontally)"
        >
          Horizontal
        </button>
        <button
          type="button"
          :aria-pressed="o.flipVertical"
          :disabled="!canEdit"
          title="Mirrors the picture from top to bottom."
          @click="editOrientation(flipVertically)"
        >
          Vertical
        </button>
      </div>
      <div class="sub">Rotate</div>
      <div class="seg">
        <button type="button" :disabled="!canEdit" title="Turns the picture 90° counterclockwise." @click="editOrientation((v) => rotate(v, -90))">
          Left 90°
        </button>
        <button type="button" :disabled="!canEdit" title="Turns the picture 90° clockwise." @click="editOrientation((v) => rotate(v, 90))">
          Right 90°
        </button>
        <button type="button" :disabled="!canEdit" title="Turns the picture upside down." @click="editOrientation((v) => rotate(v, 180))">180°</button>
      </div>
      <label class="row gap" title="Turns the picture clockwise; a negative angle turns it counterclockwise.">
        Angle (°)
        <NumberField
          v-model="angle"
          :decimals="1"
          :step="ANGLE_LIMITS.step"
          :min="ANGLE_LIMITS.min"
          :max="ANGLE_LIMITS.max"
          :width="64"
          :disabled="!canEdit"
          label="Angle in degrees"
        />
      </label>
      <input
        :value="o.angle"
        type="range"
        :min="ANGLE_LIMITS.min"
        :max="ANGLE_LIMITS.max"
        :step="ANGLE_LIMITS.step"
        :disabled="!canEdit"
        aria-label="Angle"
        @input="angle = Number(($event.target as HTMLInputElement).value)"
      />
      <div class="row" title="How the pixels are calculated when the angle is not a multiple of 90°. Bicubic is sharper, Bilinear is smoother.">
        Interpolation
        <div class="seg small">
          <button
            v-for="method in ROTATION_METHODS"
            :key="method.id"
            type="button"
            :aria-pressed="o.method === method.id"
            :disabled="!canEdit"
            @click="setMethod(method.id)"
          >
            {{ method.title }}
          </button>
        </div>
      </div>
      <label class="chk" title="Cuts away the black corners of a turned picture. Has no effect on turns by 90°, 180° or 270°.">
        <input
          type="checkbox"
          :checked="o.cropToFit"
          :disabled="!canEdit"
          @change="setCropToFit(($event.target as HTMLInputElement).checked)"
        />Crop to fit
      </label>
      <div class="row"><span class="muted">Picture</span><span aria-label="Size of the picture">{{ size }}</span></div>
      <p v-if="!canEdit" class="muted note">Finish or cancel the crop first.</p>
    </div>
  </FloatCard>
</template>

<style scoped>
.first {
  margin-top: 0;
}
.gap {
  margin-top: 6px;
}
.small {
  width: 140px;
}
.muted {
  font-size: var(--fs-s);
  color: var(--muted);
}
.note {
  margin: 2px 0 0;
}
</style>
