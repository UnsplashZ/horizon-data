import { ref, reactive } from "vue";

export interface Telemetry {
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
  tire_temp: number[]; // 四轮胎温
}

export interface Config {
  port: number;
  bg_opacity: number;
  fg_opacity: number;
  units: string;
  show_tires: boolean;   // 统一仪表盘中的轮胎模块
  show_inputs: boolean;  // 统一仪表盘中的输入模块
  show_gforce: boolean;  // 统一仪表盘中的G力模块
  pos_main: number[];
  size_main: number[];
}

export interface UdpStatus {
  port: number;
  listening: boolean;
  error: string | null;
}

export interface ShortcutStatus {
  registered: boolean;
  error: string | null;
}

export const telemetry = ref<Telemetry | null>(null);
export const editMode = ref(false);
export const udpStatus = ref<UdpStatus | null>(null);
export const configError = ref<string | null>(null);
export const shortcutStatus = ref<ShortcutStatus | null>(null);
export const config = reactive<Config>({
  port: 10989,
  bg_opacity: 0.72,
  fg_opacity: 1.0,
  units: "kmh",
  show_tires: true,
  show_inputs: true,
  show_gforce: true,
  pos_main: [-1, -1],
  size_main: [0, 0],
});

type Invoke = <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
let invoke: Invoke | null = null;
export async function tauri(): Promise<Invoke | null> {
  if (!invoke) {
    try {
      ({ invoke } = await import("@tauri-apps/api/core"));
    } catch {
      return null;
    }
  }
  return invoke;
}

let inited = false;
/** 每个窗口各自调用一次：拉取配置 + 订阅广播事件 */
export async function initShared() {
  if (inited) return;
  inited = true;
  const inv = await tauri();
  if (inv) {
    try {
      Object.assign(config, await inv<Config>("get_config"));
      udpStatus.value = await inv<UdpStatus>("get_udp_status");
      shortcutStatus.value = await inv<ShortcutStatus>("get_shortcut_status");
    } catch {}
  }
  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<Telemetry>("telemetry", (e) => (telemetry.value = e.payload));
    await listen<Config>("config", (e) => Object.assign(config, e.payload));
    await listen<boolean>("edit-mode", (e) => (editMode.value = e.payload));
    await listen<UdpStatus>("udp-status", (e) => (udpStatus.value = e.payload));
    await listen<ShortcutStatus>("shortcut-status", (e) => (shortcutStatus.value = e.payload));
  } catch {}
}

export async function updateConfig(): Promise<boolean> {
  const inv = await tauri();
  if (!inv) {
    configError.value = "Tauri API 不可用";
    return false;
  }
  try {
    await inv("update_config", { config: normalizedConfig() });
    configError.value = null;
    return true;
  } catch (err) {
    configError.value = errorMessage(err);
    return false;
  }
}

function normalizedConfig(): Config {
  const port = Math.round(Number(config.port));
  const bgOpacity = Number(config.bg_opacity);
  const fgOpacity = Number(config.fg_opacity);
  return {
    ...config,
    port: Number.isFinite(port) ? Math.min(65535, Math.max(1, port)) : 10989,
    bg_opacity: Number.isFinite(bgOpacity) ? Math.min(1, Math.max(0, bgOpacity)) : 0.72,
    // HUD 前景下限 0.2，避免滑到全透明后找不到覆盖层（与控制面板滑条 min 一致）
    fg_opacity: Number.isFinite(fgOpacity) ? Math.min(1, Math.max(0.2, fgOpacity)) : 1,
    units: config.units === "mph" ? "mph" : "kmh",
  };
}

export async function setEditMode(value: boolean) {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("set_edit_mode", { editing: value });
    } catch {}
  }
}

export async function quitApp() {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("quit_app");
    } catch {}
  }
}

function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return String(err);
}

/** 数值工具 */
export function gearLabel(g: number): string {
  return g === 0 ? "R" : g === 11 ? "N" : String(g);
}
