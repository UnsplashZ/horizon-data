<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 180;
const BASE_H = 220;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(0, 0, 0, ${config.bg_opacity})` }));

// 综合G值
const gTotal = computed(() => {
  const gx = (t.value?.accel_x ?? 0) / 9.8;
  const gz = (t.value?.accel_z ?? 0) / 9.8;
  const total = Math.sqrt(gx * gx + gz * gz);
  return total.toFixed(2);
});

// G力指示器位置（0-100%），映射 ±3G 到圆盘范围
const dotX = computed(() => {
  const g = (t.value?.accel_x ?? 0) / 9.8;
  const clamped = Math.max(-3, Math.min(3, g));
  return 50 + (clamped / 3) * 45; // ±3G 映射到 ±45% 范围
});
const dotY = computed(() => {
  const g = (t.value?.accel_z ?? 0) / 9.8;
  const clamped = Math.max(-3, Math.min(3, g));
  return 50 - (clamped / 3) * 45; // Y轴反向
});
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }" :style="bg">
    <div class="scaler" :style="{
      transform: `scale(${scale})`,
      width: BASE_W + 'px',
      height: BASE_H + 'px',
      transformOrigin: 'top left'
    }">
      <div class="content" :style="{ opacity: config.fg_opacity }" @pointerdown="onDragDown">
        <!-- 标题 -->
        <div class="title">G-FORCE</div>

        <!-- G力圆盘 -->
        <div class="g-circle">
          <svg viewBox="0 0 120 120" class="g-grid">
            <!-- 外圈 -->
            <circle cx="60" cy="60" r="54" fill="none" stroke="rgba(0, 255, 255, 0.15)" stroke-width="1"/>
            <circle cx="60" cy="60" r="36" fill="none" stroke="rgba(0, 255, 255, 0.1)" stroke-width="1"/>
            <circle cx="60" cy="60" r="18" fill="none" stroke="rgba(0, 255, 255, 0.08)" stroke-width="1"/>

            <!-- 十字线 -->
            <line x1="6" y1="60" x2="114" y2="60" stroke="rgba(0, 255, 255, 0.2)" stroke-width="1"/>
            <line x1="60" y1="6" x2="60" y2="114" stroke="rgba(0, 255, 255, 0.2)" stroke-width="1"/>

            <!-- 方向标记 -->
            <text x="60" y="15" text-anchor="middle" fill="rgba(255, 255, 0, 0.4)" font-size="10" font-family="Consolas">+Z</text>
            <text x="60" y="112" text-anchor="middle" fill="rgba(255, 255, 0, 0.4)" font-size="10" font-family="Consolas">-Z</text>
            <text x="10" y="64" text-anchor="middle" fill="rgba(255, 0, 255, 0.4)" font-size="10" font-family="Consolas">-X</text>
            <text x="110" y="64" text-anchor="middle" fill="rgba(255, 0, 255, 0.4)" font-size="10" font-family="Consolas">+X</text>
          </svg>

          <!-- G力指示点 -->
          <div class="g-dot" :style="{ left: dotX + '%', top: dotY + '%' }"></div>
        </div>

        <!-- 数值显示 -->
        <div class="g-values">
          <div class="g-row">
            <span class="g-label">TOTAL G</span>
            <span class="g-num">{{ gTotal }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}
.editing .content {
  cursor: move;
}

.title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  text-align: center;
  color: #888;
  font-family: "Consolas", monospace;
}

/* G力圆盘 */
.g-circle {
  position: relative;
  width: 120px;
  height: 120px;
  background: #0a0a0a;
  border-radius: 50%;
  overflow: hidden;
  box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.8);
}

.g-grid {
  width: 100%;
  height: 100%;
}

.g-dot {
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #ff0000;
  transform: translate(-50%, -50%);
  transition: all 0.05s linear;
  border: 2px solid #fff;
}

/* 数值显示 */
.g-values {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  padding: 0 6px;
}

.g-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #0a0a0a;
  padding: 4px 10px;
  border: 1px solid #333;
  border-radius: 2px;
}

.g-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #888;
  font-family: "Consolas", monospace;
}

.g-num {
  font-size: 16px;
  font-weight: 900;
  font-family: "Consolas", monospace;
  color: #ffffff;
  letter-spacing: 1px;
}
</style>
