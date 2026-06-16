<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useDragWindow } from "../dragwin";

const { onPointerDown } = useDragWindow();
onMounted(initShared);

const throttle = computed(() => (t.value ? t.value.accel / 255 : 0));
const brake = computed(() => (t.value ? t.value.brake / 255 : 0));
const steer = computed(() => (t.value ? t.value.steer / 127 : 0)); // -1..1
const steerDeg = computed(() => steer.value * 120);
const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }" :style="bg" @pointerdown="onPointerDown">
    <div class="content" :style="{ opacity: config.fg_opacity }">
      <div class="grid">
        <!-- 踏板 -->
        <div class="pedal">
          <div class="bar"><div class="fill thr" :style="{ height: throttle * 100 + '%' }" /></div>
          <div class="pct">{{ Math.round(throttle * 100) }}</div>
          <div class="mlabel">油门</div>
        </div>
        <div class="pedal">
          <div class="bar"><div class="fill brk" :style="{ height: brake * 100 + '%' }" /></div>
          <div class="pct">{{ Math.round(brake * 100) }}</div>
          <div class="mlabel">刹车</div>
        </div>

        <!-- 方向盘 -->
        <div class="steer">
          <svg viewBox="0 0 100 100" class="wheel" :style="{ transform: `rotate(${steerDeg}deg)` }">
            <circle cx="50" cy="50" r="42" fill="none" stroke="#6fb3ff" stroke-width="7" opacity="0.9" />
            <circle cx="50" cy="50" r="9" fill="#6fb3ff" />
            <line x1="50" y1="50" x2="50" y2="12" stroke="#6fb3ff" stroke-width="6" stroke-linecap="round" />
            <line x1="50" y1="50" x2="18" y2="64" stroke="#6fb3ff" stroke-width="6" stroke-linecap="round" />
            <line x1="50" y1="50" x2="82" y2="64" stroke="#6fb3ff" stroke-width="6" stroke-linecap="round" />
            <circle cx="50" cy="12" r="3.5" fill="#ff5a4d" />
          </svg>
          <div class="steertrack">
            <span class="tick" />
            <span class="dot" :style="{ left: 50 + steer * 50 + '%' }" />
          </div>
          <div class="mlabel">转向</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.content {
  padding: 10px 14px;
  justify-content: center;
}
.grid {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 16px;
}
.pedal {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.bar {
  width: 16px;
  height: 78px;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.1);
  display: flex;
  flex-direction: column-reverse;
  overflow: hidden;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
}
.fill {
  width: 100%;
  transition: height 0.05s linear;
}
.fill.thr {
  background: linear-gradient(0deg, #1f9e6b, #4ff0a8);
}
.fill.brk {
  background: linear-gradient(0deg, #b3362b, #ff7a6e);
}
.pct {
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.steer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.wheel {
  width: 76px;
  height: 76px;
  transition: transform 0.05s linear;
  filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.5));
}
.steertrack {
  position: relative;
  width: 84px;
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.12);
}
.tick {
  position: absolute;
  left: 50%;
  top: -2px;
  width: 1px;
  height: 10px;
  background: rgba(255, 255, 255, 0.4);
}
.dot {
  position: absolute;
  top: 50%;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #6fb3ff;
  transform: translate(-50%, -50%);
  transition: left 0.05s linear;
}
</style>
