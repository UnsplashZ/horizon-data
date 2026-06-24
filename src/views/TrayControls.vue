<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import {
  config,
  configError,
  editMode,
  initShared,
  quitApp,
  setEditMode,
  shortcutStatus,
  udpStatus,
  updateConfig,
} from "../telemetry";

let win: { startDragging: () => Promise<void>; hide: () => Promise<void> } | null = null;
const portDraft = ref(String(config.port));
let portFocused = false;
const saved = {
  port: config.port,
  units: config.units,
  fg_opacity: config.fg_opacity,
  bg_opacity: config.bg_opacity,
  show_tires: config.show_tires,
  show_inputs: config.show_inputs,
  show_gforce: config.show_gforce,
  auto_hide_inactive: config.auto_hide_inactive,
};

function syncSaved() {
  saved.port = config.port;
  saved.units = config.units;
  saved.fg_opacity = config.fg_opacity;
  saved.bg_opacity = config.bg_opacity;
  saved.show_tires = config.show_tires;
  saved.show_inputs = config.show_inputs;
  saved.show_gforce = config.show_gforce;
  saved.auto_hide_inactive = config.auto_hide_inactive;
}

onMounted(async () => {
  await initShared();
  syncSaved();
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

// 失焦即解除编辑态并回显已保存端口，避免未触发 change 时草稿框被锁住、错过外部同步
function onPortBlur() {
  portFocused = false;
  portDraft.value = String(config.port);
}

async function savePort() {
  const raw = portDraft.value.trim();
  const fallback = Number(config.port) || 10989;
  const n = raw === "" ? fallback : Math.round(Number(raw));
  config.port = Number.isFinite(n) ? Math.min(65535, Math.max(1, n)) : 10989;
  portDraft.value = String(config.port);
  portFocused = false;
  const ok = await updateConfig();
  if (!ok) {
    config.port = saved.port;
    portDraft.value = String(saved.port);
  } else {
    syncSaved();
  }
}

async function saveUnit(unit: "kmh" | "mph") {
  config.units = unit;
  const ok = await updateConfig();
  if (!ok) {
    config.units = saved.units;
  } else {
    syncSaved();
  }
}

async function saveNumber<K extends "fg_opacity" | "bg_opacity">(key: K) {
  const ok = await updateConfig();
  if (!ok) {
    config[key] = saved[key];
  } else {
    syncSaved();
  }
}

async function saveToggle<K extends "show_tires" | "show_inputs" | "show_gforce" | "auto_hide_inactive">(key: K) {
  const ok = await updateConfig();
  if (!ok) {
    config[key] = saved[key];
  } else {
    syncSaved();
  }
}

async function lockHud() {
  await setEditMode(false);
  try {
    await win?.hide();
  } catch (error) {
    console.error("隐藏控制面板失败", error);
  }
}

// 仅关闭控制面板窗口，不改动编辑/锁定状态
async function closeMenu() {
  try {
    await win?.hide();
  } catch (error) {
    console.error("关闭控制面板失败", error);
  }
}
</script>

<template>
  <div class="panel">
    <header @pointerdown="onTitleDown">
      <span>HORIZON DATA</span>
      <div class="head-right">
        <strong>{{ editMode ? "EDIT" : "LOCK" }}</strong>
        <button class="close" title="关闭菜单" @pointerdown.stop @click="closeMenu">×</button>
      </div>
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
            @blur="onPortBlur"
            @keydown.enter.prevent="savePort"
          />
        </label>
        <div class="seg">
          <button :class="{ on: config.units === 'kmh' }" @click="saveUnit('kmh')">KM/H</button>
          <button :class="{ on: config.units === 'mph' }" @click="saveUnit('mph')">MPH</button>
        </div>
      </section>

      <section class="status" v-if="udpStatus || configError || shortcutStatus?.error">
        <div
          v-if="udpStatus"
          class="status-line"
          :class="{ ok: udpStatus.listening && !udpStatus.error, bad: Boolean(udpStatus.error) }"
        >
          UDP {{ udpStatus.port }} {{ udpStatus.listening ? "LISTENING" : udpStatus.error ? "ERROR" : "WAITING" }}
        </div>
        <div v-if="udpStatus?.error" class="status-msg">{{ udpStatus.error }}</div>
        <div v-if="configError" class="status-msg">CONFIG {{ configError }}</div>
        <div v-if="shortcutStatus?.error" class="status-msg">SHORTCUT {{ shortcutStatus.error }}</div>
      </section>

      <section>
        <div class="title">OPACITY</div>
        <label class="slider">
          <span>HUD</span>
          <input
            type="range"
            min="0.2"
            max="1"
            step="0.01"
            v-model.number="config.fg_opacity"
            @input="saveNumber('fg_opacity')"
          />
          <b>{{ Math.round(config.fg_opacity * 100) }}%</b>
        </label>
        <label class="slider">
          <span>BG</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            v-model.number="config.bg_opacity"
            @input="saveNumber('bg_opacity')"
          />
          <b>{{ Math.round(config.bg_opacity * 100) }}%</b>
        </label>
      </section>

      <section>
        <div class="title">MODULES</div>
        <div class="checks">
          <label><input type="checkbox" v-model="config.show_tires" @change="saveToggle('show_tires')" /> TIRES</label>
          <label><input type="checkbox" v-model="config.show_inputs" @change="saveToggle('show_inputs')" /> INPUTS</label>
          <label><input type="checkbox" v-model="config.show_gforce" @change="saveToggle('show_gforce')" /> G-FORCE</label>
        </div>
      </section>

      <section>
        <div class="title">BEHAVIOR</div>
        <div class="checks behavior">
          <label>
            <input type="checkbox" v-model="config.auto_hide_inactive" @change="saveToggle('auto_hide_inactive')" />
            AUTO HIDE INACTIVE
          </label>
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

.head-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

button.close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  min-height: 0;
  padding: 0;
  background: #151a21;
  color: #8a929d;
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: 0;
}

button.close:hover {
  background: #ff4d4d;
  color: #fff;
}

main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
}

section {
  display: flex;
  flex-direction: column;
  gap: 8px;
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

.status {
  gap: 4px;
}

.status-line,
.status-msg {
  box-sizing: border-box;
  padding: 6px 8px;
  background: #151a21;
  color: #dce3ea;
  font-size: 10px;
  font-weight: 900;
  line-height: 1.25;
}

.status-line.ok {
  color: #18e06f;
}

.status-line.bad,
.status-msg {
  color: #ff7a70;
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

.checks.behavior {
  grid-template-columns: 1fr;
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
