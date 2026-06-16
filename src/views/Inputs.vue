<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 240;
const BASE_H = 180;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(0, 0, 0, ${config.bg_opacity})` }));

const throttle = computed(() => (t.value ? Math.round((t.value.accel / 255) * 100) : 0));
const brake = computed(() => (t.value ? Math.round((t.value.brake / 255) * 100) : 0));
const steer = computed(() => {
  if (!t.value) return 50;
  return 50 + (t.value.steer / 127) * 50;
});
const steerAngle = computed(() => (t.value ? (t.value.steer / 127) * 450 : 0));
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
        <div class="title">INPUTS</div>

        <!-- 油门和刹车 -->
        <div class="pedals">
          <div class="pedal">
            <div class="pedal-label">THR</div>
            <div class="pedal-bar">
              <div class="pedal-fill thr" :style="{ height: throttle + '%' }"></div>
            </div>
            <div class="pedal-value">{{ throttle }}</div>
          </div>

          <div class="pedal">
            <div class="pedal-label">BRK</div>
            <div class="pedal-bar">
              <div class="pedal-fill brk" :style="{ height: brake + '%' }"></div>
            </div>
            <div class="pedal-value">{{ brake }}</div>
          </div>
        </div>

        <!-- 方向盘 -->
        <div class="steering">
          <div class="steer-icon" :style="{ transform: `rotate(${steerAngle}deg)` }">
            <svg viewBox="0 0 80 80">
              <circle cx="40" cy="40" r="36" fill="none" stroke="#00ffff" stroke-width="3" opacity="0.3"/>
              <circle cx="40" cy="40" r="28" fill="none" stroke="#00ffff" stroke-width="2" opacity="0.2"/>
              <line x1="40" y1="8" x2="40" y2="20" stroke="#ff00ff" stroke-width="4" stroke-linecap="round"/>
              <circle cx="40" cy="40" r="8" fill="#00ffff" opacity="0.6"/>
            </svg>
          </div>
          <div class="steer-bar">
            <div class="steer-center"></div>
            <div class="steer-dot" :style="{ left: steer + '%' }"></div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
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

/* 踏板显示 */
.pedals {
  display: flex;
  justify-content: center;
  gap: 24px;
}

.pedal {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.pedal-label {
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 2px;
  color: rgba(255, 255, 255, 0.5);
  font-family: "Consolas", monospace;
}

.pedal-bar {
  width: 22px;
  height: 120px;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column-reverse;
  overflow: hidden;
  clip-path: polygon(20% 0%, 80% 0%, 100% 100%, 0% 100%);
  border: 1px solid rgba(0, 255, 255, 0.2);
  box-shadow: inset 0 4px 8px rgba(0, 0, 0, 0.8);
}

.pedal-fill {
  width: 100%;
  transition: height 0.05s linear;
  position: relative;
}

.pedal-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.3) 0%, transparent 40%);
}

.pedal-fill.thr {
  background: linear-gradient(0deg, rgba(0, 255, 136, 0.4) 0%, #00ff88 100%);
  box-shadow:
    inset 0 -3px 6px rgba(0, 0, 0, 0.4),
    0 0 15px rgba(0, 255, 136, 0.5);
}

.pedal-fill.brk {
  background: linear-gradient(0deg, rgba(255, 0, 64, 0.4) 0%, #ff0040 100%);
  box-shadow:
    inset 0 -3px 6px rgba(0, 0, 0, 0.4),
    0 0 15px rgba(255, 0, 64, 0.5);
}

.pedal-value {
  font-size: 16px;
  font-weight: 900;
  font-family: "Consolas", monospace;
  color: #00ffff;
  text-shadow:
    0 0 10px rgba(0, 255, 255, 0.8),
    0 2px 4px rgba(0, 0, 0, 0.8);
  letter-spacing: 1px;
}

/* 方向盘 */
.steering {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.steer-icon {
  width: 80px;
  height: 80px;
  transition: transform 0.06s linear;
  filter: drop-shadow(0 0 15px rgba(0, 255, 255, 0.5));
}

.steer-bar {
  position: relative;
  width: 100%;
  height: 10px;
  background: rgba(0, 0, 0, 0.6);
  clip-path: polygon(6px 0%, calc(100% - 6px) 0%, 100% 100%, 0% 100%);
  border: 1px solid rgba(255, 0, 255, 0.3);
  box-shadow: inset 0 3px 6px rgba(0, 0, 0, 0.8);
}

.steer-center {
  position: absolute;
  left: 50%;
  top: -4px;
  width: 3px;
  height: 18px;
  background: rgba(255, 255, 0, 0.6);
  transform: translateX(-50%);
  box-shadow: 0 0 8px rgba(255, 255, 0, 0.8);
}

.steer-dot {
  position: absolute;
  top: 50%;
  width: 14px;
  height: 14px;
  border-radius: 2px;
  background: #ff00ff;
  transform: translate(-50%, -50%);
  transition: left 0.06s linear;
  box-shadow:
    0 0 20px rgba(255, 0, 255, 0.8),
    0 3px 6px rgba(0, 0, 0, 0.8),
    inset 0 1px 2px rgba(255, 255, 255, 0.3);
  clip-path: polygon(30% 0%, 70% 0%, 100% 50%, 70% 100%, 30% 100%, 0% 50%);
}
</style>
