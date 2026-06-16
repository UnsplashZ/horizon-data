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
  show_inputs: boolean;
  show_grip: boolean;
  show_gforce: boolean;
  show_tiretemp: boolean;
  pos_main: number[];
  pos_inputs: number[];
  pos_grip: number[];
  pos_gforce: number[];
  pos_tiretemp: number[];
  size_main: number[];
  size_inputs: number[];
  size_grip: number[];
  size_gforce: number[];
  size_tiretemp: number[];
}

export const telemetry = ref<Telemetry | null>(null);
export const editMode = ref(false);
export const config = reactive<Config>({
  port: 5300,
  bg_opacity: 0.72,
  fg_opacity: 1.0,
  units: "kmh",
  show_inputs: false,
  show_grip: false,
  show_gforce: false,
  show_tiretemp: false,
  pos_main: [-1, -1],
  pos_inputs: [-1, -1],
  pos_grip: [-1, -1],
  pos_gforce: [-1, -1],
  pos_tiretemp: [-1, -1],
  size_main: [0, 0],
  size_inputs: [0, 0],
  size_grip: [0, 0],
  size_gforce: [0, 0],
  size_tiretemp: [0, 0],
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
    } catch {}
  }
  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<Telemetry>("telemetry", (e) => (telemetry.value = e.payload));
    await listen<Config>("config", (e) => Object.assign(config, e.payload));
    await listen<boolean>("edit-mode", (e) => (editMode.value = e.payload));
  } catch {}
}

export async function updateConfig() {
  const inv = await tauri();
  if (inv) {
    try {
      await inv("update_config", { config: { ...config } });
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

/** 数值工具 */
export function gearLabel(g: number): string {
  return g === 0 ? "R" : g === 11 ? "N" : String(g);
}
