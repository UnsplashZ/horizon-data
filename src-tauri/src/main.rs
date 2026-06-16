// FH6 透明覆盖层后端：监听 UDP Data Out，解析 324 字节包，emit "telemetry" 事件给前端。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::net::UdpSocket;
use tauri::{Emitter, Manager};

/// FH6 / FH5 Car Dash 包固定大小
const PACKET_SIZE: usize = 324;
/// 监听端口（与游戏 Data Out 设置一致）。后续做成可配置。
const UDP_PORT: u16 = 5300;

/// 推送给前端的精简遥测（字段与 src/App.vue 的 Telemetry 对应）
#[derive(Serialize, Clone)]
struct Telemetry {
    is_race_on: bool,
    rpm: f32,
    max_rpm: f32,
    speed_kmh: f32,
    gear: u8,
    accel: u8,
    brake: u8,
    steer: i8,
    accel_x: f32,
    accel_z: f32,
    tire_slip: [f32; 4],
}

#[inline]
fn read_f32(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

#[inline]
fn read_i32(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// 偏移见 reference/forza_packet.py
fn parse(b: &[u8]) -> Telemetry {
    Telemetry {
        is_race_on: read_i32(b, 0) == 1,
        max_rpm: read_f32(b, 8),
        rpm: read_f32(b, 16),
        accel_x: read_f32(b, 20),
        accel_z: read_f32(b, 28),
        tire_slip: [
            read_f32(b, 180),
            read_f32(b, 184),
            read_f32(b, 188),
            read_f32(b, 192),
        ],
        speed_kmh: read_f32(b, 256) * 3.6,
        accel: b[315],
        brake: b[316],
        gear: b[319],
        steer: b[320] as i8,
    }
}

/// 切换鼠标点击穿透：true=穿透（锁定显示），false=可交互（编辑/拖动）
#[tauri::command]
fn set_click_through(window: tauri::Window, enabled: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())
}

fn spawn_udp_listener(app: tauri::AppHandle, port: u16) {
    std::thread::spawn(move || {
        let socket = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[udp] 绑定 0.0.0.0:{port} 失败: {e}");
                return;
            }
        };
        println!("[udp] 监听 0.0.0.0:{port}");
        let mut buf = [0u8; 1500];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, _)) if n == PACKET_SIZE => {
                    let _ = app.emit("telemetry", parse(&buf));
                }
                Ok(_) => { /* 非 324 字节：别的游戏/格式，忽略 */ }
                Err(e) => eprintln!("[udp] recv 错误: {e}"),
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_click_through])
        .setup(|app| {
            // 启动即默认点击穿透，避免覆盖层挡住游戏操作
            if let Some(win) = app.get_webview_window("overlay") {
                let _ = win.set_ignore_cursor_events(true);
            }
            spawn_udp_listener(app.handle().clone(), UDP_PORT);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用出错");
}
