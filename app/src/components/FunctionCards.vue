<script setup lang="ts">
import { computed } from "vue";
import { isToneCard } from "../logic/colour";
import { session, settings } from "../state";
import ColorMixingCard from "./ColorMixingCard.vue";
import CropCard from "./CropCard.vue";
import DenoiseCard from "./DenoiseCard.vue";
import FlipRotateCard from "./FlipRotateCard.vue";
import ResizeCard from "./ResizeCard.vue";
import GammaCard from "./GammaCard.vue";
import MaskCard from "./MaskCard.vue";
import HistogramCard from "./HistogramCard.vue";
import RgbAlignCard from "./RgbAlignCard.vue";
import RgbBalanceCard from "./RgbBalanceCard.vue";

// The cards behind the Adjustments buttons of the Wavelets step, in the order the buttons were pressed. The crop tool, while it is open,
// comes first in the steps Wavelets and Save, so its buttons are always in view.
const open = computed(() => (session.view === "wavelets" ? settings.wavelet.functions.filter(isToneCard) : []));
const cropping = computed(() => session.crop.editing && (session.view === "wavelets" || session.view === "save"));
</script>

<template>
  <CropCard v-if="cropping" />
  <template v-for="name in open" :key="name">
    <HistogramCard v-if="name === 'Histogram'" />
    <GammaCard v-else-if="name === 'Gamma'" />
    <ColorMixingCard v-else-if="name === 'Color Mixing'" />
    <DenoiseCard v-else-if="name === 'Denoise/Deringing'" />
    <MaskCard v-else-if="name === 'Masking'" />
    <FlipRotateCard v-else-if="name === 'Flip and Rotate'" />
    <ResizeCard v-else-if="name === 'Resize Image'" />
    <RgbAlignCard v-else-if="name === 'RGB Align'" />
    <RgbBalanceCard v-else />
  </template>
</template>
