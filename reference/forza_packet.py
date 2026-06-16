"""
Forza Horizon 6 / FH5 "Data Out" UDP 数据包定义（Car Dash 格式，固定 324 字节）。

字节序：小端（little-endian）。
偏移与 FH5 逐字节一致；FH6 仅把 232..243 的 12 字节命名为
CarGroup / SmashableVelDiff / SmashableMass。

已核对的关键输入字节：Accel=315, Brake=316, Clutch=317,
HandBrake=318, Gear=319, Steer=320。

用法：
    import socket, struct
    from forza_packet import FORMAT, FIELDS, parse

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(("0.0.0.0", 5300))
    while True:
        data, _ = sock.recvfrom(1024)
        if len(data) == 324:
            t = parse(data)
            if t["IsRaceOn"]:
                print(t["CurrentEngineRpm"], t["Speed"] * 3.6, t["Gear"])
"""

import struct

PACKET_SIZE = 324  # FH6 / FH5 Car Dash
DEFAULT_PORTS = (5300,)  # 接收端自定义；游戏设置里需一致

# (名称, struct 类型码)  —— 顺序即字节顺序
# i=s32  I=u32  f=f32  H=u16  B=u8  b=s8  x=填充字节
SCHEMA = [
    # --- Sled 段 (0..231) ---
    ("IsRaceOn", "i"),                              # 0  0=菜单/暂停, 1=驾驶中
    ("TimestampMS", "I"),                           # 4  毫秒时间戳
    ("EngineMaxRpm", "f"),                          # 8
    ("EngineIdleRpm", "f"),                         # 12
    ("CurrentEngineRpm", "f"),                      # 16
    ("AccelerationX", "f"),                         # 20 车体坐标，右
    ("AccelerationY", "f"),                         # 24 上
    ("AccelerationZ", "f"),                         # 28 前
    ("VelocityX", "f"),                             # 32
    ("VelocityY", "f"),                             # 36
    ("VelocityZ", "f"),                             # 40
    ("AngularVelocityX", "f"),                      # 44
    ("AngularVelocityY", "f"),                      # 48
    ("AngularVelocityZ", "f"),                      # 52
    ("Yaw", "f"),                                   # 56
    ("Pitch", "f"),                                 # 60
    ("Roll", "f"),                                  # 64
    ("NormalizedSuspensionTravelFrontLeft", "f"),   # 68
    ("NormalizedSuspensionTravelFrontRight", "f"),  # 72
    ("NormalizedSuspensionTravelRearLeft", "f"),    # 76
    ("NormalizedSuspensionTravelRearRight", "f"),   # 80
    ("TireSlipRatioFrontLeft", "f"),                # 84
    ("TireSlipRatioFrontRight", "f"),               # 88
    ("TireSlipRatioRearLeft", "f"),                 # 92
    ("TireSlipRatioRearRight", "f"),                # 96
    ("WheelRotationSpeedFrontLeft", "f"),           # 100
    ("WheelRotationSpeedFrontRight", "f"),          # 104
    ("WheelRotationSpeedRearLeft", "f"),            # 108
    ("WheelRotationSpeedRearRight", "f"),           # 112
    ("WheelOnRumbleStripFrontLeft", "i"),           # 116
    ("WheelOnRumbleStripFrontRight", "i"),          # 120
    ("WheelOnRumbleStripRearLeft", "i"),            # 124
    ("WheelOnRumbleStripRearRight", "i"),           # 128
    ("WheelInPuddleDepthFrontLeft", "f"),           # 132
    ("WheelInPuddleDepthFrontRight", "f"),          # 136
    ("WheelInPuddleDepthRearLeft", "f"),            # 140
    ("WheelInPuddleDepthRearRight", "f"),           # 144
    ("SurfaceRumbleFrontLeft", "f"),                # 148
    ("SurfaceRumbleFrontRight", "f"),               # 152
    ("SurfaceRumbleRearLeft", "f"),                 # 156
    ("SurfaceRumbleRearRight", "f"),                # 160
    ("TireSlipAngleFrontLeft", "f"),                # 164
    ("TireSlipAngleFrontRight", "f"),               # 168
    ("TireSlipAngleRearLeft", "f"),                 # 172
    ("TireSlipAngleRearRight", "f"),                # 176
    ("TireCombinedSlipFrontLeft", "f"),             # 180  摩擦/抓地可视化用
    ("TireCombinedSlipFrontRight", "f"),            # 184
    ("TireCombinedSlipRearLeft", "f"),              # 188
    ("TireCombinedSlipRearRight", "f"),             # 192
    ("SuspensionTravelMetersFrontLeft", "f"),       # 196
    ("SuspensionTravelMetersFrontRight", "f"),      # 200
    ("SuspensionTravelMetersRearLeft", "f"),        # 204
    ("SuspensionTravelMetersRearRight", "f"),       # 208
    ("CarOrdinal", "i"),                            # 212 车型 ID
    ("CarClass", "i"),                              # 216
    ("CarPerformanceIndex", "i"),                   # 220
    ("DrivetrainType", "i"),                        # 224 0=FWD 1=RWD 2=AWD
    ("NumCylinders", "i"),                          # 228

    # --- FH6/FH5 Horizon 扩展块 (232..243) ---
    # 类型为推断（共 12 字节）；做仪表用不到，可忽略。
    ("CarGroup", "i"),                              # 232
    ("SmashableVelDiff", "f"),                      # 236
    ("SmashableMass", "f"),                         # 240

    # --- Dash 段 (244..323) ---
    ("PositionX", "f"),                             # 244 世界坐标(米)，做赛道图
    ("PositionY", "f"),                             # 248
    ("PositionZ", "f"),                             # 252
    ("Speed", "f"),                                 # 256 m/s  (×3.6=km/h)
    ("Power", "f"),                                 # 260 W
    ("Torque", "f"),                                # 264 Nm
    ("TireTempFrontLeft", "f"),                     # 268
    ("TireTempFrontRight", "f"),                    # 272
    ("TireTempRearLeft", "f"),                      # 276
    ("TireTempRearRight", "f"),                     # 280
    ("Boost", "f"),                                 # 284 涡轮(BST BAR)
    ("Fuel", "f"),                                  # 288 0..1
    ("DistanceTraveled", "f"),                      # 292 m
    ("BestLap", "f"),                               # 296 s
    ("LastLap", "f"),                               # 300 s
    ("CurrentLap", "f"),                            # 304 s
    ("CurrentRaceTime", "f"),                       # 308 s
    ("LapNumber", "H"),                             # 312
    ("RacePosition", "B"),                          # 314
    ("Accel", "B"),                                 # 315 油门 0..255
    ("Brake", "B"),                                 # 316 刹车 0..255
    ("Clutch", "B"),                                # 317 离合 0..255
    ("HandBrake", "B"),                             # 318 手刹 0..255
    ("Gear", "B"),                                  # 319 档位 0=R,11=N
    ("Steer", "b"),                                 # 320 转向 -127..127
    ("NormalizedDrivingLine", "b"),                 # 321
    ("NormalizedAIBrakeDifference", "b"),           # 322
    ("_pad", "x"),                                  # 323 末尾填充
]

# 小端格式串，例如 "<iIfff...x"
FORMAT = "<" + "".join(code for _, code in SCHEMA)
FIELDS = [name for name, code in SCHEMA if code != "x"]

assert struct.calcsize(FORMAT) == PACKET_SIZE, struct.calcsize(FORMAT)

_unpack = struct.Struct(FORMAT).unpack_from


def parse(data: bytes) -> dict:
    """把 324 字节包解析成 {字段名: 值} 字典。"""
    values = _unpack(data)
    # 跳过填充字节（unpack 不返回 'x'）
    return dict(zip(FIELDS, values))


if __name__ == "__main__":
    print("FORMAT =", FORMAT)
    print("size   =", struct.calcsize(FORMAT))
    print("fields =", len(FIELDS))
