<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

// 基准画布尺寸与 design-preview-halo.html 的 viewBox 一致
const BASE_W = 660;
const BASE_H = 204;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W, BASE_H);
onMounted(initShared);

const dimmed = computed(() => t.value != null && !t.value.is_race_on);

// ---- 转速分区配色（绿→黄→橙→红插值），换挡灯 / 渐变弧 / 外发光共用 ----
const RPM_STOPS: [number, number, number][] = [
  [31, 223, 95],
  [255, 224, 0],
  [255, 138, 0],
  [255, 45, 45],
];
function lerp(a: number, b: number, f: number): number {
  return Math.round(a + (b - a) * f);
}
function revColor(p: number): string {
  p = Math.min(1, Math.max(0, p));
  const s = p * 3;
  const i = Math.min(2, Math.floor(s));
  const f = s - i;
  const a = RPM_STOPS[i];
  const b = RPM_STOPS[i + 1];
  return `rgb(${lerp(a[0], b[0], f)}, ${lerp(a[1], b[1], f)}, ${lerp(a[2], b[2], f)})`;
}
function colorRamp(p: number, from: [number, number, number], to: [number, number, number]): string {
  p = Math.min(1, Math.max(0, p));
  return `rgb(${lerp(from[0], to[0], p)}, ${lerp(from[1], to[1], p)}, ${lerp(from[2], to[2], p)})`;
}

// ---- 转速渐变弧几何（左圆环） ----
const CX = 190;
const CY = 104;
const ARC_R = 54;
const A0 = 135;
const SPAN = 270;
const ARC_SEG = 256;
const INPUT_CX = 470;
const INPUT_CY = 104;
const INPUT_R = 54;
const INPUT_SEG = 192;
const INPUT_TOP_GAP = 8;
const INPUT_SPAN = (SPAN - INPUT_TOP_GAP) / 2;
const BODY_PATH =
  "M220.4,38.7 C232,44.1 246,55 258,55 L402,55 C414,55 428,44.1 439.6,38.7 " +
  "A72,72 0 1 1 439.6,169.3 C429,164.4 420,164 408,164 " +
  "L252,164 C240,164 231,164.4 220.4,169.3 " +
  "A72,72 0 1 1 220.4,38.7 Z";

function arcPt(deg: number): [number, number] {
  const r = (deg * Math.PI) / 180;
  return [CX + ARC_R * Math.cos(r), CY + ARC_R * Math.sin(r)];
}
function polarPt(cx: number, cy: number, radius: number, deg: number): [number, number] {
  const r = (deg * Math.PI) / 180;
  return [cx + radius * Math.cos(r), cy + radius * Math.sin(r)];
}
function arcSegFor(cx: number, cy: number, radius: number, a1: number, a2: number): string {
  const [x1, y1] = polarPt(cx, cy, radius, a1);
  const [x2, y2] = polarPt(cx, cy, radius, a2);
  const large = Math.abs(a2 - a1) > 180 ? 1 : 0;
  const sweep = a2 >= a1 ? 1 : 0;
  return `M${x1},${y1} A${radius},${radius} 0 ${large} ${sweep} ${x2},${y2}`;
}
function arcSeg(a1: number, a2: number): string {
  return arcSegFor(CX, CY, ARC_R, a1, a2);
}
const arcTrack = arcSeg(A0, A0 + SPAN);
const arcBars = Array.from({ length: ARC_SEG }, (_, i) => {
  const gap = 0.08;
  const a1 = A0 + (SPAN * i) / ARC_SEG + gap / 2;
  const a2 = A0 + (SPAN * (i + 1)) / ARC_SEG - gap / 2;
  return { d: arcSeg(a1, a2), color: revColor(i / (ARC_SEG - 1)) };
});
const brakeTrack = arcSegFor(INPUT_CX, INPUT_CY, INPUT_R, -90 - INPUT_TOP_GAP / 2, -90 - INPUT_TOP_GAP / 2 - INPUT_SPAN);
const throttleTrack = arcSegFor(INPUT_CX, INPUT_CY, INPUT_R, -90 + INPUT_TOP_GAP / 2, -90 + INPUT_TOP_GAP / 2 + INPUT_SPAN);
const brakeBars = Array.from({ length: INPUT_SEG }, (_, i) => {
  const gap = 0.12;
  const start = -90 - INPUT_TOP_GAP / 2;
  const bottom = start - INPUT_SPAN;
  const a1 = bottom + (INPUT_SPAN * i) / INPUT_SEG + gap / 2;
  const a2 = bottom + (INPUT_SPAN * (i + 1)) / INPUT_SEG - gap / 2;
  return {
    d: arcSegFor(INPUT_CX, INPUT_CY, INPUT_R, a1, a2),
    color: colorRamp(i / (INPUT_SEG - 1), [255, 127, 120], [255, 59, 48]),
  };
});
const throttleBars = Array.from({ length: INPUT_SEG }, (_, i) => {
  const gap = 0.12;
  const start = -90 + INPUT_TOP_GAP / 2;
  const bottom = start + INPUT_SPAN;
  const a1 = bottom - (INPUT_SPAN * i) / INPUT_SEG - gap / 2;
  const a2 = bottom - (INPUT_SPAN * (i + 1)) / INPUT_SEG + gap / 2;
  return {
    d: arcSegFor(INPUT_CX, INPUT_CY, INPUT_R, a1, a2),
    color: colorRamp(i / (INPUT_SEG - 1), [86, 242, 160], [0, 230, 118]),
  };
});

