# Forza Horizon 6 遥测（Data Out）说明

> 整理日期：2026-06-16
> 用途：说明 horizon-data 使用的 FH6 UDP 遥测来源、配置方式和字段能力。
> 精确偏移表见本目录 `reference/forza_packet.py` 与 `reference/forzaPacket.ts`。

## 用户速查

游戏内打开：`SETTINGS → GAMEPLAY & HUD → UDP RACE TELEMETRY / Data Out`

- Data Out：`ON`
- 同机运行 HUD：IP 填 `127.0.0.1`
- 局域网另一台机器运行 HUD：IP 填那台机器的局域网 IP
- 端口：默认填 `10989`

进入自由驾驶或比赛后，游戏会持续向这个 IP 和端口发送 UDP 包。horizon-data 只读取这些官方遥测数据，不会向游戏写入任何内容。

## 1. 什么是 "Data Out"

Forza Motorsport / Forza Horizon 系列内置 **"Data Out"（UDP Race Telemetry）** 功能：
游戏按帧率（~60Hz）**单向**向你指定的 IP+端口广播 UDP 数据包。

- **只发不收**：游戏不接收任何数据，做覆盖层/仪表/座椅都不会影响游戏，也不算作弊外挂（读的是官方广播）。
- **官方文档**：<https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation>
  （注意：该站点对脚本抓取返回 403，需浏览器人工打开。）

## 2. 如何开启（FH6）

游戏内：`SETTINGS → GAMEPLAY & HUD → "UDP RACE TELEMETRY / Data Out"`
- 设为 **ON**
- 填写接收端 **IP**（本机做覆盖层填 `127.0.0.1`，局域网另一台机器填那台的 LAN IP）
- 填写**端口**（你自己定，和接收程序监听端口一致即可，如 `10989`）
- 进入比赛/自由驾驶后开始发送

## 3. 网络与包格式

| 项目 | FH6 |
|---|---|
| 传输 | 单向 UDP，约 60 包/秒 |
| 字节序 | 小端 little-endian |
| 包大小 | **固定 324 字节**（Car Dash 格式） |
| 格式切换 | 无（Horizon 只有一种固定格式，不像 FM 有 Sled/Dash 可选） |
| 与 FH5 关系 | **与 FH5 逐字节一致**；所有标准偏移相同（输入字节在 315/316/319 等） |

### 与 Forza Motorsport 的差异
- FM 有两种可选格式：**Sled**（232B，纯物理，给动感座椅）和 **Dash**（FM7=311B / FM2023=331B，多圈速/输入）。
- **FM2023** 比 FM7 多 20 字节：4×f32 轮胎磨损 + 2×2B 未知。**Horizon 的 324B 包不含轮胎磨损字段。**
- **FH6/FH5** 在 `NumCylinders` 之后、`PositionX` 之前多一段 12 字节，FH6 文档将其命名为：
  `CarGroup`、`SmashableVelDiff`、`SmashableMass`（FH4 时代是未知占位）。

## 4. 字段结构总览（逻辑分两段）

**Sled 段（0–231，物理/运动）**：转速、三轴加速度/速度/角速度、Yaw/Pitch/Roll、
四轮悬挂行程/打滑率/转速/路肩/水深/路面震动/滑移角/综合滑移、车辆元信息。

**Horizon 扩展块（232–243）**：CarGroup / SmashableVelDiff / SmashableMass。

**Dash 段（244–323，仪表/圈速/输入）**：位置、Speed、Power、Torque、四轮胎温、
Boost、Fuel、里程、最佳/上圈/本圈/总时间、圈数、名次、**油门/刹车/离合/手刹/档位/转向**、辅助线信息。

> 关键输入字节偏移（已与 FH6 实测来源核对）：
> `Accel=315`、`Brake=316`、`Clutch=317`、`HandBrake=318`、`Gear=319`、`Steer=320`。

完整逐字段偏移见 `reference/` 下的代码常量文件。

## 5. 解析要点

1. 绑定 UDP `0.0.0.0:<端口>`，收 324 字节。
2. 小端解包（Python `struct`，格式串见 reference）。
3. **先看 `IsRaceOn`（offset 0）**：=0 表示菜单/暂停/重生，数据无意义，应丢弃。
4. 速度：`Speed`(offset 256) 单位 **m/s**，×3.6 得 km/h。
5. 转速：`CurrentEngineRpm`(offset 16)，配合 `EngineMaxRpm`(8)/`EngineIdleRpm`(12) 做表盘。
6. 档位：`Gear`(319) u8，0=倒挡，11=空挡（N），其余为前进挡。
7. 油门/刹车：u8 0–255；转向 `Steer` s8 −127..127。
8. 多游戏共端口时可按**包长度**区分（FH6=324，FM7=311，FM2023=331，Sled=232）。

## 6. 不能直接读取的状态

FH6 Data Out 包更像“车辆物理和仪表数据”，不是游戏辅助系统状态接口。当前 324B 包里没有这些独立字段：

- ABS 开关或介入状态
- TCS/TCR 开关或介入状态
- Launch Control 状态
- 单独的红线转速或最佳换挡转速

可以基于现有字段做推断，例如用刹车输入、轮速和滑移估计 ABS 是否可能介入，或用油门、driven wheels slip、功率/扭矩变化估计牵引力控制是否可能介入。但这类结果只能作为启发式提示，不能当成游戏输出的真实状态位。

## 7. 参考项目

| 项目 | 说明 |
|---|---|
| Ojansen/co-driver | Vue+TS/Nuxt，自托管第二屏仪表+热圈+dyno+调校台；按包长度区分游戏 |
| viunow/fh6-telemetry | FH6 专用，全字段解析 + JSON 会话记录 |
| TheBanHammer/fh6-tel | FH6 实时 UDP 显示 + 录制 |
| ToTo-40417/FH6-Telemetry-Live-Dashboard | ESP32-S3 + WebSocket 浏览器仪表盘 |
| satyajiit/forza-horizon-6-moza-bridge | FH6 → MOZA 力反馈桥接 |
| richstokes/Forza-data-tools | Go，".dat 格式文件 + 动态偏移" 解析设计值得借鉴 |

## 8. 来源

- 官方 FH6 Data Out 文档（support.forza.net，文章 51744149102611）
- 官方 FM Data Out 文档（support.forza.net，文章 21742934024211）
- forums.forza.net/t/udp-telemetry-packet-details/629111
- 上述各开源仓库 README（确认 324B、偏移与 FH5 一致、输入字节 315/316/319）
