// FH6 透明覆盖层后端（多窗口）：
// - 主表 + 输入/抓地/G力 各自独立透明置顶窗口，可分别拖动定位
// - 监听 UDP Data Out，解析 324B，广播 "telemetry" 给所有窗口（端口热重绑）
// - 全局快捷键 Cmd/Ctrl+Shift+H 切换 编辑/锁定（锁定=全部点击穿透），编辑态显示设置窗
// - 配置存 exe 同目录 config.json（便携/绿色版友好）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager, PhysicalPosition, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const PACKET_SIZE: usize = 324;
static EDITING: AtomicBool = AtomicBool::new(false);
/// 可点击穿透/可拖动的覆盖窗口（不含 settings）
const OVERLAY_WINDOWS: [&str; 4] = ["main", "inputs", "grip", "gforce"];

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

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Config {
    port: u16,
    bg_opacity: f32,
    fg_opacity: f32,
    units: String,
    show_inputs: bool,
    show_grip: bool,
    show_gforce: bool,
    pos_main: [i32; 2], // 物理像素；[-1,-1] 表示首次启动自动居中
    pos_inputs: [i32; 2],
    pos_grip: [i32; 2],
    pos_gforce: [i32; 2],
}
impl Default for Config {
    fn default() -> Self {
        Self {
            port: 5300,
            bg_opacity: 0.72,
            fg_opacity: 1.0,
            units: "kmh".into(),
            show_inputs: false,
            show_grip: false,
            show_gforce: false,
            pos_main: [-1, -1],
            pos_inputs: [-1, -1],
            pos_grip: [-1, -1],
            pos_gforce: [-1, -1],
        }
    }
}

struct AppState {
    config: Mutex<Config>,
    desired_port: Arc<AtomicU16>,
}

fn config_path() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join("horizon-data-config.json")
}
fn load_config_file() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
fn save_config_file(c: &Config) -> Result<(), String> {
    let s = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), s).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(app: tauri::AppHandle, state: State<AppState>, config: Config) -> Result<(), String> {
    // 位置只由拖动(save_window_pos)写入，设置面板的更新不得覆盖
    let merged = {
        let mut c = state.config.lock().unwrap();
        let pos = (c.pos_main, c.pos_inputs, c.pos_grip, c.pos_gforce);
        *c = config;
        c.pos_main = pos.0;
        c.pos_inputs = pos.1;
        c.pos_grip = pos.2;
        c.pos_gforce = pos.3;
        c.clone()
    };
    save_config_file(&merged)?;
    state.desired_port.store(merged.port, Ordering::Relaxed);
    apply_module_visibility(&app, &merged);
    let _ = app.emit("config", &merged);
    Ok(())
}

#[tauri::command]
fn save_window_pos(app: tauri::AppHandle, state: State<AppState>, label: String, x: i32, y: i32) {
    let merged = {
        let mut c = state.config.lock().unwrap();
        match label.as_str() {
            "main" => c.pos_main = [x, y],
            "inputs" => c.pos_inputs = [x, y],
            "grip" => c.pos_grip = [x, y],
            "gforce" => c.pos_gforce = [x, y],
            _ => {}
        }
        c.clone()
    };
    let _ = save_config_file(&merged);
    let _ = app.emit("config", &merged);
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

fn module_shown(c: &Config, label: &str) -> bool {
    match label {
        "main" => true,
        "inputs" => c.show_inputs,
        "grip" => c.show_grip,
        "gforce" => c.show_gforce,
        _ => false,
    }
}
fn apply_module_visibility(app: &tauri::AppHandle, c: &Config) {
    for label in ["inputs", "grip", "gforce"] {
        if let Some(w) = app.get_webview_window(label) {
            if module_shown(c, label) {
                let _ = w.show();
            } else {
                let _ = w.hide();
            }
        }
    }
}

/// 放置窗口：有保存位置用之，否则按 (dx,dy) 逻辑偏移在主显示器上排布
fn place_window(app: &tauri::AppHandle, label: &str, pos: [i32; 2], dx: f64, dy: f64) {
    if let Some(w) = app.get_webview_window(label) {
        if pos[0] >= 0 {
            let _ = w.set_position(PhysicalPosition::new(pos[0], pos[1]));
            return;
        }
        if let Ok(Some(mon)) = w.current_monitor() {
            let scale = mon.scale_factor();
            let mp = mon.position();
            let ms = mon.size();
            let ws = w
                .outer_size()
                .unwrap_or(tauri::PhysicalSize::new(400, 200));
            let cx = mp.x + (ms.width as i32 - ws.width as i32) / 2 + (dx * scale) as i32;
            let cy = mp.y + (dy * scale) as i32;
            let _ = w.set_position(PhysicalPosition::new(cx, cy));
        }
    }
}

fn toggle_edit(app: &tauri::AppHandle) {
    let editing = !EDITING.load(Ordering::Relaxed);
    EDITING.store(editing, Ordering::Relaxed);
    for label in OVERLAY_WINDOWS {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.set_ignore_cursor_events(!editing);
        }
    }
    if let Some(s) = app.get_webview_window("settings") {
        if editing {
            let _ = s.show();
            let _ = s.set_focus();
        } else {
            let _ = s.hide();
        }
    }
    let _ = app.emit("edit-mode", editing);
}

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

fn main() {
    let cfg = load_config_file();
    let desired_port = Arc::new(AtomicU16::new(cfg.port));
    let state = AppState {
        config: Mutex::new(cfg.clone()),
        desired_port: desired_port.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            save_window_pos,
            quit_app
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // 排布并显示各窗口（默认错开摆放，用户可拖动）
            place_window(&handle, "main", cfg.pos_main, 0.0, 40.0);
            place_window(&handle, "inputs", cfg.pos_inputs, -300.0, 300.0);
            place_window(&handle, "grip", cfg.pos_grip, 0.0, 300.0);
            place_window(&handle, "gforce", cfg.pos_gforce, 300.0, 300.0);

            for label in OVERLAY_WINDOWS {
                if let Some(w) = app.get_webview_window(label) {
                    let _ = w.set_ignore_cursor_events(true); // 启动即锁定
                    if module_shown(&cfg, label) {
                        let _ = w.show();
                    }
                }
            }

            app.global_shortcut()
                .on_shortcut("CmdOrCtrl+Shift+H", |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_edit(app);
                    }
                })?;

            spawn_udp_listener(handle, desired_port.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用出错");
}