// ---- 换挡灯 ----
const SEG = 13;
const ledColors = Array.from({ length: SEG }, (_, i) => revColor(i / (SEG - 1)));

// ---- 转速派生量 ----
const rpmPct = computed(() =>
  t.value && t.value.max_rpm > 0 ? Math.min(1, Math.max(0, t.value.rpm / t.value.max_rpm)) : 0,
);
const litBars = computed(() => Math.round(rpmPct.value * ARC_SEG));
const litLeds = computed(() => Math.round(rpmPct.value * SEG));
const redline = computed(() => rpmPct.value >= 0.9);
const panelOpacity = computed(() => config.bg_opacity);
const panelSoftOpacity = computed(() => Math.min(0.34, config.bg_opacity * 0.82));
const panelCoreOpacity = computed(() => config.bg_opacity * 0.5);

// ---- 速度 / 档位 ----
const speedDisplay = computed(() => {
  if (!t.value) return 0;
  return Math.round(config.units === "mph" ? t.value.speed_kmh * 0.621371 : t.value.speed_kmh);
});
const speedUnit = computed(() => (config.units === "mph" ? "MPH" : "KMH"));
function gearLabel(g: number): string {
  return g === 0 ? "R" : g === 11 ? "N" : String(g);
}

// ---- 油门 / 刹车 / 转向 ----
const throttle = computed(() => (t.value ? Math.round((t.value.accel / 255) * 100) : 0));
const brake = computed(() => (t.value ? Math.round((t.value.brake / 255) * 100) : 0));
const litBrakeBars = computed(() => Math.round((brake.value / 100) * INPUT_SEG));
const litThrottleBars = computed(() => Math.round((throttle.value / 100) * INPUT_SEG));
const steerOffset = computed(() => {
  const s = t.value ? Math.max(-1, Math.min(1, t.value.steer / 127)) : 0;
  return s * 47.8;
});

// ---- 轮胎：底色按抓地（tire_slip），数字显示胎温（°C） ----
function fahrenheitToCelsius(f: number): number {
  return ((f - 32) * 5) / 9;
}
// 正常绿 (52,227,150) → 完全失去抓地红 (255,67,70)；tire_slip≈1.2 视为彻底打滑
function gripColor(slip: number): string {
  const s = Math.min(1, Math.max(0, slip / 1.2));
  const r = Math.round(52 + s * 203);
  const g = Math.round(227 - s * 160);
  const b = Math.round(150 - s * 80);
  return `rgb(${r}, ${g}, ${b})`;
}
const temps = computed(() =>
  (t.value?.tire_temp ?? [0, 0, 0, 0]).map((f) => Math.round(fahrenheitToCelsius(f))),
);
const slips = computed(() => t.value?.tire_slip ?? [0, 0, 0, 0]);

