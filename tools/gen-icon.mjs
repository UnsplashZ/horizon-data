// 生成一个占位 app 图标（深底 + 转速环），输出 icon-src.png。
// 之后用 `npx tauri icon icon-src.png` 生成各平台图标。无第三方依赖。
import { writeFileSync } from "node:fs";
import zlib from "node:zlib";

const S = 512;
const cx = S / 2, cy = S / 2;
const rOuter = 210, rInner = 168;

function px(x, y) {
  const dx = x - cx, dy = y - cy;
  const r = Math.hypot(dx, dy);
  // 角度：从底部左侧扫到右侧（仪表盘风格，约 240°）
  let ang = Math.atan2(dy, dx); // -PI..PI
  // 背景圆角深底
  const bg = [18, 22, 28, 255];
  if (r > rOuter + 6) {
    // 圆外透明
    return [0, 0, 0, 0];
  }
  if (r >= rInner && r <= rOuter) {
    // 表环：左(绿)->右(红) 渐变，底部缺口
    const a = (ang + Math.PI) / (2 * Math.PI); // 0..1 顺时针从左
    const gapLo = 0.5 - 0.13, gapHi = 0.5 + 0.13; // 底部缺口
    if (a > gapLo && a < gapHi) return bg;
    // 沿环映射颜色
    const tcol = a < 0.5 ? a / gapLo : (a - gapHi) / (1 - gapHi);
    const g = [54, 211, 153], y = [255, 209, 102], rr = [255, 90, 77];
    const lerp = (p, q, t) => Math.round(p + (q - p) * t);
    let c;
    if (tcol < 0.6) {
      const t = tcol / 0.6;
      c = [lerp(g[0], y[0], t), lerp(g[1], y[1], t), lerp(g[2], y[2], t)];
    } else {
      const t = (tcol - 0.6) / 0.4;
      c = [lerp(y[0], rr[0], t), lerp(y[1], rr[1], t), lerp(y[2], rr[2], t)];
    }
    return [c[0], c[1], c[2], 255];
  }
  return bg;
}

// 构建 RGBA 原始数据（每行前置 filter 字节 0）
const raw = Buffer.alloc((S * 4 + 1) * S);
let o = 0;
for (let y = 0; y < S; y++) {
  raw[o++] = 0;
  for (let x = 0; x < S; x++) {
    const [r, g, b, a] = px(x, y);
    raw[o++] = r; raw[o++] = g; raw[o++] = b; raw[o++] = a;
  }
}

function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td) >>> 0);
  return Buffer.concat([len, td, crc]);
}
function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c;
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(S, 0); ihdr.writeUInt32BE(S, 4);
ihdr[8] = 8; ihdr[9] = 6; // 8-bit RGBA
const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw)),
  chunk("IEND", Buffer.alloc(0)),
]);
writeFileSync("icon-src.png", png);
console.log("wrote icon-src.png", png.length, "bytes");
