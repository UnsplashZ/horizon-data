<script setup lang="ts">
import { onMounted } from "vue";
import { config, editMode, initShared, updateConfig, quitApp } from "../telemetry";

let win: { startDragging: () => Promise<void>; startResizeDragging: (d: string) => Promise<void> } | null = null;

onMounted(async () => {
  await initShared();
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    win = getCurrentWindow() as unknown as typeof win;
  } catch {}
});

async function onTitleDown() {
  if (!win) return;
  try {
    await win.startDragging();
  } catch {}
}

async function onResizeDown(e: PointerEvent) {
  if (!editMode.value || !win) return;
  e.stopPropagation();
  try {
    await win.startResizeDragging("SouthEast");
  } catch {}
}
</script>

<template>
  <div class="panel">
    <div class="title" @pointerdown="onTitleDown">
      <span>HORIZON-DATA SETTINGS</span>
    </div>

    <div class="scroll-content">
      <label class="field">
        <span>数据透明度 {{ Math.round(config.fg_opacity * 100) }}%</span>
        <input type="range" min="0.2" max="1" step="0.01" v-model.number="config.fg_opacity" @change="updateConfig" />
      </label>
      <label class="field">
        <span>背景透明度 {{ Math.round(config.bg_opacity * 100) }}%</span>
        <input type="range" min="0" max="1" step="0.01" v-model.number="config.bg_opacity" @change="updateConfig" />
      </label>
      <label class="field">
        <span>端口</span>
        <input type="number" v-model.number="config.port" @change="updateConfig" />
      </label>
      <label class="field">
        <span>单位</span>
        <select v-model="config.units" @change="updateConfig">
          <option value="kmh">km/h</option>
          <option value="mph">mph</option>
        </select>
      </label>

      <div class="section">显示模块</div>
      <div class="toggles">
        <label><input type="checkbox" v-model="config.show_tires" @change="updateConfig" /> 轮胎信息（胎温+抓地）</label>
        <label><input type="checkbox" v-model="config.show_gforce" @change="updateConfig" /> G力显示</label>
        <label><input type="checkbox" v-model="config.show_inputs" @change="updateConfig" /> 输入显示（油门/刹车/转向）</label>
      </div>
      <div class="hint">所有模块集成在主仪表盘窗口中</div>

      <div class="hint">拖动窗口调整位置 · 再按 Ctrl+Shift+H 锁定</div>
      <button class="quit" @click="quitApp">退出程序</button>
    </div>

    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.panel {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  background: rgba(0, 0, 0, 0.95);
  font-size: 13px;
  color: #ffffff;
  overflow: hidden;
  font-family: "Consolas", monospace;
}

.title {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px 22px;
  font-weight: 900;
  font-size: 14px;
  letter-spacing: 3px;
  color: #ffffff;
  cursor: grab;
  background: #0a0a0a;
  text-transform: uppercase;
  flex-shrink: 0;
}

.title:active {
  cursor: grabbing;
}

.scroll-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 22px;
}

.scroll-content::-webkit-scrollbar {
  width: 8px;
}

.scroll-content::-webkit-scrollbar-track {
  background: #0a0a0a;
}

.scroll-content::-webkit-scrollbar-thumb {
  background: #333;
}

.scroll-content::-webkit-scrollbar-thumb:hover {
  background: #444;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.field span {
  color: #888;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 2px;
  text-transform: uppercase;
}

.field input[type="number"],
.field select {
  width: 100%;
  padding: 8px 12px;
  background: #0a0a0a;
  color: #ffffff;
  box-sizing: border-box;
  font-size: 12px;
  font-weight: 700;
  font-family: "Consolas", monospace;
  outline: none;
}

.field input[type="number"]:focus,
.field select:focus {
  background: #111;
}

.field input[type="range"] {
  padding: 0;
  cursor: pointer;
  height: 4px;
  background: #1a1a1a;
  outline: none;
  -webkit-appearance: none;
}

.field input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  background: #ffffff;
  cursor: pointer;
}

.field input[type="range"]::-moz-range-thumb {
  width: 14px;
  height: 14px;
  background: #ffffff;
  cursor: pointer;
  border: none;
}

.section {
  margin: 20px 0 12px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 2px;
  text-transform: uppercase;
  color: #888;
}

.toggles {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.toggles label {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #ffffff;
  font-weight: 700;
  font-size: 11px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.1s ease;
  background: #0a0a0a;
  letter-spacing: 1px;
}

.toggles label:hover {
  background: #111;
}

.toggles input[type="checkbox"] {
  cursor: pointer;
  width: 16px;
  height: 16px;
  accent-color: #00ff00;
}

.hint {
  font-size: 9px;
  font-weight: 700;
  color: #555;
  line-height: 1.6;
  margin-bottom: 16px;
  padding: 8px 12px;
  background: #0a0a0a;
}

.quit {
  width: 100%;
  padding: 12px;
  margin-top: 12px;
  background: #0a0a0a;
  color: #ffffff;
  font-family: "Consolas", monospace;
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 2px;
  cursor: pointer;
  text-transform: uppercase;
  transition: all 0.1s ease;
  outline: none;
}

.quit:hover {
  background: #00ff00;
  color: #000;
}

.resize {
  position: fixed;
  right: 0;
  bottom: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  background: linear-gradient(135deg, transparent 50%, #00ff00 51%);
}

.resize:hover {
  box-shadow: 0 0 10px rgba(0, 255, 0, 0.5);
}
</style>
