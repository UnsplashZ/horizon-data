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
  } catch {
    // 纯浏览器（无 Tauri）下窗口 API 不可用，属预期降级，保持静默
  }
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
  } catch (error) {
    console.error("拖动控制面板失败", error);
  }
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
      <span>Horizon Data</span>
      <div class="head-right">
        <strong>{{ editMode ? "编辑中" : "已锁定" }}</strong>
        <button class="close" title="关闭菜单" @pointerdown.stop @click="closeMenu">×</button>
      </div>
    </header>

    <main>
      <section class="row">
        <label>
          <span>UDP 端口</span>
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
          <button :class="{ on: config.units === 'kmh' }" @click="saveUnit('kmh')">km/h</button>
          <button :class="{ on: config.units === 'mph' }" @click="saveUnit('mph')">MPH</button>
        </div>
      </section>

      <section class="status" v-if="udpStatus || configError || shortcutStatus?.error">
        <div
          v-if="udpStatus"
          class="status-line"
          :class="{ ok: udpStatus.listening && !udpStatus.error, bad: Boolean(udpStatus.error) }"
        >
          UDP {{ udpStatus.port }} {{ udpStatus.listening ? "正在监听" : udpStatus.error ? "错误" : "等待中" }}
        </div>
        <div v-if="udpStatus?.error" class="status-msg">{{ udpStatus.error }}</div>
        <div v-if="configError" class="status-msg">配置 {{ configError }}</div>
        <div v-if="shortcutStatus?.error" class="status-msg">快捷键 {{ shortcutStatus.error }}</div>
      </section>

      <section>
        <div class="title">外观</div>
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
          <span>背景</span>
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
        <div class="title">模块</div>
        <div class="checks">
          <label><input type="checkbox" v-model="config.show_tires" @change="saveToggle('show_tires')" /> 轮胎</label>
          <label><input type="checkbox" v-model="config.show_inputs" @change="saveToggle('show_inputs')" /> 输入</label>
          <label><input type="checkbox" v-model="config.show_gforce" @change="saveToggle('show_gforce')" /> G 力</label>
        </div>
      </section>

      <section>
        <div class="title">行为</div>
        <div class="checks behavior">
          <label>
            <input type="checkbox" v-model="config.auto_hide_inactive" @change="saveToggle('auto_hide_inactive')" />
            非活动时自动隐藏
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
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 10px;
  background: rgba(31, 31, 34, 0.88);
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.38), inset 0 1px rgba(255, 255, 255, 0.08);
  color: #f5f5f7;
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif;
  -webkit-backdrop-filter: blur(28px) saturate(1.35);
  backdrop-filter: blur(28px) saturate(1.35);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 48px;
  padding: 0 12px 0 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.025);
  cursor: grab;
  user-select: none;
}

header span,
.title,
label span {
  color: rgba(235, 235, 245, 0.62);
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0;
}

header > span {
  color: #f5f5f7;
  font-size: 13px;
  font-weight: 600;
}

header strong {
  padding: 3px 8px;
  border: 1px solid rgba(48, 209, 88, 0.26);
  border-radius: 8px;
  background: rgba(48, 209, 88, 0.13);
  color: #62d97a;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0;
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
  width: 22px;
  height: 22px;
  min-height: 0;
  padding: 0;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(235, 235, 245, 0.68);
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: 0;
}

button.close:hover {
  border-color: rgba(255, 69, 58, 0.55);
  background: #ff453a;
  color: #fff;
}

main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  overflow-y: auto;
}

section {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.055);
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
  height: 32px;
  padding: 6px 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  outline: 0;
  background: rgba(0, 0, 0, 0.22);
  color: #f5f5f7;
  font: inherit;
  font-size: 12px;
  font-weight: 500;
}

input[type="number"]:focus {
  border-color: rgba(10, 132, 255, 0.72);
  box-shadow: 0 0 0 3px rgba(10, 132, 255, 0.16);
}

.status {
  gap: 4px;
}

.status-line,
.status-msg {
  box-sizing: border-box;
  padding: 6px 8px;
  border-radius: 5px;
  background: rgba(0, 0, 0, 0.18);
  color: rgba(235, 235, 245, 0.82);
  font-size: 10px;
  font-weight: 600;
  line-height: 1.25;
}

.status-line.ok {
  color: #62d97a;
}

.status-line.bad,
.status-msg {
  color: #ff6961;
}

.seg {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2px;
  padding: 2px;
  border-radius: 7px;
  background: rgba(0, 0, 0, 0.25);
  align-self: end;
}

button {
  min-height: 30px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(245, 245, 247, 0.9);
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0;
  cursor: pointer;
}

button.on,
.actions button:first-child {
  border-color: rgba(255, 255, 255, 0.16);
  background: rgba(255, 255, 255, 0.19);
  color: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
}

button:hover {
  background: rgba(255, 255, 255, 0.14);
}

.slider {
  display: grid;
  grid-template-columns: 68px 1fr 38px;
  align-items: center;
  gap: 10px;
}

.slider b {
  color: rgba(235, 235, 245, 0.74);
  font-size: 10px;
  text-align: right;
}

input[type="range"] {
  width: 100%;
  height: 4px;
  margin: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16);
  outline: none;
  cursor: pointer;
  -webkit-appearance: none;
}

input[type="range"]::-webkit-slider-thumb {
  width: 16px;
  height: 16px;
  border: 0.5px solid rgba(0, 0, 0, 0.2);
  border-radius: 50%;
  background: #f5f5f7;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
  cursor: pointer;
  -webkit-appearance: none;
}

input[type="range"]::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border: 0;
  border-radius: 50%;
  background: #f5f5f7;
  cursor: pointer;
}

.checks {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1px;
  overflow: hidden;
  border-radius: 6px;
}

.checks.behavior {
  grid-template-columns: 1fr;
}

.checks label {
  flex-direction: row-reverse;
  align-items: center;
  justify-content: space-between;
  min-height: 30px;
  padding: 0 8px;
  background: rgba(0, 0, 0, 0.16);
  color: rgba(245, 245, 247, 0.9);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0;
  cursor: pointer;
}

.checks input {
  position: relative;
  width: 30px;
  height: 18px;
  margin: 0;
  border-radius: 999px;
  background: rgba(120, 120, 128, 0.42);
  cursor: pointer;
  transition: background 0.16s ease;
  appearance: none;
}

.checks input::before {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
  content: "";
  transition: transform 0.16s ease;
}

.checks input:checked {
  background: #30d158;
}

.checks input:checked::before {
  transform: translateX(12px);
}

.actions {
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 8px;
  margin-top: auto;
  padding: 0;
  border: 0;
  background: transparent;
}

.actions .dark {
  min-width: 58px;
  border-color: rgba(255, 69, 58, 0.22);
  background: rgba(255, 69, 58, 0.12);
  color: #ff6961;
}
</style>
