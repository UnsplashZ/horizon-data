import { onMounted } from "vue";
import { editMode, tauri } from "./telemetry";

/**
 * 编辑模式下：在覆盖窗口内按下即拖动整个 OS 窗口；移动结束后持久化位置。
 * 锁定模式下窗口点击穿透，pointerdown 不会触发。
 */
export function useDragWindow() {
  let label = "main";

  async function onPointerDown() {
    if (!editMode.value) return;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().startDragging();
  }

  onMounted(async () => {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const w = getCurrentWindow();
      label = w.label;
      let timer: ReturnType<typeof setTimeout> | null = null;
      await w.onMoved(({ payload }) => {
        if (timer) clearTimeout(timer);
        timer = setTimeout(async () => {
          const inv = await tauri();
          if (inv) {
            try {
              await inv("save_window_pos", { label, x: payload.x, y: payload.y });
            } catch {}
          }
        }, 300);
      });
    } catch {}
  });

  return { onPointerDown };
}
