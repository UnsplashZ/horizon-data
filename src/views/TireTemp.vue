<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 190;
const BASE_H = 150;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));

// 华氏度转摄氏度
function fahrenheitToCelsius(f: number): number {
  return (f - 32) * 5 / 9;
}

// 冷(蓝)→最佳(绿)→过热(红)。阈值已转换为摄氏度。
// °F 阈值: 120°F (49°C) → 200°F (93°C) → 300°F (149°C)
function tempColor(tempC: number): string {
  const stops: [number, [number, number, number]][] = [
    [49, [74, 163, 255]],    // 冷：120°F = 49°C
    [93, [52, 227, 154]],    // 最佳：200°F = 93°C
    [149, [255, 90, 77]],    // 过热：300°F = 149°C
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

// 将华氏度转换为摄氏度显示
const temps = computed(() =>
  (t.value?.tire_temp ?? [0, 0, 0, 0]).map(f => Math.round(fahrenheitToCelsius(f)))
);
const tires = [
  { x: 12, y: 16, i: 0 },
  { x: 60, y: 16, i: 1 },
  { x: 12, y: 92, i: 2 },
  { x: 60, y: 92, i: 3 },
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
        <svg viewBox="0 0 100 134" class="car">
          <rect x="28" y="10" width="44" height="114" rx="16" fill="rgba(255,255,255,0.06)"
            stroke="rgba(255,255,255,0.18)" stroke-width="1.5" />
          <rect x="36" y="34" width="28" height="40" rx="8" fill="rgba(255,255,255,0.05)" />
          <g v-for="tire in tires" :key="tire.i">
            <rect :x="tire.x" :y="tire.y" width="28" height="26" rx="6" :fill="tempColor(temps[tire.i])" />
            <text :x="tire.x + 14" :y="tire.y + 17" class="tval">{{ temps[tire.i] }}°</text>
          </g>
        </svg>
        <div class="mlabel">胎温 / TEMP</div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 8px 10px 6px;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.editing .content {
  cursor: move;
}
.car {
  width: 104px;
  height: 134px;
}
.tval {
  fill: #0b0e12;
  font-size: 11px;
  font-weight: 700;
  text-anchor: middle;
}
</style>
