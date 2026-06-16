<script setup lang="ts">
import { ref, onMounted, computed } from "vue";

/** 与 Rust 端 Telemetry 结构对应（蛇形命名） */
interface Telemetry {
  is_race_on: boolean;
  rpm: number;
  max_rpm: number;
  speed_kmh: number;
  gear: number;
  accel: number; // 0..255
  brake: number; // 0..255
  steer: number; // -127..127
  accel_x: number; // m/s^2 横向
  accel_z: number; // m/s^2 纵向
  tire_slip: number[]; // 四轮 combined slip
}

const t = ref<Telemetry | null>(null);
const connected = ref(false);

function gearLabel(g: number): string {
  if (g === 0) return "R";
  if (g === 11) return "N";
  return String(g);
}

// 转速占比，用于换挡灯/进度
const rpmPct = computed(() => {
  if (!t.value || t.value.max_rpm <= 0) return 0;
  return Math.max(0, Math.min(1, t.value.rpm / t.value.max_rpm));
});
const nearRedline = computed(() => rpmPct.value > 0.92);

onMounted(async () => {
  // 仅在 Tauri webview 中可用；普通浏览器预览时降级为“等待”。
  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<Telemetry>("telemetry", (e) => {
      t.value = e.payload;
      connected.value = true;
    });
  } catch {
    // 非 Tauri 环境（如纯 vite 预览），忽略。
  }
});
</script>

<template>
  <div class="overlay">
    <div class="hud" :class="{ dim: t && !t.is_race_on }">
      <template v-if="t">
        <div class="metric">
          <div class="label">RPM</div>
          <div class="value" :class="{ redline: nearRedline }">{{ Math.round(t.rpm) }}</div>
          <div class="rpmbar"><div class="rpmfill" :style="{ width: rpmPct * 100 + '%' }"></div></div>
        </div>

        <div class="gear" :class="{ redline: nearRedline }">{{ gearLabel(t.gear) }}</div>

        <div class="metric">
          <div class="label">KM/H</div>
          <div class="value">{{ Math.round(t.speed_kmh) }}</div>
        </div>
      </template>

      <div v-else class="waiting">
        等待遥测数据…<br />
        在 FH6 → Data Out 指向本机 <b>:5300</b>，进入漫游/比赛驾驶即可。
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 24px;
  /* 全局透明度：后续做成可调滑块，默认 ~0.9 */
  opacity: 0.92;
}

.hud {
  display: flex;
  align-items: center;
  gap: 28px;
  padding: 14px 26px;
  border-radius: 18px;
  background: rgba(18, 20, 24, 0.42);
  backdrop-filter: blur(2px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35);
  transition: opacity 0.3s;
}
.hud.dim {
  opacity: 0.45; /* IsRaceOn=0（菜单/暂停）时淡化 */
}

.metric {
  text-align: center;
  min-width: 110px;
}
.label {
  font-size: 12px;
  letter-spacing: 2px;
  color: #9fb4c8;
}
.value {
  font-size: 44px;
  font-weight: 700;
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
  text-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
}
.value.redline {
  color: #ff5a4d;
}

.gear {
  font-size: 72px;
  font-weight: 800;
  min-width: 80px;
  text-align: center;
  font-variant-numeric: tabular-nums;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.7);
}
.gear.redline {
  color: #ff5a4d;
}

.rpmbar {
  margin-top: 6px;
  height: 6px;
  width: 100%;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.15);
  overflow: hidden;
}
.rpmfill {
  height: 100%;
  background: linear-gradient(90deg, #36d399, #ffd166 70%, #ff5a4d);
  transition: width 0.05s linear;
}

.waiting {
  font-size: 14px;
  line-height: 1.6;
  color: #cdd9e5;
  text-align: center;
}
</style>
