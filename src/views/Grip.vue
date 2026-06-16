<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useDragWindow } from "../dragwin";

const { onPointerDown } = useDragWindow();
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));

/** combined slip：~<0.3 抓地，>1 明显打滑 */
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
const slips = computed(() => t.value?.tire_slip ?? [0, 0, 0, 0]);
// 顺序：0=FL 1=FR 2=RL 3=RR
const tires = [
  { x: 14, y: 18, i: 0 },
  { x: 62, y: 18, i: 1 },
  { x: 14, y: 92, i: 2 },
  { x: 62, y: 92, i: 3 },
];
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }" :style="bg" @pointerdown="onPointerDown">
    <div class="content" :style="{ opacity: config.fg_opacity }">
      <svg viewBox="0 0 100 134" class="car">
        <!-- 车体 -->
        <rect x="28" y="10" width="44" height="114" rx="16" fill="rgba(255,255,255,0.06)"
          stroke="rgba(255,255,255,0.18)" stroke-width="1.5" />
        <rect x="36" y="34" width="28" height="40" rx="8" fill="rgba(255,255,255,0.05)" />
        <!-- 四轮 -->
        <g v-for="tire in tires" :key="tire.i">
          <rect
            :x="tire.x" :y="tire.y" width="24" height="24" rx="6"
            :fill="gripColor(slips[tire.i])"
            :class="{ slip: slipping(slips[tire.i]) }"
          />
        </g>
      </svg>
      <div class="mlabel">抓地 / GRIP</div>
    </div>
  </div>
</template>

<style scoped>
.content {
  padding: 8px 10px 6px;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.car {
  width: 100px;
  height: 134px;
}
rect.slip {
  animation: pulse 0.25s steps(1) infinite;
}
@keyframes pulse {
  50% {
    opacity: 0.55;
  }
}
</style>
