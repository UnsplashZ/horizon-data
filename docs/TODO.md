# 待办 / TODO

## 待处理（来自实测反馈）

- [ ] **胎温改为摄氏度**
  - 现在直接显示包内 `tire_temp` 原值（量级偏 °F）。需确认 FH6 胎温单位并换算/标注为 ℃。
  - 影响：`src/views/TireTemp.vue`（`tempColor` 阈值与显示）、可能加单位换算。

- [ ] **编辑模式仍然不能拖动窗口**
  - 已尝试：拖动前预加载窗口 API、按下时同步 `startDragging()`，仍无效（mac 上）。
  - 待查方向：① 改用 Tauri 的 `data-tauri-drag-region` 属性拖动（比 JS `startDragging` 更可靠）；
    ② 确认 `set_ignore_cursor_events(false)` 在 mac 上是否真正恢复了鼠标事件（已知 mac 有相关问题）；
    ③ 确认 `core:window:allow-start-dragging` 能力是否对各窗口生效；
    ④ 在 Windows dev 版上验证是否同样无法拖动（可能仅 mac 问题）。
  - 影响：`src/dragwin.ts`、各 `src/views/*.vue`、`src/views/Settings.vue`。

- [ ] **默认各窗口高度保持一致**
  - 当前默认尺寸不一（main 150 / inputs 150 / grip 170 / gforce 200 / tiretemp 175）。
  - 期望：默认高度统一，排成一行时观感整齐（宽度可不同）。
  - 影响：`src-tauri/tauri.conf.json` 各窗口 height、对应视图 `BASE_H` 与排版。

## 已完成（节选）
- 多窗口拆分、独立定位/持久化、缩放（角柄）、透明度拆分、换挡灯、输入/抓地/G力/胎温模块、圆角修复。