// ---- G 力 ----
const gTotal = computed(() => {
  const x = (t.value?.accel_x ?? 0) / 9.8;
  const z = (t.value?.accel_z ?? 0) / 9.8;
  return Math.sqrt(x * x + z * z).toFixed(1);
});
const dotX = computed(() => {
  const g = (t.value?.accel_x ?? 0) / 9.8;
  return 50 + Math.max(-1, Math.min(1, g / 1.5)) * 42;
});
const dotY = computed(() => {
  const g = (t.value?.accel_z ?? 0) / 9.8;
  return 50 - Math.max(-1, Math.min(1, g / 1.5)) * 42;
});
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }">
    <div
      class="scaler"
      :style="{
        transform: `scale(${scale})`,
        width: BASE_W + 'px',
        height: BASE_H + 'px',
        transformOrigin: 'top left',
      }"
    >
      <div class="wrap" :style="{ opacity: config.fg_opacity }" @pointerdown="onDragDown">
        <template v-if="t">
          <svg class="layer" :viewBox="`0 0 ${BASE_W} ${BASE_H}`">
            <defs>
              <filter id="soft-frame-blur" x="-18%" y="-26%" width="136%" height="152%">
                <feGaussianBlur stdDeviation="6.5" />
              </filter>
              <filter id="panel-soft-edge" x="-10%" y="-18%" width="120%" height="136%">
                <feGaussianBlur stdDeviation="2.4" />
              </filter>
              <radialGradient
                id="body-dark-scrim"
                gradientUnits="userSpaceOnUse"
                cx="330"
                cy="116"
                r="250"
                gradientTransform="translate(330 116) scale(1 .46) translate(-330 -116)"
              >
                <stop offset="0" stop-color="#000000" stop-opacity=".48" />
                <stop offset=".56" stop-color="#000000" stop-opacity=".25" />
                <stop offset="1" stop-color="#000000" stop-opacity="0" />
              </radialGradient>
              <radialGradient id="pod-dark-scrim">
                <stop offset="0" stop-color="#000000" stop-opacity=".42" />
                <stop offset=".66" stop-color="#000000" stop-opacity=".22" />
                <stop offset="1" stop-color="#000000" stop-opacity="0" />
              </radialGradient>
            </defs>
            <!-- Soft Frame：关闭主体外轮廓泛光，只保留软背板 -->
            <!-- 半透明主体，透明度受 bg_opacity 控制 -->
            <path :d="BODY_PATH" fill="#11161e" :opacity="panelSoftOpacity" filter="url(#panel-soft-edge)" />
            <path :d="BODY_PATH" fill="url(#body-dark-scrim)" opacity=".5" filter="url(#panel-soft-edge)" />
            <path :d="BODY_PATH" fill="#11161e" :opacity="panelCoreOpacity" />
            <!-- 转速渐变弧 -->
            <g>
              <path :d="arcTrack" fill="none" stroke="#20242c" stroke-width="6.25" stroke-linecap="round" opacity=".8" />
              <g :class="{ flash: redline }">
                <path
                  v-for="(bar, i) in arcBars"
                  :key="i"
                  :d="bar.d"
                  fill="none"
                  :stroke="bar.color"
                  stroke-width="6.25"
                  stroke-linecap="round"
                  :style="{ opacity: i < litBars ? 1 : 0 }"
                />
              </g>
            </g>
            <!-- 右圆输入弧：左刹车 / 右油门 -->
            <g v-if="config.show_inputs">
              <path :d="brakeTrack" fill="none" stroke="#20242c" stroke-width="6.25" stroke-linecap="round" opacity=".72" />
              <path :d="throttleTrack" fill="none" stroke="#20242c" stroke-width="6.25" stroke-linecap="round" opacity=".72" />
              <path
                v-for="(bar, i) in brakeBars"
                :key="`brake-${i}`"
                :d="bar.d"
                fill="none"
                :stroke="bar.color"
                stroke-width="6.25"
                stroke-linecap="round"
                :style="{ opacity: i < litBrakeBars ? 1 : 0 }"
              />
              <path
                v-for="(bar, i) in throttleBars"
                :key="`throttle-${i}`"
                :d="bar.d"
                fill="none"
                :stroke="bar.color"
                stroke-width="6.25"
                stroke-linecap="round"
                :style="{ opacity: i < litThrottleBars ? 1 : 0 }"
              />
            </g>
            <!-- 侧舱圆环：左轮胎 / 右 G 力 -->
            <g>
              <template v-if="config.show_tires">
                <circle cx="58" cy="104" r="38" fill="#11161e" :opacity="panelOpacity" />
                <circle cx="58" cy="104" r="38" fill="url(#pod-dark-scrim)" opacity=".78" />
                <circle cx="58" cy="104" r="38" fill="none" stroke="#96a8bd" stroke-width="5" opacity=".12" filter="url(#soft-frame-blur)" />
              </template>
              <template v-if="config.show_gforce">
                <circle cx="602" cy="104" r="38" fill="#11161e" :opacity="panelOpacity" />
                <circle cx="602" cy="104" r="38" fill="url(#pod-dark-scrim)" opacity=".78" />
                <circle cx="602" cy="104" r="38" fill="none" stroke="#96a8bd" stroke-width="5" opacity=".12" filter="url(#soft-frame-blur)" />
              </template>
            </g>
          </svg>

          <!-- 换挡灯 -->
          <div class="ov rev" :class="{ flash: redline }" style="left: 261px; top: 60px">
            <div
              v-for="(c, i) in ledColors"
              :key="i"
              class="led"
              :style="{
                background: i < litLeds ? c : '#1c2128',
                boxShadow: i < litLeds ? `0 0 5px ${c}` : 'none',
              }"
            ></div>
          </div>

          <!-- 转速数字 -->
          <div class="ov rpm-box" :class="{ redline }" style="left: 190px; top: 84px; transform: translateX(-50%)">
            <div class="lbl">RPM</div>
            <div class="v">{{ Math.round(t.rpm) }}</div>
          </div>

          <!-- 档位 -->
          <div class="ov gear" :class="{ redline }" style="left: 290px; top: 73px">{{ gearLabel(t.gear) }}</div>

          <!-- 转向 -->
          <div v-if="config.show_inputs" class="ov center-steer" style="left: 330px; top: 145px; transform: translateX(-50%)">
            <div class="steer-track"><div class="steer-ind" :style="{ left: `calc(50% + ${steerOffset}%)` }"></div></div>
          </div>

          <!-- 速度 -->
          <div class="ov spd-box" style="left: 470px; top: 84px; transform: translateX(-50%)">
            <div class="lbl">{{ speedUnit }}</div>
            <div class="v">{{ speedDisplay }}</div>
          </div>

          <!-- 左舱：轮胎 -->
          <div
            v-if="config.show_tires"
            class="ov tires"
            style="left: 58px; top: 104px; transform: translate(-50%, -50%)"
          >
            <div
              v-for="i in 4"
              :key="i"
              class="tt"
              :style="{ background: gripColor(slips[i - 1]) }"
            >
              {{ temps[i - 1] }}
            </div>
          </div>

          <!-- 右舱：G 力 -->
          <div
            v-if="config.show_gforce"
            class="ov gpod"
            style="left: 602px; top: 104px; transform: translate(-50%, -50%)"
          >
            <div class="gbox">
              <div class="ln v"></div>
              <div class="ln h"></div>
              <div class="gdot" :style="{ left: dotX + '%', top: dotY + '%' }"></div>
            </div>
            <div class="gval">{{ gTotal }} G</div>
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
.win {
  width: 100%;
  height: 100%;
  font-family: "Rajdhani", "Consolas", -apple-system, sans-serif;
  color: #fff;
  user-select: none;
}
.editing .wrap {
  cursor: move;
}

