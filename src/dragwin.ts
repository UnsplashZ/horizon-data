import { ref, onMounted } from "vue";
import { editMode, tauri } from "./telemetry";

/**
 * 覆盖窗口通用行为：编辑模式下拖动移动 / 拖角缩放，并把位置和尺寸持久化；
 * 同时根据窗口宽高返回整体缩放比例（内容按基准尺寸等比缩放）。
 *
 * 关键：窗口 API 在 onMounted 预加载好，拖动/缩放在 pointerdown 中同步发起，
 * 否则 macOS 上动作会因 await import 的延迟而失效。
 */
export function useOverlayWindow(baseWidth: number, baseHeight?: number) {
  const scale = ref(1);
  let win: { label: string; startDragging: () => Promise<void>; startResizeDragging: (d: string) => Promise<void> } | null =
    null;

  function recompute() {
    // 根据宽高比例取较小值，保持内容不超出窗口
    const scaleX = window.innerWidth / baseWidth;
    const scaleY = baseHeight ? window.innerHeight / baseHeight : scaleX;
    scale.value = Math.min(scaleX, scaleY);
  }

  async function onDragDown() {
    if (!editMode.value || !win) return;
    try {
      await win.startDragging();
    } catch (error) {
      console.error("拖动 HUD 窗口失败", error);
    }
  }
  async function onResizeDown(e: PointerEvent) {
    if (!editMode.value || !win) return;
    e.stopPropagation();
    try {
      await win.startResizeDragging("SouthEast");
    } catch (error) {
      console.error("缩放 HUD 窗口失败", error);
    }
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
            } catch (error) {
              console.error("保存窗口位置失败", error);
            }
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
            } catch (error) {
              console.error("保存窗口尺寸失败", error);
            }
          }
        }, 300);
      });
    } catch {
      // 纯浏览器（无 Tauri）下窗口 API 不可用，属预期降级，保持静默
    }
  });

  return { scale, onDragDown, onResizeDown };
}
