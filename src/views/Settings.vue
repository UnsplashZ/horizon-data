<script setup lang="ts">
import { onMounted } from "vue";
import { config, initShared, updateConfig, quitApp } from "../telemetry";

onMounted(initShared);
</script>

<template>
  <div class="panel">
    <div class="title">horizon-data · 设置</div>

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

    <div class="section">模块（独立窗口）</div>
    <div class="toggles">
      <label><input type="checkbox" v-model="config.show_inputs" @change="updateConfig" /> 输入（油门/刹车/转向）</label>
      <label><input type="checkbox" v-model="config.show_grip" @change="updateConfig" /> 摩擦力 / 抓地</label>
      <label><input type="checkbox" v-model="config.show_gforce" @change="updateConfig" /> G力</label>
    </div>

    <div class="hint">拖动任意窗口调整位置 · 再按 ⌘/Ctrl+Shift+H 锁定</div>
    <button class="quit" @click="quitApp">退出程序</button>
  </div>
</template>

<style scoped>
.panel {
  position: fixed;
  inset: 0;
  padding: 16px 18px;
  box-sizing: border-box;
  background: rgba(18, 22, 30, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 14px;
  font-size: 13px;
  color: #e6eef6;
  overflow: hidden;
}
.title {
  font-weight: 700;
  margin-bottom: 14px;
  color: #9fd0ff;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 11px;
}
.field span {
  color: #9fb4c8;
}
.field input,
.field select {
  width: 100%;
  padding: 5px 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(0, 0, 0, 0.3);
  color: #fff;
  box-sizing: border-box;
}
.field input[type="range"] {
  padding: 0;
}
.section {
  margin: 6px 0 8px;
  font-size: 11px;
  letter-spacing: 1px;
  color: #7e93a8;
}
.toggles {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 14px;
}
.toggles label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #cdd9e5;
}
.hint {
  font-size: 11px;
  color: #7e93a8;
  line-height: 1.5;
  margin-bottom: 12px;
}
.quit {
  width: 100%;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 90, 77, 0.5);
  background: rgba(255, 90, 77, 0.15);
  color: #ff8a80;
  cursor: pointer;
}
.quit:hover {
  background: rgba(255, 90, 77, 0.28);
}
</style>
