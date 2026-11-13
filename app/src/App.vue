<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AlignpointsStep from "./components/AlignpointsStep.vue";
import AlignStep from "./components/AlignStep.vue";
import FloatingCards from "./components/FloatingCards.vue";
import FunctionCards from "./components/FunctionCards.vue";
import FrameBar from "./components/FrameBar.vue";
import FunctionsPanel from "./components/FunctionsPanel.vue";
import ImageOverlays from "./components/ImageOverlays.vue";
import ImageTopBar from "./components/ImageTopBar.vue";
import ImageViewer from "./components/ImageViewer.vue";
import LimitStep from "./components/LimitStep.vue";
import OpenStep from "./components/OpenStep.vue";
import SaveStep from "./components/SaveStep.vue";
import StackStep from "./components/StackStep.vue";
import StatusBar from "./components/StatusBar.vue";
import StepBar from "./components/StepBar.vue";
import Toolbar from "./components/Toolbar.vue";
import WaveletsStep from "./components/WaveletsStep.vue";
import type { Rect } from "./logic/align";
import { startColour } from "./colour";
import { openDroppedFiles } from "./recording";
import { startResourceMonitor } from "./resource";
import { handleShortcut } from "./shortcuts";
import { pickToneImage } from "./logic/colour";
import { imageCaption, imageKind, pickImage, windowTitle } from "./logic/workflow";
import { session, settings, status } from "./state";
import "./steps";

// "Before/After" in the zoom bar of the Wavelets and Save steps: the unprocessed stack instead of the sharpened image.
const before = ref(false);

const kind = computed(() => imageKind(session.view));
const image = computed(() =>
  pickImage(session.view, session.step, before.value, {
    frame: session.frameImage,
    stack: session.stackImage,
    processed: session.processedImage,
  }),
);
const wavelet = computed(() => kind.value === "wavelet");
startColour();
openDroppedFiles();
startResourceMonitor();
// ← and → page through the frames, Alt+1 to Alt+7 open the steps (shortcuts.ts).
onMounted(() => window.addEventListener("keydown", handleShortcut));
onBeforeUnmount(() => window.removeEventListener("keydown", handleShortcut));

// Wavelets and Save show the colour and tone cards applied to the image, or the remembered image or the difference to it;
// Before/After shows the plain stack.
const shown = computed(() => {
  if (!wavelet.value || before.value) return image.value;
  return pickToneImage(session.toneView, {
    current: session.adjustedImage ?? image.value,
    clipboard: session.clipboardImage,
    difference: session.differenceImage,
  });
});
const withFunctions = computed(() => session.view === "wavelets" && settings.wavelet.functionsOpen);
const emptyText = computed(() => (session.step === "empty" ? "Select a recording or drop a video or images here" : "No image to show yet"));
const caption = computed(() =>
  shown.value
    ? imageCaption(session.view, session.step, before.value, {
        frame: session.frame,
        frameCount: session.info?.frameCount ?? 0,
        stackSize: session.stackSize,
        processed: session.processedImage !== null,
      })
    : "",
);
// The frame bar belongs to the steps that work on single frames.
const withFrameBar = computed(() => ["alignpoints", "align", "limit"].includes(session.view));

watch(
  () => session.view,
  () => {
    before.value = false;
    status.position = "";
  },
);

// A region drawn with Draw Region ends the drawing and shows the rectangle.
function onSelect(rect: Rect) {
  session.roi = rect;
  session.roiDrawing = false;
  settings.align.showRoi = true;
}

// Position under the pointer, shown in the status bar of the Wavelets and Save steps.
function onHover(pixel: { x: number; y: number } | null) {
  if (!wavelet.value || !pixel) {
    status.position = "";
    return;
  }
  const stack = session.stackSize === null ? "" : ` Stack=${session.stackSize}`;
  status.position = `X=${pixel.x} Y=${pixel.y}${stack}`;
}

// The title of the window shows the file and, during a calculation, step and percent (also in the taskbar).
watch(
  () => windowTitle(session.fileName, status.busy ? { name: status.busy.name, progress: status.progress } : null),
  (title) => {
    document.title = title;
    try {
      getCurrentWindow()
        .setTitle(title)
        .catch(() => undefined);
    } catch {
      // Not running inside Tauri (browser preview of a dev build): the document title is enough.
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="shell">
    <Toolbar />
    <div class="work-view">
      <StepBar />
      <div class="work" :class="{ 'with-functions': withFunctions }">
        <OpenStep v-if="session.view === 'open'" />
        <AlignpointsStep v-else-if="session.view === 'alignpoints'" />
        <AlignStep v-else-if="session.view === 'align'" />
        <LimitStep v-else-if="session.view === 'limit'" />
        <StackStep v-else-if="session.view === 'stack'" />
        <WaveletsStep v-else-if="session.view === 'wavelets'" />
        <SaveStep v-else />
        <ImageViewer
          v-model:before="before"
          :image="shown"
          :comparable="wavelet"
          :empty="emptyText"
          :selecting="session.roiDrawing && kind === 'frame'"
          :fit-key="session.view"
          @hover="onHover"
          @select="onSelect"
        >
          <template #overlay>
            <ImageOverlays v-if="shown" :width="shown.width" :height="shown.height" />
          </template>
          <template #top>
            <ImageTopBar :caption="caption" />
          </template>
          <template #floats>
            <FloatingCards />
            <FunctionCards />
          </template>
        </ImageViewer>
        <FunctionsPanel v-if="withFunctions" />
      </div>
      <FrameBar v-if="withFrameBar" />
      <StatusBar />
    </div>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-width: 1120px;
  min-height: 600px;
}
.work-view {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
}
.work {
  display: grid;
  flex: 1;
  grid-template-columns: var(--side) minmax(0, 1fr);
  min-height: 0;
}
.work.with-functions {
  grid-template-columns: var(--side) minmax(0, 1fr) var(--side-right);
}
</style>
