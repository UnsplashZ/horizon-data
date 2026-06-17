# AGENTS.md

本文件给后续在本仓库工作的代理使用。回复尽量使用中文，改动前先读当前代码，不要只按旧 README 或设计文档推断状态。

## 项目概览

`horizon-data` 是 Forza Horizon 6 透明置顶 HUD 覆盖层。应用读取游戏 UDP Data Out 的 324 字节遥测包，在 Tauri 透明窗口中显示速度、转速、档位、输入、轮胎与 G 力信息。

当前主技术栈：

- 前端：Vue 3 + Vite + TypeScript。
- 桌面壳：Tauri v2 + Rust。
- 运行窗口：`main` HUD 窗口和 `controls` 托盘弹出控制面板。旧独立模块视图和旧设置窗口已经清理。
- 遥测协议真源：`reference/forzaPacket.ts`、`reference/forza_packet.py` 与 `docs/forza-horizon-6-telemetry.md`。

## Git 操作规则

执行任何 git workflow 动作前必须先向用户确认并得到明确授权。

包括但不限于：

- 创建或切换分支。
- commit、amend、squash、rebase。
- push、pull、merge、cherry-pick、reset。
- 创建 PR 或修改 remote tracking。
- 打 tag、触发发布相关的 git 操作。

只读命令如 `git status --short`、`git diff` 可以用于了解工作区状态，但不得借此执行上面的写入/工作流动作。

## 常用命令

```bash
npm install
npm run dev          # 仅启动 Vite 前端，端口 1420
npm run build        # vue-tsc --noEmit + vite build
npm run tauri dev    # 启动完整 Tauri 覆盖层
npm run tauri build  # 构建桌面应用包
```

遥测链路调试：

```bash
npm run listen       # 监听 UDP:10989 并打印关键遥测
npm run fake         # 向 127.0.0.1:10989 发送伪造 324B 包
```

注意：`tools/*.ts` 直接由 `node` 执行，当前写法依赖支持直接运行 TypeScript 的新版本 Node；若本机 Node 不支持，先不要改业务逻辑，优先确认运行时版本或改用合适的 TS runner。

## 关键代码位置

- `src-tauri/src/main.rs`
  - 读取/保存 `horizon-data-config.json`。
  - 绑定 `0.0.0.0:<port>` 接收 UDP。
  - 按 FH6 324B Car Dash 包解析字段并通过 Tauri event emit `telemetry`。
  - 注册 `CmdOrCtrl+Shift+H`，仅用于临时切换 HUD 编辑/锁定以调整位置和尺寸。
  - 创建系统托盘/状态栏图标，左键打开/隐藏 `controls`，右键菜单提供编辑布局、锁定 HUD、打开控制面板和退出。
  - 锁定模式对 HUD 窗口调用 `set_ignore_cursor_events(true)` 实现点击穿透。
- `src/telemetry.ts`
  - 前端共享响应式状态。
  - 封装 `get_config`、`update_config`、`set_edit_mode`、`quit_app` 与事件订阅。
- `src/dragwin.ts`
  - 编辑模式下拖动、缩放窗口，并 debounce 保存位置和尺寸。
- `src/views/HudMain.vue`
  - 当前主 HUD。集成速度、档位、转速灯、输入条、轮胎信息与 G 力模块。
- `src/views/TrayControls.vue`
  - 托盘弹出的控制面板。提供端口直接输入、透明度滑条、单位、模块开关、编辑/锁定和退出。
- `src-tauri/tauri.conf.json`
  - Tauri 窗口、透明置顶、bundle 和 dev/build 命令配置。
- `src-tauri/capabilities/default.json`
  - Tauri v2 权限。新增窗口或窗口 API 能力时要同步更新。
- `.github/workflows/release.yml`
  - tag `v*` 或手动触发时构建 macOS/Windows 预发布包。

## 实现约束

- FH6 数据包大小必须保持为 324 字节，小端解析。改字段偏移时要同时核对 Rust 解析、`reference/forzaPacket.ts`、`reference/forza_packet.py` 和文档。
- `Speed` 原始单位是 m/s，HUD 中 km/h 为 `Speed * 3.6`，mph 在前端由 km/h 转换。
- `Gear` 显示规则：`0=R`、`11=N`、其余显示数字。
- 胎温源字段按现有实现从华氏度转摄氏度显示；不要只改 UI 单位而不改颜色阈值。
- 保持透明覆盖层语义：`html/body/#app` 背景透明，Tauri 窗口 `transparent: true`、`decorations: false`、`alwaysOnTop: true`。
- 锁定模式必须点击穿透，编辑模式才允许拖动/缩放。
- `update_config` 必须保留窗口位置和尺寸；窗口位置/尺寸只由拖动和缩放保存。
- 配置文件优先写在可执行文件同目录，便于绿色版/便携版使用；安装目录不可写时必须落用户配置目录。
- 新增 Tauri command、窗口、全局快捷键、托盘菜单或文件/网络能力时，要检查 `capabilities/default.json` 和 `src-tauri/Cargo.toml` feature。

## 文档维护

- README 写当前可运行状态、安装运行方式和用户操作说明。
- `docs/forza-horizon-6-telemetry.md` 记录协议和字段偏移。
- `docs/overlay-design.md` 是设计/路线图，可能滞后；与代码冲突时先以代码和 README 的当前实现为准，再更新设计文档。
- `docs/TODO.md` 目前较粗略，更新时避免重复标题和过期完成项。

## 验证建议

文档或注释改动通常无需完整打包，但涉及代码时至少运行：

```bash
npm run build
```

涉及 Tauri/Rust、窗口权限、UDP 接收、全局快捷键或配置持久化时，优先运行：

```bash
npm run tauri dev
```

Windows 覆盖行为、点击穿透、置顶效果、NSIS 安装包与 SmartScreen 提示必须在 Windows 或 release workflow 产物上复核，不能只凭 macOS 开发环境下结论。
