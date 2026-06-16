# horizon-data

Forza Horizon 6 的**透明置顶屏幕覆盖仪表**：读游戏的 UDP "Data Out" 遥测，把时速 / 转速 / 档位等
显示在屏幕自定义位置，替代右下角看着费劲的原生表。可选开启输入、摩擦力、G力可视化。

> 状态：方案设计阶段。详细文档见 [`docs/`](docs/)，数据接口常量见 [`reference/`](reference/)。

## 文档导航

- [docs/forza-horizon-6-telemetry.md](docs/forza-horizon-6-telemetry.md) — FH6 遥测接口调研（开启方式、UDP/包格式、字段表、解析要点）
- [docs/overlay-design.md](docs/overlay-design.md) — 覆盖层产品设计、交互/视觉决策、打包要求、路线图
- [reference/forza_packet.py](reference/forza_packet.py) — Python 精确偏移表 + `struct` 解析（324B，已验证）
- [reference/forzaPacket.ts](reference/forzaPacket.ts) — TypeScript/Node 偏移表 + `parseForzaPacket()`

## 完整方案概览

### 数据来源
FH6 内置 **Data Out**：游戏按 ~60Hz **单向** UDP 广播一个**固定 324 字节**的包到你指定 IP:端口。
只发不收、读官方广播，安全合规。游戏内 `SETTINGS → GAMEPLAY & HUD → UDP RACE TELEMETRY` 开启。

关键字段偏移（小端）：`IsRaceOn`=0、`CurrentEngineRpm`=16、`Speed`=256(m/s)、
`Accel`=315、`Brake`=316、`Gear`=319、`Steer`=320、`TireCombinedSlip*`=180–192、`Acceleration X/Y/Z`=20/24/28。

### 技术栈：Tauri（已选定）
Rust + Web 前端（WebView2）。理由：产物小（便携 exe 几~十几 MB）、占用低（与游戏同跑不抢资源）、
原生支持透明窗口 / 置顶 / 点击穿透、可同时产出 NSIS 安装版与绿色便携版、仪表用 Canvas/SVG 高效开发。

覆盖窗口配置要点：`transparent:true`、`decorations:false`、`alwaysOnTop:true`、`skipTaskbar:true`；
点击穿透用 `set_ignore_cursor_events`（锁定态穿透、编辑态可拖动）。
透明度用 CSS 背景 alpha 控制（文字始终清晰）+ 全局透明度滑块，默认背景 `rgba(20,20,20,0.4)`、整体 ~88%。

### 开发工作流：Mac 开发 + Windows 同局域网跑游戏
- Windows 上 FH6 的 Data Out 目标 IP 填 **Mac 的局域网 IP**，把真实遥测包发到 Mac。
- Mac 上 `tauri dev` 即可实时看仪表、调 UI/解析逻辑（接收端 bind `0.0.0.0`，端口与游戏一致）。
- **只有最终覆盖行为（透明置顶/点击穿透/全屏游戏之上）和 `.exe` 打包需要在 Windows 上验证。**
- 监听地址/端口做成可配置：生产同机 `127.0.0.1`，远程调试用 Mac 的 LAN IP。

### Windows 构建环境（不复杂，建议直接在 Windows 构建）
1. **MS C++ Build Tools**（VS Build Tools，勾选「使用 C++ 的桌面开发」）— 最大一项
2. **WebView2** — Win10/11 已预装，通常无需操作
3. **Rust**（rustup-init）
4. **Node.js LTS** + 包管理器

NSIS/便携打包工具由 Tauri bundler 自动下载。开发 `npm run tauri dev`，出包 `npm run tauri build`。
CI（GitHub Actions windows runner）留作后续自动出 release 的可选项。

## 路线图
1. UDP 接收+解析 demo（验证 FH6 正常发包，打印 rpm/速度/档位）
2. 搭 Tauri 透明置顶+点击穿透空覆盖窗口
3. 核心三件套（速度/转速/档位）+ 透明度滑块 + 拖动定位 + 配置持久化
4. 可选模块：输入 / 摩擦力 / G力 可视化
5. 打包 NSIS 安装版 + 绿色便携版；CI 出 release

## 致谢 / 参考
- 官方 FH6 Data Out 文档（support.forza.net 文章 51744149102611）
- 参考项目 [Ojansen/co-driver](https://github.com/Ojansen/co-driver)
