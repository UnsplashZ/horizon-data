<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 180;
const BASE_H = 220;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));

// 抓地力颜色
function gripColor(slip: number): string {
  const s = Math.min(1, Math.max(0, slip / 1.2));
  const r = Math.round(52 + s * 203);
  const g = Math.round(227 - s * 160);
  const b = Math.round(150 - s * 80);
  return `rgb(${r}, ${g}, ${b})`;
}

function slipping(slip: number): boolean {
  return slip > 0.95;
}

// 华氏度转摄氏度
function fahrenheitToCelsius(f: number): number {
  return (f - 32) * 5 / 9;
}

// 胎温颜色
function tempColor(tempC: number): string {
  const stops: [number, [number, number, number]][] = [
    [49, [74, 163, 255]],
    [93, [52, 227, 154]],
    [149, [255, 90, 77]],
  ];
  if (tempC <= stops[0][0]) return rgb(stops[0][1]);
  if (tempC >= stops[2][0]) return rgb(stops[2][1]);
  const [lo, hi] = tempC < stops[1][0] ? [stops[0], stops[1]] : [stops[1], stops[2]];
  const r = (tempC - lo[0]) / (hi[0] - lo[0]);
  return rgb([
    Math.round(lo[1][0] + (hi[1][0] - lo[1][0]) * r),
    Math.round(lo[1][1] + (hi[1][1] - lo[1][1]) * r),
    Math.round(lo[1][2] + (hi[1][2] - lo[1][2]) * r),
  ]);
}

function rgb(c: [number, number, number]): string {
  return `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
}

const slips = computed(() => t.value?.tire_slip ?? [0, 0, 0, 0]);
const temps = computed(() =>
  (t.value?.tire_temp ?? [0, 0, 0, 0]).map(f => Math.round(fahrenheitToCelsius(f)))
);

const tires = [
  { x: 20, y: 20, i: 0, label: "FL" },
  { x: 108, y: 20, i: 1, label: "FR" },
  { x: 20, y: 142, i: 2, label: "RL" },
  { x: 108, y: 142, i: 3, label: "RR" },
];
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
        <div class="title">TIRES</div>

        <!-- 轮胎数据网格 -->
        <div class="tire-grid">
          <div v-for="tire in tires" :key="tire.i" class="tire-card">
            <!-- 轮胎位置标签 -->
            <div class="tire-label">{{ tire.label }}</div>

            <!-- 抓地力指示器 -->
            <div class="grip-bar">
              <div
                class="grip-fill"
                :class="{ slip: slipping(slips[tire.i]) }"
                :style="{
                  height: `${Math.min(100, slips[tire.i] * 100)}%`,
                  background: gripColor(slips[tire.i])
                }"
              ></div>
            </div>

            <!-- 胎温显示 -->
            <div class="temp-display" :style="{ background: tempColor(temps[tire.i]) }">
              <span class="temp-value">{{ temps[tire.i] }}°</span>
            </div>
          </div>
        </div>

        <!-- 图例 -->
        <div class="legend">
          <span class="legend-item"><span class="legend-bar"></span>GRIP</span>
          <span class="legend-item"><span class="legend-temp"></span>TEMP</span>
        </div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.editing .content {
  cursor: move;
}

.title {
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 3px;
  text-align: center;
  color: #00ffff;
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.8);
  font-family: "Consolas", "SF Mono", monospace;
}

.tire-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 10px;
  flex: 1;
}

.tire-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px;
  background: radial-gradient(circle at 30% 30%, rgba(0, 255, 255, 0.08) 0%, rgba(0, 0, 0, 0.6) 70%);
  clip-path: polygon(6px 0%, calc(100% - 6px) 0%, 100% 50%, calc(100% - 6px) 100%, 6px 100%, 0% 50%);
  border: 1px solid rgba(0, 255, 255, 0.2);
}

.tire-label {
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 2px;
  color: rgba(255, 255, 255, 0.5);
  font-family: "Consolas", monospace;
}

.grip-bar {
  width: 24px;
  height: 40px;
  background: rgba(0, 0, 0, 0.6);
  clip-path: polygon(20% 0%, 80% 0%, 100% 100%, 0% 100%);
  border: 1px solid rgba(255, 0, 255, 0.2);
  display: flex;
  flex-direction: column-reverse;
  overflow: hidden;
  box-shadow: inset 0 4px 8px rgba(0, 0, 0, 0.8);
}

.grip-fill {
  width: 100%;
  transition: height 0.1s linear;
  box-shadow: 0 0 12px currentColor;
}

.grip-fill.slip {
  animation: slip-flash 0.15s steps(2) infinite;
}

@keyframes slip-flash {
  50% {
    opacity: 0.3;
  }
}

.temp-display {
  width: 100%;
  padding: 6px 4px;
  text-align: center;
  clip-path: polygon(4px 0%, calc(100% - 4px) 0%, 100% 50%, calc(100% - 4px) 100%, 4px 100%, 0% 50%);
  border: 1px solid rgba(0, 0, 0, 0.5);
  box-shadow:
    inset 0 2px 4px rgba(255, 255, 255, 0.2),
    0 0 15px currentColor;
}

.temp-value {
  font-size: 14px;
  font-weight: 900;
  color: rgba(0, 0, 0, 0.9);
  font-family: "Consolas", monospace;
  text-shadow:
    0 1px 2px rgba(255, 255, 255, 0.4),
    0 0 8px currentColor;
  letter-spacing: 1px;
}

.legend {
  display: flex;
  justify-content: center;
  gap: 16px;
  font-size: 9px;
  font-weight: 900;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.5);
  font-family: "Consolas", monospace;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.legend-bar {
  width: 12px;
  height: 8px;
  background: linear-gradient(0deg, #00ff88 0%, #ffff00 50%, #ff0040 100%);
  clip-path: polygon(20% 0%, 80% 0%, 100% 100%, 0% 100%);
  box-shadow: 0 0 8px rgba(0, 255, 136, 0.5);
}

.legend-temp {
  width: 12px;
  height: 8px;
  background: linear-gradient(90deg, #00a3ff 0%, #00ff88 50%, #ff0040 100%);
  clip-path: polygon(15% 0%, 85% 0%, 100% 50%, 85% 100%, 15% 100%, 0% 50%);
  box-shadow: 0 0 8px rgba(0, 163, 255, 0.5);
}
</style>
