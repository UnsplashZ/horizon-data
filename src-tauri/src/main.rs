// FH6 透明覆盖层后端：
// - 监听 UDP Data Out，解析 324B 包，emit "telemetry" 给前端（端口可热重绑）
// - 全局快捷键 Cmd/Ctrl+Shift+H 切换 编辑/锁定（锁定=点击穿透）
// - 配置存到 exe 同目录 config.json（便携/绿色版友好）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const PACKET_SIZE: usize = 324;
static EDITING: AtomicBool = AtomicBool::new(false);

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

/// 持久化配置（存 exe 同目录）
#[derive(Serialize, Deserialize, Clone)]
struct Config {
    port: u16,
    opacity: f32,
    hud_x: f64, // <0 表示首次启动，前端用屏幕宽度居中
    hud_y: f64,
    units: String, // "kmh" | "mph"
}
impl Default for Config {
    fn default() -> Self {
        Self {
            port: 5300,
            opacity: 0.92,
            hud_x: -1.0,
            hud_y: 28.0,
            units: "kmh".into(),
        }
    }
}

fn config_path() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join("horizon-data-config.json")
}

#[tauri::command]
fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    let s = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), s).map_err(|e| e.to_string())
}

struct PortState(Arc<AtomicU16>);

#[tauri::command]
fn set_port(port: u16, state: State<PortState>) {
    state.0.store(port, Ordering::Relaxed);
}

#[tauri::command]
fn set_click_through(window: tauri::Window, enabled: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
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

/// 监听线程：用读超时轮询 desired 端口，端口变化时热重绑
fn spawn_udp_listener(app: tauri::AppHandle, desired: Arc<AtomicU16>) {
    std::thread::spawn(move || {
        let mut current: u16 = 0;
        let mut socket: Option<UdpSocket> = None;
        let mut buf = [0u8; 1500];
        loop {
            let want = desired.load(Ordering::Relaxed);
            if socket.is_none() || want != current {
                match UdpSocket::bind(("0.0.0.0", want)) {
                    Ok(s) => {
                        let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
                        println!("[udp] 监听 0.0.0.0:{want}");
                        socket = Some(s);
                        current = want;
                    }
                    Err(e) => {
                        eprintln!("[udp] 绑定 {want} 失败: {e}");
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
            if let Some(s) = &socket {
                match s.recv_from(&mut buf) {
                    Ok((n, _)) if n == PACKET_SIZE => {
                        let _ = app.emit("telemetry", parse(&buf));
                    }
                    Ok(_) => {}
                    Err(ref e)
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
                    Err(e) => eprintln!("[udp] recv 错误: {e}"),
                }
            }
        }
    });
}

/// 切换编辑/锁定：编辑=可交互(关穿透)，锁定=点击穿透(纯显示)
fn toggle_edit(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let editing = !EDITING.load(Ordering::Relaxed);
        EDITING.store(editing, Ordering::Relaxed);
        let _ = win.set_ignore_cursor_events(!editing);
        let _ = win.emit("edit-mode", editing);
        if editing {
            let _ = win.set_focus();
        }
    }
}

fn main() {
    let desired_port = Arc::new(AtomicU16::new(load_config().port));

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(PortState(desired_port.clone()))
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            set_port,
            set_click_through,
            quit_app
        ])
        .setup(move |app| {
            // 覆盖整个主显示器，HUD 可在其中任意定位
            if let Some(win) = app.get_webview_window("overlay") {
                if let Ok(Some(monitor)) = win.current_monitor() {
                    let _ = win.set_position(*monitor.position());
                    let _ = win.set_size(*monitor.size());
                }
                let _ = win.set_ignore_cursor_events(true); // 启动即锁定
            }

            // 全局快捷键：Cmd/Ctrl+Shift+H 切换编辑/锁定
            app.global_shortcut()
                .on_shortcut("CmdOrCtrl+Shift+H", |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_edit(app);
                    }
                })?;

            spawn_udp_listener(app.handle().clone(), desired_port.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用出错");
}
