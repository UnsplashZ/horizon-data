import { ref, onMounted } from "vue";
import { editMode, tauri } from "./telemetry";

/**
 * 覆盖窗口通用行为：编辑模式下拖动移动 / 拖角缩放，并把位置和尺寸持久化；
 * 同时根据窗口宽度返回整体缩放比例（内容按基准尺寸等比缩放）。
 *
 * 关键：窗口 API 在 onMounted 预加载好，拖动/缩放在 pointerdown 中同步发起，
 * 否则 macOS 上动作会因 await import 的延迟而失效。
 */
export function useOverlayWindow(baseWidth: number) {
  const scale = ref(1);
  let win: { label: string; startDragging: () => Promise<void>; startResizeDragging: (d: string) => Promise<void> } | null =
    null;

  function recompute() {
    scale.value = window.innerWidth / baseWidth;
  }

  async function onDragDown() {
    if (!editMode.value || !win) return;
    try {
      await win.startDragging();
    } catch {}
  }
  async function onResizeDown(e: PointerEvent) {
    if (!editMode.value || !win) return;
    e.stopPropagation();
    try {
      await win.startResizeDragging("SouthEast");
    } catch {}
  }

  onMounted(async () => {
    recompute();
    window.addEventListener("resize", recompute);
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const w = getCurrentWindow();
      win = w as unknown as typeof win;
      const label = w.label;
      let tmove: ReturnType<typeof setTimeout> | null = null;
      let tsize: ReturnType<typeof setTimeout> | null = null;
      await w.onMoved(({ payload }) => {
        if (tmove) clearTimeout(tmove);
        tmove = setTimeout(async () => {
          const inv = await tauri();
          if (inv) {
            try {
              await inv("save_window_pos", { label, x: payload.x, y: payload.y });
            } catch {}
          }
        }, 300);
      });
      await w.onResized(({ payload }) => {
        recompute();
        if (tsize) clearTimeout(tsize);
        tsize = setTimeout(async () => {
          const inv = await tauri();
          if (inv) {
            try {
              await inv("save_window_size", { label, w: payload.width, h: payload.height });
            } catch {}
          }
        }, 300);
      });
    } catch {}
  });

  return { scale, onDragDown, onResizeDown };
}
