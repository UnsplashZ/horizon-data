/**
 * Forza Horizon 6 / FH5 "Data Out" UDP 包定义（Car Dash，固定 324 字节，小端）。
 *
 * 偏移与 FH5 一致；FH6 仅把 232..243 命名为 CarGroup/SmashableVelDiff/SmashableMass。
 * 已核对：Accel=315, Brake=316, Clutch=317, HandBrake=318, Gear=319, Steer=320。
 *
 * 用法（Node）：
 *   import dgram from "node:dgram";
 *   import { parseForzaPacket, FH6_PACKET_SIZE } from "./forzaPacket";
 *   const sock = dgram.createSocket("udp4");
 *   sock.on("message", (buf) => {
 *     if (buf.length === FH6_PACKET_SIZE) {
 *       const t = parseForzaPacket(buf);
 *       if (t.IsRaceOn) console.log(t.CurrentEngineRpm, t.Speed * 3.6, t.Gear);
 *     }
 *   });
 *   sock.bind(5300, "0.0.0.0");
 */

export const FH6_PACKET_SIZE = 324;

type FieldType = "s32" | "u32" | "f32" | "u16" | "u8" | "s8";

interface FieldDef {
  name: string;
  type: FieldType;
  offset: number;
}

const SIZE: Record<FieldType, number> = { s32: 4, u32: 4, f32: 4, u16: 2, u8: 1, s8: 1 };

// 声明顺序即字节顺序；offset 自动累加。
const SCHEMA: ReadonlyArray<[string, FieldType]> = [
  // Sled 段
  ["IsRaceOn", "s32"],
  ["TimestampMS", "u32"],
  ["EngineMaxRpm", "f32"],
  ["EngineIdleRpm", "f32"],
  ["CurrentEngineRpm", "f32"],
  ["AccelerationX", "f32"], ["AccelerationY", "f32"], ["AccelerationZ", "f32"],
  ["VelocityX", "f32"], ["VelocityY", "f32"], ["VelocityZ", "f32"],
  ["AngularVelocityX", "f32"], ["AngularVelocityY", "f32"], ["AngularVelocityZ", "f32"],
  ["Yaw", "f32"], ["Pitch", "f32"], ["Roll", "f32"],
  ["NormalizedSuspensionTravelFrontLeft", "f32"], ["NormalizedSuspensionTravelFrontRight", "f32"],
  ["NormalizedSuspensionTravelRearLeft", "f32"], ["NormalizedSuspensionTravelRearRight", "f32"],
  ["TireSlipRatioFrontLeft", "f32"], ["TireSlipRatioFrontRight", "f32"],
  ["TireSlipRatioRearLeft", "f32"], ["TireSlipRatioRearRight", "f32"],
  ["WheelRotationSpeedFrontLeft", "f32"], ["WheelRotationSpeedFrontRight", "f32"],
  ["WheelRotationSpeedRearLeft", "f32"], ["WheelRotationSpeedRearRight", "f32"],
  ["WheelOnRumbleStripFrontLeft", "s32"], ["WheelOnRumbleStripFrontRight", "s32"],
  ["WheelOnRumbleStripRearLeft", "s32"], ["WheelOnRumbleStripRearRight", "s32"],
  ["WheelInPuddleDepthFrontLeft", "f32"], ["WheelInPuddleDepthFrontRight", "f32"],
  ["WheelInPuddleDepthRearLeft", "f32"], ["WheelInPuddleDepthRearRight", "f32"],
  ["SurfaceRumbleFrontLeft", "f32"], ["SurfaceRumbleFrontRight", "f32"],
  ["SurfaceRumbleRearLeft", "f32"], ["SurfaceRumbleRearRight", "f32"],
  ["TireSlipAngleFrontLeft", "f32"], ["TireSlipAngleFrontRight", "f32"],
  ["TireSlipAngleRearLeft", "f32"], ["TireSlipAngleRearRight", "f32"],
  ["TireCombinedSlipFrontLeft", "f32"], ["TireCombinedSlipFrontRight", "f32"], // 摩擦/抓地
  ["TireCombinedSlipRearLeft", "f32"], ["TireCombinedSlipRearRight", "f32"],
  ["SuspensionTravelMetersFrontLeft", "f32"], ["SuspensionTravelMetersFrontRight", "f32"],
  ["SuspensionTravelMetersRearLeft", "f32"], ["SuspensionTravelMetersRearRight", "f32"],
  ["CarOrdinal", "s32"], ["CarClass", "s32"], ["CarPerformanceIndex", "s32"],
  ["DrivetrainType", "s32"], ["NumCylinders", "s32"],

  // FH6 Horizon 扩展块（类型为推断，仪表用不到）
  ["CarGroup", "s32"], ["SmashableVelDiff", "f32"], ["SmashableMass", "f32"],

  // Dash 段
  ["PositionX", "f32"], ["PositionY", "f32"], ["PositionZ", "f32"],
  ["Speed", "f32"],   // m/s
  ["Power", "f32"],   // W
  ["Torque", "f32"],  // Nm
  ["TireTempFrontLeft", "f32"], ["TireTempFrontRight", "f32"],
  ["TireTempRearLeft", "f32"], ["TireTempRearRight", "f32"],
  ["Boost", "f32"], ["Fuel", "f32"], ["DistanceTraveled", "f32"],
  ["BestLap", "f32"], ["LastLap", "f32"], ["CurrentLap", "f32"], ["CurrentRaceTime", "f32"],
  ["LapNumber", "u16"], ["RacePosition", "u8"],
  ["Accel", "u8"], ["Brake", "u8"], ["Clutch", "u8"], ["HandBrake", "u8"], ["Gear", "u8"],
  ["Steer", "s8"], ["NormalizedDrivingLine", "s8"], ["NormalizedAIBrakeDifference", "s8"],
  // 323: 1 字节末尾填充（不读）
];

export const FIELDS: FieldDef[] = (() => {
  let offset = 0;
  return SCHEMA.map(([name, type]) => {
    const def = { name, type, offset };
    offset += SIZE[type];
    return def;
  });
})();

export type ForzaTelemetry = Record<string, number>;

export function parseForzaPacket(buf: Buffer): ForzaTelemetry {
  const out: ForzaTelemetry = {};
  for (const { name, type, offset } of FIELDS) {
    switch (type) {
      case "s32": out[name] = buf.readInt32LE(offset); break;
      case "u32": out[name] = buf.readUInt32LE(offset); break;
      case "f32": out[name] = buf.readFloatLE(offset); break;
      case "u16": out[name] = buf.readUInt16LE(offset); break;
      case "u8":  out[name] = buf.readUInt8(offset); break;
      case "s8":  out[name] = buf.readInt8(offset); break;
    }
  }
  return out;
}
