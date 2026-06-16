<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";

/** 与 Rust 端 Telemetry 对应 */
interface Telemetry {
  is_race_on: boolean;
  rpm: number;
  max_rpm: number;
  speed_kmh: number;
  gear: number;
  accel: number; // 0..255
  brake: number; // 0..255
  steer: number; // -127..127
  accel_x: number;
  accel_z: number;
  tire_slip: number[];
}
interface Config {
  port: number;
  opacity: number;
  hud_x: number;
  hud_y: number;
  units: string;
}

const t = ref<Telemetry | null>(null);
const editMode = ref(false);
const cfg = reactive<Config>({ port: 5300, opacity: 0.92, hud_x: -1, hud_y: 28, units: "kmh" });

// ---- Tauri 桥接（非 Tauri 环境下降级） ----
type Invoke = <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
let invoke: Invoke | null = null;
async function tauri(): Promise<Invoke | null> {
  if (!invoke) {
    try {
      ({ invoke } = await import("@tauri-apps/api/core"));
    } catch {
      return null;
    }
  }
  return invoke;
}

// ---- 换挡灯条 ----
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

// ---- 显示值 ----
const speedDisplay = computed(() => {
  if (!t.value) return 0;
  return Math.round(cfg.units === "mph" ? t.value.speed_kmh * 0.621371 : t.value.speed_kmh);
});
const speedUnit = computed(() => (cfg.units === "mph" ? "MPH" : "KM/H"));
function gearLabel(g: number): string {
  return g === 0 ? "R" : g === 11 ? "N" : String(g);
}
const dimmed = computed(() => t.value != null && !t.value.is_race_on);

// ---- 配置持久化 ----
async function persist() {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("save_config", { config: { ...cfg } });
    } catch {}
  }
}
async function applyPort() {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("set_port", { port: Number(cfg.port) });
    } catch {}
  }
  persist();
}
async function quit() {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("quit_app");
    } catch {}
  }
}

// ---- 拖动定位 ----
let dragging = false;
let ox = 0;
let oy = 0;
function onDown(e: PointerEvent) {
  if (!editMode.value) return;
  dragging = true;
  ox = e.clientX - cfg.hud_x;
  oy = e.clientY - cfg.hud_y;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}
function onMove(e: PointerEvent) {
  if (!dragging) return;
  cfg.hud_x = Math.round(e.clientX - ox);
  cfg.hud_y = Math.round(e.clientY - oy);
}
function onUp() {
  if (dragging) {
    dragging = false;
    persist();
  }
}

onMounted(async () => {
  const inv = await tauri();
  if (inv) {
    try {
      Object.assign(cfg, await inv<Config>("load_config"));
    } catch {}
  }
  if (cfg.hud_x < 0) cfg.hud_x = Math.round(window.innerWidth / 2 - 250);

  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<Telemetry>("telemetry", (e) => (t.value = e.payload));
    await listen<boolean>("edit-mode", (e) => {
      editMode.value = e.payload;
      if (!e.payload) persist();
    });
  } catch {}
});
</script>

