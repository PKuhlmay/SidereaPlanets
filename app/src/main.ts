import { createApp } from "vue";
import App from "./App.vue";
import HelpApp from "./HelpApp.vue";
import "./align";
import { HELP_QUERY } from "./helpWindow";
import { restoreTheme } from "./theme";
import "../../docs/design/tokens.css";
import "./styles.css";

restoreTheme();
// The help window loads the same page with ?view=help and shows only the documentation.
const helpWindow = new URLSearchParams(location.search).toString() === HELP_QUERY;
createApp(helpWindow ? HelpApp : App).mount("#app");

// Dev builds can jump into any state of the window without a recording (see preview.ts); release builds drop this.
if (import.meta.env.DEV && !helpWindow) void import("./preview").then((m) => m.applyPreview(location.search));
