# horizon-data

Forza Horizon 6 的透明置顶 HUD 覆盖层。它读取游戏内置的 UDP Data Out 遥测包，把速度、转速、档位、输入、轮胎和 G 力信息显示在游戏画面上。

当前版本已接入主 HUD、托盘控制面板、透明置顶、点击穿透、编辑/锁定模式、配置保存和 324B FH6 遥测包解析。Windows 覆盖效果建议在无边框窗口化模式下使用。

## 使用方法

1. 启动 `horizon-data`。
2. 打开 FH6：`SETTINGS -> GAMEPLAY & HUD -> UDP RACE TELEMETRY / Data Out`。
3. 将 Data Out 设为 `ON`。
4. 如果 HUD 和游戏在同一台电脑上，IP 填 `127.0.0.1`；如果 HUD 在另一台局域网设备上，IP 填运行 HUD 机器的局域网 IP。
5. 端口填 `10989`。
6. 进入自由驾驶或比赛，HUD 会在收到遥测后开始显示。

如果你升级前已经运行过旧版本，应用可能会继续读取旧配置文件里的端口。可以在托盘控制面板里把端口改为 `10989` 并保存。

## 托盘操作

启动后应用会常驻系统托盘/状态栏。

- 左键托盘图标：打开或隐藏控制面板。
- 右键托盘图标：
  - `打开控制面板`：显示端口、单位、透明度、模块开关和非活动自动隐藏开关；面板标题栏右上角的 `×` 可关闭面板（不影响编辑/锁定状态）。
  - `隐藏/显示 HUD`：临时隐藏或恢复覆盖层，应用和 UDP 监听继续运行。
  - `编辑 HUD 布局`：恢复 HUD 并进入可拖动/缩放模式。
  - `锁定 HUD`：锁定覆盖层并开启点击穿透。
  - `退出`：关闭应用。

快捷键 `Ctrl+Shift+H`（macOS 为 `Cmd+Shift+H`）可以临时切换 HUD 编辑/锁定。锁定后 HUD 不会挡住游戏鼠标操作。

## HUD 内容

- 速度：支持 `KM/H` 和 `MPH`。
- 转速：使用 `CurrentEngineRpm / EngineMaxRpm` 点亮转速灯，并分三段提示换挡——接近红线（约 88%）转速/档位转琥珀色，到达换挡点（约 92%）转橙并闪烁，触顶/超转（约 95%）转红并加速闪烁。FH6 遥测没有独立的红线/最佳换挡转速字段，这里按占最大转速的比例近似。
- 马力与换挡点：左转速环内显示当前马力（`Power` 换算为公制 PS）；转速弧外缘有一颗发光圆点，标记本台车记录到的**峰值功率转速**（最优换挡点）。当前转速越过该点后圆点与马力数字转橙提示。峰值随驾驶实时记录，离开比赛（`IsRaceOn=false`）后重置，便于换车重新记录。
- 档位：`0` 显示为 `R`，`11` 显示为 `N`。
- 输入：油门、刹车、转向。
- 轮胎：胎温和 combined slip 抓地状态。
- G 力：横向/纵向加速度圆点。
- 非活动自动隐藏：在控制面板开启后，HUD 会在收到 `IsRaceOn=false` 的遥测状态时自动隐藏；编辑模式下保持可见，方便调整位置和尺寸。

FH6 遥测包没有单独输出 ABS、TCS/TCR 或 Launch Control 的开关/介入状态。HUD 可以读取刹车、油门、轮胎滑移、轮速和功率等数据，但这些辅助系统状态只能做推断，不能直接准确显示。

## 常见问题

### 没有数据显示

- 确认 FH6 的 Data Out 已开启。
- 确认游戏里填写的端口和控制面板端口一致，默认是 `10989`。
- 同机运行时 IP 填 `127.0.0.1`。
- 进入自由驾驶或比赛后才会持续输出有效遥测。
- 防火墙拦截 UDP 时，需要允许 FH6 或本应用通信。

### HUD 不在游戏上方

普通透明置顶窗口通常无法覆盖全屏独占游戏。建议把 FH6 设置为无边框窗口化或窗口化模式。

### HUD 挡住鼠标

在托盘菜单里点 `锁定 HUD`，或按 `Ctrl+Shift+H` / `Cmd+Shift+H` 切回锁定模式。锁定模式会开启点击穿透。

### 想临时关掉 HUD

右键托盘图标，选择 `隐藏/显示 HUD`。这不会停止应用，也不会停止 UDP 监听。

## 开发

需要 Node.js、Rust 和 Tauri 支持的系统依赖。Windows 打包还需要 MS C++ Build Tools；Windows 10/11 通常已带 WebView2。

```bash
npm install
npm run tauri dev
```

常用命令：

```bash
npm run dev          # 仅启动 Vite 前端，端口 1420
npm run build        # TypeScript 检查 + Vite 构建
npm run tauri dev    # 启动完整 Tauri 应用
npm run tauri build  # 构建桌面应用包
```

UDP 调试工具：

```bash
npm run listen       # 监听 UDP:10989 并打印关键遥测
npm run fake         # 向 127.0.0.1:10989 发送伪造 324B 包
```

`tools/*.ts` 当前直接由 `node` 执行，适合支持直接运行 TypeScript 的新版本 Node。若本机 Node 不支持，优先使用 Tauri 应用验证真实 UDP 链路。

## 项目结构

```text
src/
  main.ts              # 按 view 挂载主 HUD 或托盘控制面板
  telemetry.ts         # 前端共享状态、Tauri invoke、事件订阅
  dragwin.ts           # 编辑模式拖动/缩放与窗口位置持久化
  views/
    HudMain.vue        # 主 HUD
    TrayControls.vue   # 托盘控制面板
src-tauri/
  src/main.rs          # UDP 监听、包解析、托盘菜单、配置、快捷键、窗口控制
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

## 发布

GitHub Actions 会在 push `v*` tag 或手动触发时构建预发布包：

- Windows：NSIS 安装包。
- macOS：universal `.app/.dmg`。

正式使用建议优先下载最新的 GitHub pre-release 包。未签名版本在 Windows 上可能出现 SmartScreen 提示，选择“仍要运行”即可。

## 参考

- [FH6 遥测说明](docs/forza-horizon-6-telemetry.md)
- [覆盖层设计记录](docs/overlay-design.md)
- [TODO](docs/TODO.md)
