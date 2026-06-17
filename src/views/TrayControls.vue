<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { config, editMode, initShared, quitApp, setEditMode, updateConfig } from "../telemetry";

let win: { startDragging: () => Promise<void>; hide: () => Promise<void> } | null = null;
const portDraft = ref(String(config.port));
let portFocused = false;

onMounted(async () => {
  await initShared();
  portDraft.value = String(config.port);
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    win = getCurrentWindow() as unknown as typeof win;
  } catch {}
});

watch(
  () => config.port,
  (port) => {
    if (!portFocused) {
      portDraft.value = String(port);
    }
  },
);

async function onTitleDown() {
  if (!win) return;
  try {
    await win.startDragging();
  } catch {}
}

function onPortInput(event: Event) {
  portDraft.value = (event.target as HTMLInputElement).value;
}

function savePort() {
  const raw = portDraft.value.trim();
  const fallback = Number(config.port) || 10989;
  const n = raw === "" ? fallback : Math.round(Number(raw));
  config.port = Number.isFinite(n) ? Math.min(65535, Math.max(1, n)) : 10989;
  portDraft.value = String(config.port);
  portFocused = false;
  updateConfig();
}

function saveUnit(unit: "kmh" | "mph") {
  config.units = unit;
  updateConfig();
}

async function lockHud() {
  await setEditMode(false);
  try {
    await win?.hide();
  } catch {}
}
</script>

<template>
  <div class="panel">
    <header @pointerdown="onTitleDown">
      <span>HORIZON DATA</span>
      <strong>{{ editMode ? "EDIT" : "LOCK" }}</strong>
    </header>

    <main>
      <section class="row">
        <label>
          <span>PORT</span>
          <input
            type="number"
            min="1"
            max="65535"
            inputmode="numeric"
            :value="portDraft"
            @focus="portFocused = true"
            @input="onPortInput"
            @change="savePort"
            @keydown.enter.prevent="savePort"
          />
        </label>
        <div class="seg">
          <button :class="{ on: config.units === 'kmh' }" @click="saveUnit('kmh')">KM/H</button>
          <button :class="{ on: config.units === 'mph' }" @click="saveUnit('mph')">MPH</button>
        </div>
      </section>

      <section>
        <div class="title">OPACITY</div>
        <label class="slider">
          <span>HUD</span>
          <input type="range" min="0.2" max="1" step="0.01" v-model.number="config.fg_opacity" @input="updateConfig" />
          <b>{{ Math.round(config.fg_opacity * 100) }}%</b>
        </label>
        <label class="slider">
          <span>BG</span>
          <input type="range" min="0" max="1" step="0.01" v-model.number="config.bg_opacity" @input="updateConfig" />
          <b>{{ Math.round(config.bg_opacity * 100) }}%</b>
        </label>
      </section>

      <section>
        <div class="title">MODULES</div>
        <div class="checks">
          <label><input type="checkbox" v-model="config.show_tires" @change="updateConfig" /> TIRES</label>
          <label><input type="checkbox" v-model="config.show_inputs" @change="updateConfig" /> INPUTS</label>
          <label><input type="checkbox" v-model="config.show_gforce" @change="updateConfig" /> G-FORCE</label>
        </div>
      </section>

      <section class="actions">
        <button @click="setEditMode(true)">编辑 HUD</button>
        <button @click="lockHud">锁定</button>
        <button class="dark" @click="quitApp">退出</button>
      </section>
    </main>
  </div>
</template>

<style scoped>
.panel {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  overflow: hidden;
  background: rgba(6, 8, 11, 0.96);
  color: #fff;
  font-family: "Consolas", "SF Mono", monospace;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 14px;
  background: #0c0f13;
  cursor: grab;
  user-select: none;
}

header span,
.title,
label span {
  color: #8a929d;
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 2px;
}

header > span {
  color: #fff;
  font-size: 12px;
}

header strong {
  padding: 3px 8px;
  background: #18e06f;
  color: #041006;
  font-size: 10px;
  letter-spacing: 1px;
}

main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 16px;
  padding: 14px;
}

section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

input[type="number"] {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border: 0;
  outline: 0;
  background: #151a21;
  color: #fff;
  font: inherit;
  font-size: 13px;
  font-weight: 900;
}

.seg {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
  align-self: end;
}

button {
  min-height: 34px;
  border: 0;
  background: #151a21;
  color: #dce3ea;
  font: inherit;
  font-size: 11px;
  font-weight: 900;
  letter-spacing: 1px;
  cursor: pointer;
}

button.on,
.actions button:first-child {
  background: #18e06f;
  color: #041006;
}

.slider {
  display: grid;
  grid-template-columns: 42px 1fr 42px;
  align-items: center;
  gap: 10px;
}

.slider b {
  color: #dce3ea;
  font-size: 10px;
  text-align: right;
}

input[type="range"] {
  width: 100%;
  height: 5px;
  margin: 0;
  background: #1c232d;
  outline: none;
  cursor: pointer;
  -webkit-appearance: none;
}

input[type="range"]::-webkit-slider-thumb {
  width: 16px;
  height: 16px;
  background: #18e06f;
  cursor: pointer;
  -webkit-appearance: none;
}

input[type="range"]::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border: 0;
  background: #18e06f;
  cursor: pointer;
}

.checks {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.checks label {
  flex-direction: row;
  align-items: center;
  justify-content: center;
  min-height: 34px;
  background: #151a21;
  color: #dce3ea;
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 1px;
  cursor: pointer;
}

.checks input {
  width: 14px;
  height: 14px;
  margin: 0;
  accent-color: #18e06f;
}

.actions {
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 8px;
  margin-top: auto;
}

.actions .dark {
  min-width: 58px;
  background: #0c0f13;
  color: #fff;
}
</style>
