/**
 * FH6 遥测链路验证工具：监听 UDP，实时打印 rpm / 速度 / 档位 / 输入。
 *
 * 运行（Node 24+，自带 TS 支持）：
 *   node tools/udp-listen.ts            # 默认端口 5300
 *   node tools/udp-listen.ts 5300       # 指定端口
 *   PORT=5300 node tools/udp-listen.ts
 *
 * 在 Windows 的 FH6 里把 Data Out 目标 IP 填本机/本工具所在机器的局域网 IP，
 * 端口与此处一致，进入比赛即可看到数据滚动。
 */

import dgram from "node:dgram";
import os from "node:os";
import { parseForzaPacket, FH6_PACKET_SIZE } from "../reference/forzaPacket.ts";

const PORT = Number(process.argv[2] ?? process.env.PORT ?? 5300);
const HOST = "0.0.0.0";

/** 列出本机所有非内网回环的 IPv4，便于在游戏里填对地址 */
function lanIPv4(): string[] {
  const out: string[] = [];
  for (const addrs of Object.values(os.networkInterfaces())) {
    for (const a of addrs ?? []) {
      if (a.family === "IPv4" && !a.internal) out.push(`${a.address}`);
    }
  }
  return out;
}

function gearLabel(g: number): string {
  if (g === 0) return "R";
  if (g === 11) return "N";
  return String(g);
}

const bar = (v01: number, width = 10) => {
  const n = Math.round(Math.max(0, Math.min(1, v01)) * width);
  return "█".repeat(n) + "·".repeat(width - n);
};

const sock = dgram.createSocket("udp4");

let total = 0;
let inWindow = 0;
let lastSize = 0;
let warnedSize = false;

sock.on("message", (buf) => {
  total++;
  inWindow++;
  lastSize = buf.length;

  if (buf.length !== FH6_PACKET_SIZE) {
    if (!warnedSize) {
      warnedSize = true;
      process.stdout.write(
        `\n⚠️  收到 ${buf.length} 字节包（期望 ${FH6_PACKET_SIZE}）。` +
          ` 可能是别的游戏/格式：FM7=311, FM2023=331, Sled=232。\n`,
      );
    }
    return;
  }

  const t = parseForzaPacket(buf);
  const racing = t.IsRaceOn === 1;
  const rpm = Math.round(t.CurrentEngineRpm);
  const maxRpm = Math.round(t.EngineMaxRpm);
  const kmh = (t.Speed * 3.6).toFixed(0).padStart(3);
  const gear = gearLabel(t.Gear).padStart(2);
  const thr = bar(t.Accel / 255);
  const brk = bar(t.Brake / 255);
  const steer = (t.Steer / 127).toFixed(2).padStart(5); // -1..1

  // 单行原地刷新
  process.stdout.write(
    `\r${racing ? "🟢" : "⚪"} ` +
      `RPM ${String(rpm).padStart(5)}/${maxRpm}  ` +
      `${kmh} km/h  ` +
      `档 ${gear}  ` +
      `油门 ${thr}  刹车 ${brk}  ` +
      `转向 ${steer}   `,
  );
});

sock.on("error", (err) => {
  console.error(`socket 错误: ${err.message}`);
  if ((err as NodeJS.ErrnoException).code === "EADDRINUSE") {
    console.error(`端口 ${PORT} 被占用，换一个端口或关掉占用程序。`);
  }
  sock.close();
  process.exit(1);
});

sock.bind(PORT, HOST, () => {
  const ips = lanIPv4();
  console.log(`▶ 监听 UDP ${HOST}:${PORT}`);
  console.log(
    `  在 FH6 的 Data Out 里把 IP 填：${ips.length ? ips.join(" 或 ") : "本机局域网 IP"}，端口 ${PORT}`,
  );
  console.log(`  等待数据中…（进入比赛/自由驾驶后开始；IsRaceOn=0 时为菜单/暂停）\n`);
});

// 每 2 秒在新行打印一次包速率，便于确认链路活着
setInterval(() => {
  const pps = inWindow / 2;
  inWindow = 0;
  if (total > 0) {
    process.stdout.write(
      `\n   [${new Date().toLocaleTimeString()}] 累计 ${total} 包, ~${pps.toFixed(0)}/s, 末包 ${lastSize}B\n`,
    );
  }
}, 2000);

process.on("SIGINT", () => {
  console.log(`\n收到 ${total} 个包，退出。`);
  sock.close();
  process.exit(0);
});
