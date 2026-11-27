import { createApp } from "vue";
import App from "./App.vue";
import HelpApp from "./HelpApp.vue";
import "./align";
import { initCpus } from "./compute";
import { startDerotation } from "./derotation";
import { HELP_QUERY } from "./helpWindow";
import { handleContextMenu, handleHelpShortcut } from "./shortcuts";
import { restoreTheme } from "./theme";
import "../../docs/design/tokens.css";
import "./styles.css";

restoreTheme();
// The help window loads the same page with ?view=help and shows only the documentation.
const helpWindow = new URLSearchParams(location.search).toString() === HELP_QUERY;
createApp(helpWindow ? HelpApp : App).mount("#app");

// Release builds: no native context menu except in text fields. Reload keys are blocked in shortcuts.ts, in the main window by
// handleShortcut (App.vue) and in the help window here.
document.addEventListener("contextmenu", handleContextMenu);
if (helpWindow) window.addEventListener("keydown", handleHelpShortcut);

// The CPUs field needs the number of cores from the backend.
if (!helpWindow) void initCpus();
// De-rotate images asks the core for span, rotation and disc whenever its settings or the kept frames change.
if (!helpWindow) startDerotation();

// PLANETS_PREVIEW_BENCH in the environment of the app starts the measurement of the live preview (previewBench.ts).
if (!helpWindow && "__TAURI_INTERNALS__" in window) void import("./previewBench").then((m) => m.runPreviewBench());

// Dev builds can jump into any state of the window without a recording (see preview.ts); release builds drop this.
if (import.meta.env.DEV && !helpWindow) void import("./preview").then((m) => m.applyPreview(location.search));
