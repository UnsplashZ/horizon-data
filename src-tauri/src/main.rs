// FH6 透明覆盖层后端（多窗口）：
// - 主表 + 输入/抓地/G力/胎温 各自独立透明置顶窗口，可分别拖动定位、拖角缩放
// - 监听 UDP Data Out，解析 324B，广播 "telemetry"（端口热重绑）
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
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const PACKET_SIZE: usize = 324;
static EDITING: AtomicBool = AtomicBool::new(false);
const OVERLAY_WINDOWS: [&str; 5] = ["main", "inputs", "grip", "gforce", "tiretemp"];
const MODULES: [&str; 4] = ["inputs", "grip", "gforce", "tiretemp"];

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
    tire_temp: [f32; 4],
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
    show_tiretemp: bool,
    pos_main: [i32; 2], // 物理像素；[-1,-1]=首次自动排布
    pos_inputs: [i32; 2],
    pos_grip: [i32; 2],
    pos_gforce: [i32; 2],
    pos_tiretemp: [i32; 2],
    size_main: [u32; 2], // 物理像素；[0,0]=用配置默认尺寸
    size_inputs: [u32; 2],
    size_grip: [u32; 2],
    size_gforce: [u32; 2],
    size_tiretemp: [u32; 2],
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
            show_tiretemp: false,
            pos_main: [-1, -1],
            pos_inputs: [-1, -1],
            pos_grip: [-1, -1],
            pos_gforce: [-1, -1],
            pos_tiretemp: [-1, -1],
            size_main: [0, 0],
            size_inputs: [0, 0],
            size_grip: [0, 0],
            size_gforce: [0, 0],
            size_tiretemp: [0, 0],
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

fn pos_of<'a>(c: &'a mut Config, label: &str) -> Option<&'a mut [i32; 2]> {
    match label {
        "main" => Some(&mut c.pos_main),
        "inputs" => Some(&mut c.pos_inputs),
        "grip" => Some(&mut c.pos_grip),
        "gforce" => Some(&mut c.pos_gforce),
        "tiretemp" => Some(&mut c.pos_tiretemp),
        _ => None,
    }
}
fn size_of<'a>(c: &'a mut Config, label: &str) -> Option<&'a mut [u32; 2]> {
    match label {
        "main" => Some(&mut c.size_main),
        "inputs" => Some(&mut c.size_inputs),
        "grip" => Some(&mut c.size_grip),
        "gforce" => Some(&mut c.size_gforce),
        "tiretemp" => Some(&mut c.size_tiretemp),
        _ => None,
    }
}
fn module_shown(c: &Config, label: &str) -> bool {
    match label {
        "main" => true,
        "inputs" => c.show_inputs,
        "grip" => c.show_grip,
        "gforce" => c.show_gforce,
        "tiretemp" => c.show_tiretemp,
        _ => false,
    }
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(app: tauri::AppHandle, state: State<AppState>, config: Config) -> Result<(), String> {
    // 位置/尺寸只由拖动写入，设置面板更新不得覆盖
    let merged = {
        let mut c = state.config.lock().unwrap();
        let positions = (c.pos_main, c.pos_inputs, c.pos_grip, c.pos_gforce, c.pos_tiretemp);
        let sizes = (c.size_main, c.size_inputs, c.size_grip, c.size_gforce, c.size_tiretemp);
        *c = config;
        c.pos_main = positions.0;
        c.pos_inputs = positions.1;
        c.pos_grip = positions.2;
        c.pos_gforce = positions.3;
        c.pos_tiretemp = positions.4;
        c.size_main = sizes.0;
        c.size_inputs = sizes.1;
        c.size_grip = sizes.2;
        c.size_gforce = sizes.3;
        c.size_tiretemp = sizes.4;
        c.clone()
    };
    save_config_file(&merged)?;
    state.desired_port.store(merged.port, Ordering::Relaxed);
    apply_module_visibility(&app, &merged);
    let _ = app.emit("config", &merged);
    // 显示新窗口会抢焦点：编辑态下把焦点交回设置窗，方便连续勾选
    if EDITING.load(Ordering::Relaxed) {
        if let Some(s) = app.get_webview_window("settings") {
            let _ = s.set_focus();
        }
    }
    Ok(())
}

#[tauri::command]
fn save_window_pos(app: tauri::AppHandle, state: State<AppState>, label: String, x: i32, y: i32) {
    let merged = {
        let mut c = state.config.lock().unwrap();
        if let Some(p) = pos_of(&mut c, &label) {
            *p = [x, y];
        }
        c.clone()
    };
    let _ = save_config_file(&merged);
    let _ = app.emit("config", &merged);
}

#[tauri::command]
fn save_window_size(app: tauri::AppHandle, state: State<AppState>, label: String, w: u32, h: u32) {
    let merged = {
        let mut c = state.config.lock().unwrap();
        if let Some(s) = size_of(&mut c, &label) {
            *s = [w, h];
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
        tire_slip: [read_f32(b, 180), read_f32(b, 184), read_f32(b, 188), read_f32(b, 192)],
        tire_temp: [read_f32(b, 268), read_f32(b, 272), read_f32(b, 276), read_f32(b, 280)],
        speed_kmh: read_f32(b, 256) * 3.6,
        accel: b[315],
        brake: b[316],
        gear: b[319],
        steer: b[320] as i8,
    }
}

fn apply_module_visibility(app: &tauri::AppHandle, c: &Config) {
    for label in MODULES {
        if let Some(w) = app.get_webview_window(label) {
            if module_shown(c, label) {
                let _ = w.show();
            } else {
                let _ = w.hide();
            }
        }
    }
}

/// 放置窗口：套用保存的位置/尺寸；位置缺省时按 (dx,dy) 逻辑偏移在主显示器排布
fn place_window(app: &tauri::AppHandle, c: &mut Config, label: &str, dx: f64, dy: f64) {
    if let Some(w) = app.get_webview_window(label) {
        let size = size_of(c, label).copied().unwrap_or([0, 0]);
        if size[0] > 0 {
            let _ = w.set_size(PhysicalSize::new(size[0], size[1]));
        }
        let pos = pos_of(c, label).copied().unwrap_or([-1, -1]);
        if pos[0] >= 0 {
            let _ = w.set_position(PhysicalPosition::new(pos[0], pos[1]));
            return;
        }
        if let Ok(Some(mon)) = w.current_monitor() {
            let scale = mon.scale_factor();
            let mp = mon.position();
            let ms = mon.size();
            let ws = w.outer_size().unwrap_or(PhysicalSize::new(400, 200));
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
    let mut cfg = load_config_file();
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
            save_window_size,
            quit_app
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            place_window(&handle, &mut cfg, "main", 0.0, 40.0);
            place_window(&handle, &mut cfg, "inputs", -450.0, 300.0);
            place_window(&handle, &mut cfg, "grip", -150.0, 300.0);
            place_window(&handle, &mut cfg, "gforce", 150.0, 300.0);
            place_window(&handle, &mut cfg, "tiretemp", 450.0, 300.0);

            for label in OVERLAY_WINDOWS {
                if let Some(w) = app.get_webview_window(label) {
                    let _ = w.set_ignore_cursor_events(true);
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