.wrap {
  position: relative;
  width: 660px;
  height: 204px;
}
.layer {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}
.ov {
  position: absolute;
}

/* 红线闪烁 */
.flash {
  animation: hud-flash 0.1s steps(2) infinite;
}
@keyframes hud-flash {
  50% {
    opacity: 0.2;
  }
}

/* 换挡灯 */
.rev {
  display: flex;
  gap: 5px;
}
.led {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #1c2128;
  transition: background 0.05s, box-shadow 0.05s;
}

/* 转速数字 */
.rpm-box {
  text-align: center;
  width: 112px;
}
.rpm-box .lbl,
.spd-box .lbl {
  display: block;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 3px;
  text-indent: 3px;
  color: #9aa4b2;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.65);
}
.rpm-box .v {
  font-size: 30px;
  font-weight: 700;
  color: #fff;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  transition: color 0.1s;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.72), 0 0 1px rgba(0, 0, 0, 0.85);
}
.rpm-box.redline .v {
  color: #ff3b30;
}

/* 档位 */
.gear {
  width: 80px;
  text-align: center;
  font-size: 76px;
  font-weight: 700;
  line-height: 1;
  color: #fff;
  transition: color 0.1s;
}
.gear.redline {
  color: #ff3b30;
}

/* 速度 */
.spd-box {
  text-align: center;
  width: 112px;
}
.spd-box .v {
  font-size: 34px;
  font-weight: 700;
  color: #fff;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.72), 0 0 1px rgba(0, 0, 0, 0.85);
}
.steer-track {
  margin: 7px auto 0;
  width: 74px;
  height: 5px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  position: relative;
}
.steer-ind {
  position: absolute;
  top: -1px;
  width: 5px;
  height: 7px;
  background: #19e0ff;
  border-radius: 2px;
  transform: translateX(-50%);
  transition: left 0.06s linear;
}
/* 中央转向条 */
.center-steer {
  width: 122px;
  text-align: center;
}
.center-steer .steer-track {
  width: 112px;
}

