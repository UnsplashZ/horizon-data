<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useDragWindow } from "../dragwin";

const { onPointerDown } = useDragWindow();
onMounted(initShared);

const G = 9.81;
const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));

const clamp = (v: number) => Math.max(-1, Math.min(1, v / (2 * G))); // ±2g 满量程
const fmtG = (v: number) => (Math.abs(v) < 0.05 ? "0.0" : v.toFixed(1)); // 避免 -0.0
const latG = computed(() => fmtG(t.value ? t.value.accel_x / G : 0));
const lonG = computed(() => fmtG(t.value ? t.value.accel_z / G : 0));

// 拖尾：保留最近若干点
const trail = ref<{ x: number; y: number }[]>([]);
watch(t, (v) => {
  if (!v) return;
  trail.value.push({ x: 50 + clamp(v.accel_x) * 45, y: 50 - clamp(v.accel_z) * 45 });
  if (trail.value.length > 18) trail.value.shift();
});
const dot = computed(() => ({
  x: t.value ? 50 + clamp(t.value.accel_x) * 45 : 50,
  y: t.value ? 50 - clamp(t.value.accel_z) * 45 : 50,
}));
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }" :style="bg" @pointerdown="onPointerDown">
    <div class="content" :style="{ opacity: config.fg_opacity }">
      <svg viewBox="0 0 100 100" class="gg">
        <circle cx="50" cy="50" r="45" fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.18)" stroke-width="1" />
        <circle cx="50" cy="50" r="22.5" fill="none" stroke="rgba(255,255,255,0.14)" stroke-width="1" stroke-dasharray="2 2" />
        <line x1="50" y1="6" x2="50" y2="94" stroke="rgba(255,255,255,0.12)" stroke-width="1" />
        <line x1="6" y1="50" x2="94" y2="50" stroke="rgba(255,255,255,0.12)" stroke-width="1" />
        <text x="50" y="20" class="rl">1g</text>
        <!-- 拖尾 -->
        <circle
          v-for="(p, idx) in trail"
          :key="idx"
          :cx="p.x"
          :cy="p.y"
          r="2.2"
          fill="#ffd166"
          :opacity="(idx + 1) / trail.length * 0.5"
        />
        <!-- 当前点 -->
        <circle :cx="dot.x" :cy="dot.y" r="4.5" fill="#ffd166" stroke="#fff" stroke-width="0.8" />
      </svg>
      <div class="readout">
        <span>横 <b>{{ latG }}</b>g</span>
        <span>纵 <b>{{ lonG }}</b>g</span>
      </div>
      <div class="mlabel">G力</div>
    </div>
  </div>
</template>

<style scoped>
.content {
  padding: 8px 10px 6px;
  align-items: center;
  justify-content: center;
  gap: 5px;
}
.gg {
  width: 116px;
  height: 116px;
  filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.4));
}
.rl {
  fill: rgba(255, 255, 255, 0.4);
  font-size: 7px;
  text-anchor: middle;
}
.readout {
  display: flex;
  gap: 14px;
  font-size: 12px;
  color: #cfe3f5;
  font-variant-numeric: tabular-nums;
}
.readout b {
  font-size: 14px;
  color: #fff;
}
</style>
