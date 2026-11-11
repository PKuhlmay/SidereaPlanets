<script setup lang="ts">
import { computed } from "vue";
import { runAction } from "../actions";
import { cancelJob, togglePause } from "../jobs";
import { LUMINANCE, type ProcessingArea, type StretchFit } from "../logic/settings";
import { recentLabel } from "../logic/recent";
import { applyTheme } from "../theme";
import { chooseFiles, openRecording, recent } from "../recording";
import { session, settings, status, theme } from "../state";
import { openProject, saveProject } from "../saving";
import MenuButton from "./MenuButton.vue";
import NumberField from "./NumberField.vue";

defineProps<{ docsOpen: boolean }>();
const emit = defineEmits<{ toggleDocs: [] }>();

const loaded = computed(() => session.step !== "empty");
const busy = computed(() => status.busy !== null);

const processingAreas: ProcessingArea[] = [2048, 1024, 512, 256, 128];
const stretchChoices: { value: StretchFit; label: string }[] = [
  { value: "ask", label: "Always Ask" },
  { value: "yes", label: "Yes" },
  { value: "no", label: "No" },
];

// The value shown at the right of a menu entry that opens a submenu.
const automaticText = computed(() => {
  const on = settings.menu.automatic;
  const names = [on.alignToWavelet && "Align to Wavelet", on.alignToStack && "Align to Stack", on.stackToWavelet && "Stack to Wavelet"];
  return names.filter(Boolean).join(", ") || "Off";
});
const luminanceText = computed(() => {
  const l = settings.menu.luminance;
  return `${l.red} · ${l.green} · ${l.blue}`;
});
const stretchText = computed(() => stretchChoices.find((c) => c.value === settings.menu.stretchFit)?.label ?? "");

function resetLuminance() {
  Object.assign(settings.menu.luminance, LUMINANCE);
}
</script>

