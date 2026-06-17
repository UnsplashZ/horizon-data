# horizon-data

Forza Horizon 6 的透明置顶屏幕覆盖仪表。应用读取游戏内置 UDP Data Out 遥测包，把速度、转速、档位、输入、轮胎和 G 力信息显示在可锁定的 HUD 窗口中。

当前状态：Tauri + Vue 实现已可运行。主 HUD、托盘入口、托盘控制面板、UDP 监听、324B 包解析、配置持久化、透明置顶、点击穿透、编辑/锁定模式已经接入。Windows 覆盖游戏之上的最终体验仍需要在目标机器上复核。

## 功能

- 读取 FH6 Data Out UDP 包，默认监听 `0.0.0.0:5300`。
- 显示核心三件套：速度、转速、档位。
- 转速灯条按当前转速占最大转速比例点亮，接近红线时闪烁。
- 可开关模块：轮胎信息（胎温 + combined slip）、输入信息（油门/刹车/转向）、G 力圆点。
- 状态栏/托盘图标控制入口：左键打开/隐藏托盘控制面板，右键菜单提供编辑布局、锁定 HUD、打开控制面板和退出。
- 托盘控制面板支持端口直接输入、透明度滑条、速度单位、模块开关、编辑/锁定和退出。
- `Cmd/Ctrl+Shift+H` 仅用于临时调整 HUD 位置和尺寸：
  - 锁定模式：HUD 点击穿透，不挡游戏操作。
  - 编辑模式：HUD 可拖动和缩放。
- 配置优先保存为可执行文件同目录的 `horizon-data-config.json`；如果安装目录不可写，则自动保存到用户配置目录。

## 快速开始

需要 Node.js、Rust 和 Tauri 支持的系统依赖。Windows 打包还需要 MS C++ Build Tools；Windows 10/11 通常已带 WebView2。

```bash
npm install
npm run tauri dev
```

启动后进入 FH6：

1. 打开 `SETTINGS -> GAMEPLAY & HUD -> UDP RACE TELEMETRY / Data Out`。
2. 设置为 `ON`。
3. 如果游戏和 HUD 在同一台 Windows 机器上，IP 填 `127.0.0.1`；如果在局域网另一台机器上调试，IP 填运行本应用机器的局域网 IP。
4. 端口填 `5300`，或填托盘菜单中配置的端口。
5. 进入比赛或自由驾驶后 HUD 会开始显示数据。

开发常用命令：

```bash
npm run dev          # 仅启动 Vite 前端，端口 1420
npm run build        # TypeScript 检查 + Vite 构建
npm run tauri dev    # 启动完整 Tauri 应用
npm run tauri build  # 构建桌面应用包
```

UDP 调试工具：

```bash
npm run listen       # 监听 UDP 并打印 rpm/速度/档位/输入
npm run fake         # 发送伪造包到 127.0.0.1:5300
```

`tools/*.ts` 当前直接由 `node` 执行，适合支持直接运行 TypeScript 的新版本 Node。若本机 Node 不支持，可先使用 Tauri 应用本身验证真实 UDP 链路。

## 项目结构

```text
src/
  main.ts              # 按 view 挂载主 HUD 或托盘控制面板
  telemetry.ts         # 前端共享状态、Tauri invoke、事件订阅
  dragwin.ts           # 编辑模式拖动/缩放与窗口位置持久化
  views/
    HudMain.vue        # 当前主 HUD
    TrayControls.vue   # 托盘弹出的控制面板
src-tauri/
  src/main.rs          # UDP 监听、FH6 包解析、托盘菜单、配置、快捷键、窗口控制
  tauri.conf.json      # 窗口与构建配置
  capabilities/        # Tauri v2 权限
reference/
  forzaPacket.ts       # TypeScript 324B 字段表和解析器
  forza_packet.py      # Python 324B 字段表和解析器
tools/
  udp-listen.ts        # 命令行 UDP 监听器
  fake-send.ts         # 本地伪造包发送器
docs/
  forza-horizon-6-telemetry.md
  overlay-design.md
  TODO.md
```

## 遥测数据

FH6 Data Out 会以约 60Hz 单向 UDP 广播固定 324 字节 Car Dash 包，小端编码。关键字段：

| 字段 | 偏移 | 说明 |
|---|---:|---|
| `IsRaceOn` | 0 | `1` 表示比赛/自由驾驶中 |
| `EngineMaxRpm` | 8 | 最大转速 |
| `CurrentEngineRpm` | 16 | 当前转速 |
| `AccelerationX` / `AccelerationZ` | 20 / 28 | G 力显示使用 |
| `TireCombinedSlip*` | 180-192 | 四轮 combined slip |
| `Speed` | 256 | m/s，HUD 转为 km/h 或 mph |
| `TireTemp*` | 268-280 | 四轮胎温，前端转摄氏度显示 |
| `Accel` / `Brake` | 315 / 316 | 油门、刹车，`0..255` |
| `Gear` | 319 | `0=R`、`11=N` |
| `Steer` | 320 | 转向，`-127..127` |

完整字段表见 [`reference/forzaPacket.ts`](reference/forzaPacket.ts) 和 [`reference/forza_packet.py`](reference/forza_packet.py)。

## 构建与发布

本仓库已有 GitHub Actions release workflow：

- push `v*` tag 或手动触发 `workflow_dispatch` 会构建预发布包。
- macOS 产出 universal `.app/.dmg`。
- Windows 产出 NSIS 安装包。

本地 Windows 构建：

```bash
npm run tauri build
```

注意：普通置顶透明窗口可能无法覆盖全屏独占游戏，建议 FH6 使用无边框窗口化模式。点击穿透、置顶、透明和安装包行为都应在 Windows 目标环境复核。

## 文档导航

- [docs/forza-horizon-6-telemetry.md](docs/forza-horizon-6-telemetry.md) — FH6 遥测接口调研、开启方式、UDP/包格式、字段表和解析要点。
- [docs/overlay-design.md](docs/overlay-design.md) — 覆盖层产品设计、交互/视觉决策、打包要求和路线图。
- [docs/TODO.md](docs/TODO.md) — 当前待办与已完成项摘录。
- [AGENTS.md](AGENTS.md) — 给后续代理的项目协作规范。

## 路线图

1. ✅ UDP 接收 + 324B 包解析。
2. ✅ Tauri 透明置顶窗口、点击穿透、编辑/锁定模式。
3. ✅ 主 HUD、托盘控制入口、透明度滑条、端口直接输入、单位、模块开关和配置持久化。
4. ✅ 输入、轮胎、G 力模块集成到主 HUD，并清理旧独立模块视图与旧设置窗口。
5. 待复核：Windows 目标环境下的托盘入口、覆盖层体验、安装包、便携运行和 release workflow 产物。
6. 后续：F1 halo HUD 视觉重构、窗口布局重置、录制/回放、圈速/功率/扭矩等扩展模块。

## 致谢 / 参考

- 官方 FH6 Data Out 文档（support.forza.net 文章 51744149102611）
- 参考项目 [Ojansen/co-driver](https://github.com/Ojansen/co-driver)