/* 轮胎 */
.tires {
  display: grid;
  grid-template-columns: 17px 17px;
  gap: 3px;
}
.tt {
  font-size: 11px;
  font-weight: 700;
  color: #06140d;
  text-align: center;
  border-radius: 3px;
  padding: 2px 0;
  background: #34e39a;
}

/* G 力 */
.gpod {
  width: 76px;
  height: 76px;
  text-align: center;
}
.gbox {
  position: absolute;
  left: 50%;
  top: 50%;
  width: 36px;
  height: 36px;
  transform: translate(-50%, -50%);
}
.gbox .ln {
  position: absolute;
  background: rgba(255, 255, 255, 0.1);
}
.gbox .ln.v {
  left: 50%;
  top: 0;
  bottom: 0;
  width: 1px;
}
.gbox .ln.h {
  top: 50%;
  left: 0;
  right: 0;
  height: 1px;
}
.gdot {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #19e0ff;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 6px #19e0ff;
  transition: all 0.05s linear;
}
.gval {
  position: absolute;
  left: 0;
  right: 0;
  top: 58px;
  font-size: 10px;
  line-height: 1;
  font-weight: 700;
  letter-spacing: 0.8px;
  color: #aeb7c4;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.65);
}

/* 等待状态 */
.waiting {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.waiting-text {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 1px;
  color: #8a929d;
}
.waiting-port {
  font-size: 11px;
  font-weight: 700;
  color: #5f6b7a;
}

/* 缩放手柄 */
.resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 18px;
  height: 18px;
  cursor: nwse-resize;
  background: linear-gradient(135deg, transparent 50%, rgba(255, 255, 255, 0.4) 50%);
}
</style>
