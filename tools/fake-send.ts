/**
 * 本地自测：伪造一个 324 字节 FH6 包发到 127.0.0.1，验证 udp-listen 解析正确。
 * 用法：先开 `node tools/udp-listen.ts`，再另开终端 `node tools/fake-send.ts`
 */
import dgram from "node:dgram";
import { FIELDS, FH6_PACKET_SIZE } from "../reference/forzaPacket.ts";

const PORT = Number(process.argv[2] ?? process.env.PORT ?? 10989);

const values: Record<string, number> = {
  IsRaceOn: 1,
  EngineMaxRpm: 9000,
  CurrentEngineRpm: 7647,
  Speed: 138 / 3.6, // m/s
  Gear: 1,
  Accel: 255,
  Brake: 0,
  Steer: 32, // ~0.25
};

const buf = Buffer.alloc(FH6_PACKET_SIZE);
for (const { name, type, offset } of FIELDS) {
  const v = values[name] ?? 0;
  switch (type) {
    case "s32": buf.writeInt32LE(v | 0, offset); break;
    case "u32": buf.writeUInt32LE(v >>> 0, offset); break;
    case "f32": buf.writeFloatLE(v, offset); break;
    case "u16": buf.writeUInt16LE(v & 0xffff, offset); break;
    case "u8": buf.writeUInt8(v & 0xff, offset); break;
    case "s8": buf.writeInt8(((v + 128) % 256) - 128, offset); break;
  }
}

const sock = dgram.createSocket("udp4");
let n = 0;
const timer = setInterval(() => {
  sock.send(buf, PORT, "127.0.0.1");
  if (++n >= 20) {
    clearInterval(timer);
    sock.close();
    console.log(`已发送 ${n} 个伪造包到 127.0.0.1:${PORT}`);
  }
}, 100);