<template>
  <!-- 编辑模式下整窗轻微变暗，提示已解锁可交互 -->
  <div class="stage" :class="{ editing: editMode }">
    <div
      class="hud"
      :class="{ dim: dimmed, editing: editMode }"
      :style="{ left: cfg.hud_x + 'px', top: cfg.hud_y + 'px', opacity: cfg.opacity }"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
    >
      <template v-if="t">
        <!-- 换挡灯条 -->
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

        <!-- 主行：速度 | 档位 | 转速 -->
        <div class="row">
          <div class="cell speed">
            <div class="num">{{ speedDisplay }}</div>
            <div class="unit">{{ speedUnit }}</div>
          </div>

          <div class="cell gearwrap">
            <div class="gear" :class="{ redline: nearRedline }">{{ gearLabel(t.gear) }}</div>
          </div>

          <div class="cell rpm">
            <div class="num small" :class="{ redline: nearRedline }">{{ Math.round(t.rpm) }}</div>
            <div class="unit">RPM</div>
          </div>
        </div>
      </template>

      <div v-else class="waiting">
        等待遥测数据…<br />
        FH6 → Data Out 指向本机 <b>:{{ cfg.port }}</b>，进入驾驶即可
      </div>
    </div>

    <!-- 编辑模式控制面板 -->
    <div v-if="editMode" class="panel">
      <div class="panel-title">horizon-data · 编辑模式</div>
      <label class="field">
        <span>透明度 {{ Math.round(cfg.opacity * 100) }}%</span>
        <input type="range" min="0.25" max="1" step="0.01" v-model.number="cfg.opacity" @change="persist" />
      </label>
      <label class="field">
        <span>端口</span>
        <input type="number" v-model.number="cfg.port" @change="applyPort" />
      </label>
      <label class="field">
        <span>单位</span>
        <select v-model="cfg.units" @change="persist">
          <option value="kmh">km/h</option>
          <option value="mph">mph</option>
        </select>
      </label>
      <div class="hint">拖动 HUD 调整位置 · 再按 ⌘/Ctrl+Shift+H 锁定</div>
      <button class="quit" @click="quit">退出程序</button>
    </div>

    <div v-if="!editMode && !t" class="lockhint">按 ⌘/Ctrl+Shift+H 进入编辑模式</div>
  </div>
</template>

<style scoped>
.stage {
  position: fixed;
  inset: 0;
  transition: background 0.2s;
}
.stage.editing {
  background: rgba(0, 0, 0, 0.18);
  outline: 2px dashed rgba(120, 200, 255, 0.5);
  outline-offset: -2px;
}

/* ---------- HUD ---------- */
.hud {
  position: absolute;
  width: 500px;
  padding: 10px 18px 14px;
  border-radius: 16px;
  background: linear-gradient(180deg, rgba(16, 19, 26, 0.72), rgba(10, 12, 17, 0.78));
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.06);
  transition: opacity 0.25s;
}
.hud.dim {
  filter: saturate(0.6) brightness(0.8);
}
.hud.editing {
  cursor: grab;
  border-color: rgba(120, 200, 255, 0.7);
}
.hud.editing:active {
  cursor: grabbing;
}

/* 换挡灯条 */
.shiftlights {
  display: flex;
  gap: 3px;
  justify-content: center;
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

/* 主行 */
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
  padding: 0 22px;
}
.gear {
  font-size: 78px;
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
  font-size: 14px;
  line-height: 1.7;
  text-align: center;
  color: #cdd9e5;
  padding: 14px 4px;
}

/* ---------- 编辑面板 ---------- */
.panel {
  position: absolute;
  top: 24px;
  left: 24px;
  width: 220px;
  padding: 14px 16px;
  border-radius: 14px;
  background: rgba(18, 22, 30, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  font-size: 13px;
  color: #e6eef6;
}
.panel-title {
  font-weight: 700;
  margin-bottom: 12px;
  color: #9fd0ff;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 12px;
}
.field span {
  color: #9fb4c8;
}
.field input,
.field select {
  width: 100%;
  padding: 5px 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(0, 0, 0, 0.3);
  color: #fff;
  box-sizing: border-box;
}
.field input[type="range"] {
  padding: 0;
}
.hint {
  font-size: 11px;
  color: #7e93a8;
  line-height: 1.5;
  margin-bottom: 10px;
}
.quit {
  width: 100%;
  padding: 7px;
  border-radius: 8px;
  border: 1px solid rgba(255, 90, 77, 0.5);
  background: rgba(255, 90, 77, 0.15);
  color: #ff8a80;
  cursor: pointer;
}
.quit:hover {
  background: rgba(255, 90, 77, 0.28);
}

.lockhint {
  position: absolute;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 12px;
  color: rgba(200, 220, 240, 0.55);
}
</style>
