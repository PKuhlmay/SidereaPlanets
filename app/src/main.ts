import { createApp } from "vue";
import App from "./App.vue";
import "./align";
import { restoreTheme } from "./theme";
import "../../docs/design/tokens.css";
import "./styles.css";

restoreTheme();
createApp(App).mount("#app");

// Dev builds can jump into any state of the window without a recording (see preview.ts); release builds drop this.
if (import.meta.env.DEV) void import("./preview").then((m) => m.applyPreview(location.search));