<template>
  <header class="toolbar">
    <div class="zone">
      <button type="button" class="btn next select" :disabled="busy" @click="chooseFiles">Select</button>

      <MenuButton v-slot="{ close }" label="MRU">
        <div v-if="recent.list.length === 0" class="empty">No recent files</div>
        <button
          v-for="paths in recent.list"
          :key="paths.join('|')"
          type="button"
          class="mi"
          :title="paths.join(', ')"
          @click="close(), openRecording(paths)"
        >
          {{ recentLabel(paths) }}
        </button>
      </MenuButton>

      <!-- Darkframe, flatfield and reference frame: loading is not built yet, so the entries that need a loaded file stay locked. -->
      <MenuButton v-slot="{ close }" label="Flat/Dark/Reference">
        <button type="button" class="mi" :disabled="!loaded" @click="close(), runAction('loadDarkframe')">Load Darkframe</button>
        <label class="mi"><input type="checkbox" disabled />Use Darkframe</label>
        <label class="mi"><input type="checkbox" disabled />Use Raw-Darkframe</label>
        <div class="mi off">
          Create Darkframe
          <NumberField v-model="settings.menu.darkframeFactor" :decimals="2" :step="0.1" disabled label="Darkframe factor" />
        </div>
        <hr />
        <button type="button" class="mi" :disabled="!loaded" @click="close(), runAction('loadFlatfield')">Load Flatfield</button>
        <label class="mi"><input type="checkbox" disabled />Use Flatfield</label>
        <div class="mi off">Create Flatfield</div>
        <hr />
        <button type="button" class="mi" :disabled="!loaded" @click="close(), runAction('loadReferenceframe')">Load Referenceframe</button>
        <label class="mi"><input type="checkbox" disabled />Use Referenceframe</label>
      </MenuButton>

      <MenuButton v-slot="{ close }" label="Tools">
        <button type="button" class="mi" @click="close(), runAction('macroBatch')">Show MACRO/Batch window</button>
        <label class="mi"><input v-model="settings.menu.recordInFilename" type="checkbox" />Record Processing Information in Filename</label>
      </MenuButton>

      <MenuButton label="Settings" wide>
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Automatic Processing <span class="v arrow">{{ automaticText }}</span></button>
          <div class="menu fly">
            <label class="mi"><input v-model="settings.menu.automatic.alignToWavelet" type="checkbox" />Align to Wavelet</label>
            <label class="mi"><input v-model="settings.menu.automatic.alignToStack" type="checkbox" />Align to Stack</label>
            <label class="mi"><input v-model="settings.menu.automatic.stackToWavelet" type="checkbox" />Stack to Wavelet</label>
          </div>
        </div>
        <label class="mi"><input v-model="settings.menu.autodetectColour" type="checkbox" />Autodetect Colour/BW</label>
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Processing Area <span class="v arrow">{{ settings.menu.processingArea }}</span></button>
          <div class="menu fly">
            <label v-for="size in processingAreas" :key="size" class="mi">
              <input v-model="settings.menu.processingArea" type="radio" name="processing-area" :value="size" />{{ size }}
            </label>
          </div>
        </div>
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Luminance Setting <span class="v arrow">{{ luminanceText }}</span></button>
          <div class="menu fly">
            <button type="button" class="mi" @click="resetLuminance">Reset Luminance</button>
            <hr />
            <div class="mi">Red <NumberField v-model="settings.menu.luminance.red" :decimals="3" :step="0.001" label="Red" /></div>
            <div class="mi">Green <NumberField v-model="settings.menu.luminance.green" :decimals="3" :step="0.001" label="Green" /></div>
            <div class="mi">Blue <NumberField v-model="settings.menu.luminance.blue" :decimals="3" :step="0.001" label="Blue" /></div>
          </div>
        </div>
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Stretch FIT/TIFF <span class="v arrow">{{ stretchText }}</span></button>
          <div class="menu fly">
            <label v-for="choice in stretchChoices" :key="choice.value" class="mi">
              <input v-model="settings.menu.stretchFit" type="radio" name="stretch-fit" :value="choice.value" />{{ choice.label }}
            </label>
          </div>
        </div>
        <!-- Takes the place of "Set Skin" in the original: the light theme is part of the design (docs/design.md). -->
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Theme <span class="v arrow">{{ theme.current === "dark" ? "Dark" : "Light" }}</span></button>
          <div class="menu fly">
            <label class="mi"><input type="radio" name="theme" :checked="theme.current === 'dark'" @change="applyTheme('dark')" />Dark</label>
            <label class="mi"><input type="radio" name="theme" :checked="theme.current === 'light'" @change="applyTheme('light')" />Light</label>
          </div>
        </div>
        <hr />
        <label class="mi"><input v-model="settings.menu.advancedControls" type="checkbox" />Show Advanced Controls</label>
        <label class="mi"><input v-model="settings.menu.playSounds" type="checkbox" />Play Sounds</label>
        <label class="mi"><input v-model="settings.menu.swapRedBlue" type="checkbox" />Swap Red/Blue Channel</label>
        <label class="mi"><input v-model="settings.menu.waitSingleImage" type="checkbox" />Wait when loading a single image</label>
        <label class="mi"><input v-model="settings.menu.maximizeMessages" type="checkbox" />Maximize during messages</label>
        <hr />
        <label class="mi">Save to Subdir <input v-model="settings.menu.saveToSubdir" type="text" placeholder="Subdirname" /></label>
        <div class="mi">Bias subtract <NumberField v-model="settings.menu.biasSubtract" label="Bias subtract" /></div>
      </MenuButton>

      <!-- Not in the original: the project file keeps the recording and the settings of a session (docs/formats.md). -->
      <button type="button" class="btn ghost" title="Open project" aria-label="Open project" :disabled="busy" @click="openProject">
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M1.5 3.5h4l1.5 1.5h7.5v7.5h-13z" />
        </svg>
        Open
      </button>
      <button type="button" class="btn ghost" title="Save project" aria-label="Save project" :disabled="!loaded || busy" @click="saveProject">
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M2.5 2.5h8l3 3v8h-11zM5 2.5v3.5h5V2.5M5 13.5v-4h6v4" />
        </svg>
        Save
      </button>
    </div>

    <div class="zone mid">
      <button type="button" class="btn" :class="{ next: busy }" :disabled="!busy" @click="cancelJob">Cancel</button>
      <button type="button" class="btn" :disabled="!busy" @click="togglePause">{{ status.busy?.paused ? "Resume" : "Pause" }}</button>
    </div>

    <div class="zone right">
      <button type="button" class="btn ghost" :aria-pressed="docsOpen" @click="emit('toggleDocs')">
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M8 3.5C6.5 2.5 4.5 2.3 1.5 2.5v10c3-.2 5 0 6.5 1zm0 0c1.5-1 3.5-1.2 6.5-1v10c-3-.2-5 0-6.5 1" />
        </svg>
        Help
      </button>
      <button type="button" class="btn ghost" @click="runAction('about')">About</button>
      <label class="cpus">CPUs <NumberField v-model="settings.cpus" :min="1" :width="52" label="CPUs" /></label>
    </div>
  </header>
</template>

<style scoped>
.toolbar {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 12px;
  align-items: center;
  padding: 8px 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
}
.zone {
  display: flex;
  gap: 4px;
  align-items: center;
}
.zone.mid {
  justify-content: center;
}
.zone.right {
  justify-content: flex-end;
  gap: 10px;
}
.select {
  margin-right: 8px;
  padding: 0 18px;
}
.cpus {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* menu entries (the list itself is styled in MenuButton) */
.mi {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  text-align: left;
  background: none;
  border: 0;
  border-radius: var(--r-s);
  cursor: pointer;
}
.mi:hover:not(:disabled),
.sw:focus-within > .mi {
  background: var(--accent-soft);
}
.mi:disabled,
.mi.off,
.mi:has(input:disabled) {
  color: var(--off);
  cursor: default;
}
.mi:disabled:hover,
.mi:has(input:disabled):hover {
  background: none;
}
.mi .v {
  margin-left: auto;
  padding-left: 16px;
  color: var(--muted);
}
.mi .arrow::after {
  content: " ›";
}
.mi input[type="text"] {
  width: 110px;
  height: 24px;
  margin-left: auto;
  user-select: text;
}
.mi > :deep(.num) {
  margin-left: auto;
}
hr {
  margin: 4px 0;
  border: 0;
  border-top: 1px solid var(--line);
}
.empty {
  padding: 8px;
  color: var(--muted);
}

/* submenus open to the right of their entry on hover or focus */
.sw {
  position: relative;
}
.fly {
  position: absolute;
  top: -5px;
  left: calc(100% - 2px);
  z-index: 31;
  display: none;
  min-width: 220px;
  padding: 4px;
  background: var(--raised);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
}
.sw:hover > .fly,
.sw:focus-within > .fly {
  display: block;
}
</style>
