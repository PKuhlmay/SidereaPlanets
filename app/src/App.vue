<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AlignpointsStep from "./components/AlignpointsStep.vue";
import AlignStep from "./components/AlignStep.vue";
import AboutDialog from "./components/AboutDialog.vue";
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
import UpdateDialog from "./components/UpdateDialog.vue";
import WaveletsStep from "./components/WaveletsStep.vue";
import type { Rect } from "./logic/align";
import { startColour } from "./colour";
import { cancelCrop, clampedDivider, isComparing, setBefore, setCropDraft } from "./cropTool";
import { orientationApplies, orientImage } from "./geometryView";
import { formatAspect } from "./logic/crop";
import { startNativeBridge } from "./nativeBridge";
import { openDroppedFiles } from "./openFromOutside";
import { startResourceMonitor } from "./resource";
import { startUpdateCheck } from "./updater";
import { handleShortcut } from "./shortcuts";
import { pickToneImage } from "./logic/colour";
import { imageCaption, imageKind, pickImage, windowTitle } from "./logic/workflow";
import { session, settings, status } from "./state";
import "./steps";

// "Before/After" in the zoom bar of the Wavelets and Save steps (`session.before`): the unprocessed stack instead of the sharpened
// image. It and View Compare exclude each other (cropTool.ts).
const before = computed(() => session.before);
// View Compare: the stack left of the divider; the crop tool and its rectangle (SP-021).
const comparing = computed(() => isComparing());
const cropShown = computed(() =>
  session.view === "wavelets" || session.view === "save" ? (session.crop.editing ? session.crop.draft : session.crop.applied) : null,
);

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
void startNativeBridge();
startResourceMonitor();
startUpdateCheck();
// ← and → page through the frames, Alt+1 to Alt+7 (Mac: ⌥⌘1 to ⌥⌘7) open the steps, Cmd or Ctrl and a key run commands (shortcuts.ts).
onMounted(() => window.addEventListener("keydown", handleShortcut));
onBeforeUnmount(() => window.removeEventListener("keydown", handleShortcut));

// Wavelets and Save show the colour and tone cards applied to the image, or the remembered image or the difference to it;
// Before/After shows the plain stack.
const plainShown = computed(() => {
  if (!wavelet.value || before.value) return image.value;
  // Show mask (Masking card) replaces the current image; Before, the remembered image and the difference go first.
  if (session.toneView === "current" && session.maskImage) return session.maskImage;
  return pickToneImage(session.toneView, {
    current: session.adjustedImage ?? image.value,
    clipboard: session.clipboardImage,
    difference: session.differenceImage,
  });
});
// Flip and Rotate (SP-022): the steps Wavelets and Save show the picture turned by the core; until the turned copy is there, the plain one.
const turned = computed(() => wavelet.value && orientationApplies());
const shown = computed(() => (turned.value ? (session.oriented.shown ?? plainShown.value) : plainShown.value));
const compareImage = computed(() => (comparing.value ? session.stackImage : null));
const shownBefore = computed(() => (turned.value && compareImage.value ? (session.oriented.before ?? compareImage.value) : compareImage.value));
watch(
  [plainShown, turned, () => JSON.stringify(settings.geometry.orientation)],
  () => void orientImage("shown", turned.value ? plainShown.value : null),
);
watch(
  [compareImage, turned, () => JSON.stringify(settings.geometry.orientation)],
  () => void orientImage("before", turned.value ? compareImage.value : null),
);
const withFunctions = computed(() => session.view === "wavelets" && settings.wavelet.functionsOpen);
const emptyText = computed(() => (session.step === "empty" ? "Select a recording or drop a video or images here" : "No image to show yet"));
// The caption at the top left gives way while the image is split: the labels Before and After say what is where.
const caption = computed(() =>
  shown.value && !comparing.value
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
  (view) => {
    session.before = false;
    status.position = "";
    // The crop rectangle is edited only where the sharpened stack is shown.
    if (view !== "wavelets" && view !== "save") cancelCrop();
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
          :before="before"
          :image="shown"
          :before-image="shownBefore"
          :divider="session.compareDivider"
          :crop="cropShown"
          :crop-editing="session.crop.editing"
          :crop-aspect="session.crop.editing ? formatAspect(session.crop.format) : null"
          :comparable="wavelet"
          :empty="emptyText"
          :selecting="session.roiDrawing && kind === 'frame'"
          :fit-key="session.view"
          @update:before="setBefore"
          @hover="onHover"
          @select="onSelect"
          @divider="session.compareDivider = clampedDivider($event)"
          @crop="setCropDraft"
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
    <AboutDialog />
    <UpdateDialog />
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
