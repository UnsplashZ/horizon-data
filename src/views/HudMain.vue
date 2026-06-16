<script setup lang="ts">
import { computed, onMounted } from "vue";
import { telemetry as t, config, editMode, initShared, gearLabel } from "../telemetry";
import { useOverlayWindow } from "../dragwin";

const BASE_W = 520;
const BASE_H = 150;
const { scale, onDragDown, onResizeDown } = useOverlayWindow(BASE_W);
onMounted(initShared);

const SEG = 16;
const rpmPct = computed(() =>
  t.value && t.value.max_rpm > 0 ? Math.min(1, Math.max(0, t.value.rpm / t.value.max_rpm)) : 0,
);
const litSegments = computed(() => Math.round(rpmPct.value * SEG));
const nearRedline = computed(() => rpmPct.value > 0.93);
function segColor(i: number): string {
  const p = i / SEG;
  if (p < 0.6) return "#34e39a";
  if (p < 0.84) return "#ffd166";
  return "#ff4d4d";
}
const speedDisplay = computed(() => {
  if (!t.value) return 0;
  return Math.round(config.units === "mph" ? t.value.speed_kmh * 0.621371 : t.value.speed_kmh);
});
const speedUnit = computed(() => (config.units === "mph" ? "MPH" : "KM/H"));
const dimmed = computed(() => t.value != null && !t.value.is_race_on);
const bg = computed(() => ({ background: `rgba(15, 18, 25, ${config.bg_opacity})` }));
</script>

<template>
  <div class="win" :class="{ editing: editMode, dim: dimmed }" :style="bg" @pointerdown="onDragDown">
    <div class="scaler" :style="{ transform: `scale(${scale})`, width: BASE_W + 'px', height: BASE_H + 'px' }">
      <div class="content" :style="{ opacity: config.fg_opacity }">
        <template v-if="t">
          <div class="shiftlights" :class="{ flash: nearRedline }">
            <span
              v-for="i in SEG"
              :key="i"
              class="seg"
              :style="{
                background: i <= litSegments ? segColor(i - 1) : 'rgba(255,255,255,0.08)',
                boxShadow: i <= litSegments ? `0 0 6px ${segColor(i - 1)}` : 'none',
              }"
            />
          </div>
          <div class="row">
            <div class="cell">
              <div class="num">{{ speedDisplay }}</div>
              <div class="unit">{{ speedUnit }}</div>
            </div>
            <div class="gearwrap">
              <div class="gear" :class="{ redline: nearRedline }">{{ gearLabel(t.gear) }}</div>
            </div>
            <div class="cell">
              <div class="num small" :class="{ redline: nearRedline }">{{ Math.round(t.rpm) }}</div>
              <div class="unit">RPM</div>
            </div>
          </div>
        </template>
        <div v-else class="waiting">
          等待遥测数据…<br />FH6 → Data Out 指向本机 <b>:{{ config.port }}</b>
        </div>
      </div>
    </div>
    <div v-if="editMode" class="resize" @pointerdown="onResizeDown"></div>
  </div>
</template>

<style scoped>
.content {
  padding: 10px 18px 14px;
  justify-content: center;
}
.shiftlights {
  display: flex;
  gap: 3px;
  margin-bottom: 10px;
}
.seg {
  flex: 1;
  height: 7px;
  border-radius: 2px;
  transition: background 0.04s linear, box-shadow 0.04s linear;
}
.shiftlights.flash {
  animation: flash 0.16s steps(1) infinite;
}
@keyframes flash {
  50% {
    opacity: 0.25;
  }
}
.row {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
}
.cell {
  text-align: center;
}
.num {
  font-size: 46px;
  font-weight: 800;
  line-height: 1;
  font-variant-numeric: tabular-nums;
  letter-spacing: -1px;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.7);
}
.num.small {
  font-size: 34px;
  font-weight: 700;
  color: #cfe3f5;
}
.num.redline,
.gear.redline {
  color: #ff5a4d;
}
.unit {
  margin-top: 2px;
  font-size: 11px;
  letter-spacing: 2px;
  color: #8aa0b6;
}
.gearwrap {
  padding: 0 20px;
}
.gear {
  font-size: 62px;
  font-weight: 900;
  line-height: 0.9;
  font-variant-numeric: tabular-nums;
  background: linear-gradient(180deg, #ffffff, #b9d2ea);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  text-shadow: 0 4px 14px rgba(0, 0, 0, 0.6);
}
.gear.redline {
  background: none;
  color: #ff5a4d;
}
.waiting {
  font-size: 13px;
  line-height: 1.7;
  text-align: center;
  color: #cdd9e5;
  margin: auto;
}
</style>
