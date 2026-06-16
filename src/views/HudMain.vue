<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 800;
const BASE_H = 200;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(0, 0, 0, ${config.bg_opacity})` }));

const SEG = 16;
const rpmPct = computed(() =>
  t.value && t.value.max_rpm > 0 ? Math.min(1, Math.max(0, t.value.rpm / t.value.max_rpm)) : 0,
);
const litSegments = computed(() => Math.round(rpmPct.value * SEG));
const nearRedline = computed(() => rpmPct.value > 0.90);

// 根据实际转速比例标色：60% 绿，85% 黄，100% 红
function segColor(i: number): string {
  const p = i / SEG;
  if (p < 0.6) return "#00ff00"; // 0-60% 绿色
  if (p < 0.85) return "#ffff00"; // 60-85% 黄色
  return "#ff0000"; // 85-100% 红色
}

const speedDisplay = computed(() => {
  if (!t.value) return 0;
  return Math.round(config.units === "mph" ? t.value.speed_kmh * 0.621371 : t.value.speed_kmh);
});
const speedUnit = computed(() => (config.units === "mph" ? "MPH" : "KMH"));

function gearLabel(g: number): string {
  return g === 0 ? "R" : g === 11 ? "N" : String(g);
}

// G力数据（m/s² 转换为 G，限制 ±3G）
const gx = computed(() => {
  const g = (t.value?.accel_x ?? 0) / 9.8;
  return Math.max(-3, Math.min(3, g)).toFixed(2);
});
const gz = computed(() => {
  const g = (t.value?.accel_z ?? 0) / 9.8;
  return Math.max(-3, Math.min(3, g)).toFixed(2);
});
const gTotal = computed(() => {
  const gx = (t.value?.accel_x ?? 0) / 9.8;
  const gz = (t.value?.accel_z ?? 0) / 9.8;
  const total = Math.sqrt(gx * gx + gz * gz);
  return total.toFixed(2);
});
const dotX = computed(() => {
  const g = (t.value?.accel_x ?? 0) / 9.8;
  const clamped = Math.max(-3, Math.min(3, g));
  return 50 + (clamped / 3) * 45;
});
const dotY = computed(() => {
  const g = (t.value?.accel_z ?? 0) / 9.8;
  const clamped = Math.max(-3, Math.min(3, g));
  return 50 - (clamped / 3) * 45;
});

// 胎温
function fahrenheitToCelsius(f: number): number {
  return (f - 32) * 5 / 9;
}
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
const temps = computed(() =>
  (t.value?.tire_temp ?? [0, 0, 0, 0]).map(f => Math.round(fahrenheitToCelsius(f)))
);

// 抓地力
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

const tires = [
  { i: 0, label: "FL" },
  { i: 1, label: "FR" },
  { i: 2, label: "RL" },
  { i: 3, label: "RR" },
];

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
        <template v-if="t">
          <!-- 顶部换挡灯条 -->
          <div class="shift-bar" :title="`RPM: ${Math.round(t.rpm)}/${Math.round(t.max_rpm)} (${(rpmPct * 100).toFixed(1)}%) nearRedline: ${nearRedline}`">
            <div
              v-for="i in SEG"
              :key="i"
              class="shift-led"
              :class="{ lit: i <= litSegments, flash: nearRedline }"
              :style="{ background: i <= litSegments ? segColor(i - 1) : '#1a1a1a' }"
            ></div>
          </div>

          <!-- 主面板 -->
          <div class="main-panel">
            <!-- 左侧面板：轮胎信息 -->
            <div v-if="config.show_tires" class="side-panel tires">
              <div class="panel-title">TIRES</div>
              <div class="tire-grid-compact">
                <div v-for="tire in tires" :key="tire.i" class="tire-compact">
                  <div class="tire-label-compact">{{ tire.label }}</div>
                  <div class="tire-bar-compact" :style="{ background: gripColor(slips[tire.i]) }"
                    :class="{ slip: slipping(slips[tire.i]) }"></div>
                  <div class="tire-temp-compact" :style="{ background: tempColor(temps[tire.i]) }">
                    {{ temps[tire.i] }}°
                  </div>
                </div>
              </div>
            </div>

            <!-- 油门 -->
            <div v-if="config.show_inputs" class="input-side">
              <div class="input-label">THR</div>
              <div class="input-bar-v">
                <div class="input-fill-v thr" :style="{ height: Math.round((t.accel / 255) * 100) + '%' }"></div>
              </div>
              <div class="input-value">{{ Math.round((t.accel / 255) * 100) }}</div>
            </div>

            <!-- 中央：速度 + 档位 + 转速 + 转向 -->
            <div class="center-panel">
              <div class="speed-block">
                <div class="speed-value">{{ String(speedDisplay).padStart(3, '0') }}</div>
                <div class="speed-label">{{ speedUnit }}</div>
              </div>
              <div class="gear-display" :class="{ redline: nearRedline }">
                {{ gearLabel(t.gear) }}
              </div>
              <div class="rpm-display" :class="{ redline: nearRedline }">
                <span class="rpm-label">RPM</span>
                <span class="rpm-value">{{ Math.round(t.rpm) }}</span>
              </div>
              <!-- 转向条 -->
              <div v-if="config.show_inputs" class="steer-bar">
                <div class="steer-indicator" :style="{ left: `calc(50% + ${steerAngle / 9}%)` }"></div>
              </div>
            </div>

            <!-- 刹车 -->
            <div v-if="config.show_inputs" class="input-side">
              <div class="input-label">BRK</div>
              <div class="input-bar-v">
                <div class="input-fill-v brk" :style="{ height: Math.round((t.brake / 255) * 100) + '%' }"></div>
              </div>
              <div class="input-value">{{ Math.round((t.brake / 255) * 100) }}</div>
            </div>

            <!-- 右侧面板：G力 -->
            <div v-if="config.show_gforce" class="side-panel gforce">
              <div class="panel-title">G-FORCE</div>
              <div class="g-circle-compact">
                <svg viewBox="0 0 100 100" class="g-grid">
                  <circle cx="50" cy="50" r="45" fill="none" stroke="#333" stroke-width="1"/>
                  <circle cx="50" cy="50" r="30" fill="none" stroke="#222" stroke-width="1"/>
                  <line x1="5" y1="50" x2="95" y2="50" stroke="#222" stroke-width="1"/>
                  <line x1="50" y1="5" x2="50" y2="95" stroke="#222" stroke-width="1"/>
                </svg>
                <div class="g-dot-compact" :style="{ left: dotX + '%', top: dotY + '%' }"></div>
              </div>
              <div class="g-values-compact">
                <div class="g-row-compact"><span>TOTAL G</span><span>{{ gTotal }}</span></div>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="waiting">
          <div class="waiting-text">WAITING FOR TELEMETRY</div>
          <div class="waiting-port">PORT {{ config.port }}</div>
        </div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.editing .content {
  cursor: move;
}

/* 换挡灯条 */
.shift-bar {
  display: flex;
  gap: 3px;
  height: 20px;
  padding: 0 14px;
}
.shift-led {
  flex: 1;
  background: #1a1a1a;
  transition: background 0.05s ease;
  border: 1px solid #0a0a0a;
}
.shift-led.lit {
  box-shadow: 0 0 8px currentColor;
  border: 1px solid rgba(255, 255, 255, 0.3);
}
.shift-led.flash {
  animation: led-flash 0.1s steps(2) infinite;
}
@keyframes led-flash {
  50% { opacity: 0.2; }
}

/* 主面板 */
.main-panel {
  display: flex;
  gap: 12px;
  align-items: center;
  flex: 1;
}

/* 油门/刹车竖条 */
.input-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 40px;
  background: #0a0a0a;
  padding: 8px 6px;
}

.input-label {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #666;
}

.input-bar-v {
  width: 100%;
  height: 120px;
  background: #0a0a0a;
  position: relative;
  display: flex;
  align-items: flex-end;
}

.input-fill-v {
  width: 100%;
  transition: height 0.05s linear;
}

.input-fill-v.thr {
  background: #00ff00;
}

.input-fill-v.brk {
  background: #ff0000;
}

.input-value {
  font-size: 11px;
  font-weight: 900;
  font-family: "Consolas", monospace;
  color: #ffffff;
}

.side-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: #0a0a0a;
  padding: 8px;
  width: 140px;
  height: 140px;
}

.panel-title {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #666;
  text-align: center;
}

/* 轮胎紧凑视图 */
.tire-grid-compact {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
  flex: 1;
}

.tire-compact {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.tire-label-compact {
  font-size: 8px;
  font-weight: 700;
  color: #666;
}

.tire-bar-compact {
  width: 100%;
  height: 10px;
}

.tire-bar-compact.slip {
  animation: slip-flash 0.15s steps(2) infinite;
}

@keyframes slip-flash {
  50% { opacity: 0.3; }
}

.tire-temp-compact {
  width: 100%;
  padding: 2px;
  text-align: center;
  font-size: 10px;
  font-weight: 700;
  color: #000;
}

/* 中央速度+档位 */
.center-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.speed-block {
  background: #0a0a0a;
  padding: 8px 16px;
  text-align: center;
}

.speed-value {
  font-size: 48px;
  font-weight: 900;
  line-height: 1;
  font-family: "Consolas", monospace;
  color: #ffffff;
  letter-spacing: 2px;
}

.speed-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #888;
  margin-top: 2px;
}

.gear-display {
  font-size: 64px;
  font-weight: 900;
  line-height: 0.9;
  font-family: "Consolas", monospace;
  color: #ffffff;
}

.gear-display.redline {
  color: #ff0000;
  animation: gear-pulse 0.2s ease-in-out infinite;
}

@keyframes gear-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.08); }
}

.rpm-display {
  display: flex;
  align-items: baseline;
  gap: 8px;
  background: #0a0a0a;
  padding: 4px 16px;
}

.rpm-display.redline .rpm-value {
  color: #ff0000;
  animation: rpm-pulse 0.2s ease-in-out infinite;
}

@keyframes rpm-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}

.rpm-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #888;
}

.rpm-value {
  font-size: 18px;
  font-weight: 900;
  font-family: "Consolas", monospace;
  color: #ffffff;
  letter-spacing: 1px;
}

/* 转向条 */
.steer-bar {
  width: 180px;
  height: 8px;
  background: #0a0a0a;
  position: relative;
  margin-top: 4px;
}

.steer-indicator {
  position: absolute;
  top: 0;
  width: 4px;
  height: 100%;
  background: #ffffff;
  transform: translateX(-50%);
  transition: left 0.06s linear;
}

/* G力紧凑视图 */
.g-circle-compact {
  position: relative;
  width: 100px;
  height: 100px;
  background: #0a0a0a;
  border-radius: 50%;
  margin: 0 auto;
  flex-shrink: 0;
}

.g-grid {
  width: 100%;
  height: 100%;
}

.g-dot-compact {
  position: absolute;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ff0000;
  transform: translate(-50%, -50%);
  transition: all 0.05s linear;
  border: 2px solid #fff;
}

.g-values-compact {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.g-row-compact {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
  color: #fff;
  padding: 2px 4px;
  background: #0a0a0a;
}

/* 等待状态 */
.waiting {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 6px;
}

.waiting-text {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #888;
}

.waiting-port {
  font-size: 11px;
  font-weight: 700;
  color: #555;
}
</style>
