<script setup lang="ts">
import { computed } from "vue";
import { isToneCard } from "../logic/colour";
import { session, settings } from "../state";
import ColorMixingCard from "./ColorMixingCard.vue";
import DenoiseCard from "./DenoiseCard.vue";
import GammaCard from "./GammaCard.vue";
import MaskCard from "./MaskCard.vue";
import HistogramCard from "./HistogramCard.vue";
import RgbAlignCard from "./RgbAlignCard.vue";
import RgbBalanceCard from "./RgbBalanceCard.vue";

// The cards behind the Functions buttons of the Wavelets step, in the order the buttons were pressed.
const open = computed(() => (session.view === "wavelets" ? settings.wavelet.functions.filter(isToneCard) : []));
</script>

<template>
  <template v-for="name in open" :key="name">
    <HistogramCard v-if="name === 'Histogram'" />
    <GammaCard v-else-if="name === 'Gamma'" />
    <ColorMixingCard v-else-if="name === 'Color Mixing'" />
    <DenoiseCard v-else-if="name === 'Denoise/Deringing'" />
    <MaskCard v-else-if="name === 'Masking'" />
    <RgbAlignCard v-else-if="name === 'RGB Align'" />
    <RgbBalanceCard v-else />
  </template>
</template>
