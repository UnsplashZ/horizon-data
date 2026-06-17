import { createApp, type Component } from "vue";
import "./style.css";

const view = new URLSearchParams(location.search).get("view") ?? "main";

const loaders: Record<string, () => Promise<{ default: Component }>> = {
  main: () => import("./views/HudMain.vue"),
  controls: () => import("./views/TrayControls.vue"),
};

(loaders[view] ?? loaders.main)().then((m) => createApp(m.default).mount("#app"));
