// FH6 透明覆盖层后端：
// - 主 HUD 窗口 + 状态栏/托盘控制入口
// - 监听 UDP Data Out，解析 324B，广播 "telemetry"（端口热重绑）
// - 全局快捷键 Cmd/Ctrl+Shift+H 仅切换 HUD 编辑/锁定，用于调整位置和尺寸
// - 配置优先存 exe 同目录，安装目录不可写时落到用户配置目录
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const PACKET_SIZE: usize = 324;
static EDITING: AtomicBool = AtomicBool::new(false);
const OVERLAY_WINDOWS: [&str; 1] = ["main"];

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

#[derive(Serialize, Clone, PartialEq)]
struct UdpStatus {
    port: u16,
    listening: bool,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct ShortcutStatus {
    registered: bool,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Config {
    port: u16,
    bg_opacity: f32,
    fg_opacity: f32,
    units: String,
    show_tires: bool,  // 统一仪表盘中的轮胎模块
    show_inputs: bool, // 统一仪表盘中的输入模块
    show_gforce: bool, // 统一仪表盘中的G力模块
    pos_main: [i32; 2],
    size_main: [u32; 2],
}
impl Default for Config {
    fn default() -> Self {
        Self {
            port: 10989,
            bg_opacity: 0.72,
            fg_opacity: 1.0,
            units: "kmh".into(),
            show_tires: true,
            show_inputs: true,
            show_gforce: true,
            pos_main: [-1, -1],
            size_main: [0, 0],
        }
    }
}

struct AppState {
    config: Mutex<Config>,
    desired_port: Arc<AtomicU16>,
    udp_status: Arc<Mutex<UdpStatus>>,
    shortcut_status: Mutex<ShortcutStatus>,
}

fn config_path() -> PathBuf {
    let portable = portable_config_path();

    if let Some(path) = &portable {
        if (path.exists() && is_writable_file(path))
            || (!path.exists() && path.parent().is_some_and(is_writable_dir))
        {
            return path.clone();
        }
    }

    user_config_dir().join("horizon-data-config.json")
}

fn portable_config_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("horizon-data-config.json")))
}

fn user_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("horizon-data");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("horizon-data");
        }
    }

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("horizon-data");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("horizon-data");
    }
    PathBuf::from(".").join("horizon-data")
}

fn is_writable_dir(dir: &Path) -> bool {
    let probe = dir.join(".horizon-data-write-test");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn is_writable_file(path: &Path) -> bool {
    OpenOptions::new().append(true).open(path).is_ok()
}

fn load_config_file() -> Config {
    let writable = config_path();
    let mut candidates = vec![writable.clone()];
    if let Some(portable) = portable_config_path() {
        if portable != writable {
            candidates.push(portable);
        }
    }

    for path in candidates {
        if let Some(config) = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            return normalize_config(config);
        }
    }

    Config::default()
}

fn normalize_config(mut c: Config) -> Config {
    c.port = c.port.max(1);
    c.bg_opacity = clamp01(c.bg_opacity);
    c.fg_opacity = clamp01(c.fg_opacity);
    if c.units != "mph" {
        c.units = "kmh".into();
    }
    c
}

fn save_config_file(c: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let s = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    std::fs::write(path, s).map_err(|e| e.to_string())
}

fn pos_of<'a>(c: &'a mut Config, label: &str) -> Option<&'a mut [i32; 2]> {
    match label {
        "main" => Some(&mut c.pos_main),
        _ => None,
    }
}
fn size_of<'a>(c: &'a mut Config, label: &str) -> Option<&'a mut [u32; 2]> {
    match label {
        "main" => Some(&mut c.size_main),
        _ => None,
    }
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(
    app: tauri::AppHandle,
    state: State<AppState>,
    config: Config,
) -> Result<(), String> {
    let merged = {
        let mut c = state.config.lock().unwrap();
        let pos_main = c.pos_main;
        let size_main = c.size_main;
        let mut next = normalize_config(config);
        next.pos_main = pos_main;
        next.size_main = size_main;
        save_config_file(&next)?;
        *c = next;
        c.clone()
    };
    state.desired_port.store(merged.port, Ordering::Relaxed);
    let _ = app.emit("config", &merged);
    Ok(())
}

#[tauri::command]
fn get_udp_status(state: State<AppState>) -> UdpStatus {
    state.udp_status.lock().unwrap().clone()
}

#[tauri::command]
fn get_shortcut_status(state: State<AppState>) -> ShortcutStatus {
    state.shortcut_status.lock().unwrap().clone()
}

#[tauri::command]
fn save_window_pos(app: tauri::AppHandle, state: State<AppState>, label: String, x: i32, y: i32) {
    let merged = {
        let mut c = state.config.lock().unwrap();
        if let Some(p) = pos_of(&mut c, &label) {
            *p = [x, y];
        }
        if let Err(e) = save_config_file(&c) {
            eprintln!("[config] 保存窗口位置失败: {e}");
        }
        c.clone()
    };
    let _ = app.emit("config", &merged);
}

#[tauri::command]
fn save_window_size(app: tauri::AppHandle, state: State<AppState>, label: String, w: u32, h: u32) {
    let merged = {
        let mut c = state.config.lock().unwrap();
        if let Some(s) = size_of(&mut c, &label) {
            *s = [w, h];
        }
        if let Err(e) = save_config_file(&c) {
            eprintln!("[config] 保存窗口尺寸失败: {e}");
        }
        c.clone()
    };
    let _ = app.emit("config", &merged);
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
        tire_temp: [
            read_f32(b, 268),
            read_f32(b, 272),
            read_f32(b, 276),
            read_f32(b, 280),
        ],
        speed_kmh: read_f32(b, 256) * 3.6,
        accel: b[315],
        brake: b[316],
        gear: b[319],
        steer: b[320] as i8,
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
            // 水平：相对于屏幕中心，负数向左，正数向右
            let cx = mp.x + (ms.width as i32 - ws.width as i32) / 2 + (dx * scale) as i32;
            // 垂直：负数从底部向上，正数从顶部向下
            let cy = if dy < 0.0 {
                mp.y + ms.height as i32 - ws.height as i32 + (dy * scale) as i32
            } else {
                mp.y + (dy * scale) as i32
            };
            let _ = w.set_position(PhysicalPosition::new(cx, cy));
        }
    }
}

fn apply_edit_mode(app: &tauri::AppHandle, editing: bool) {
    EDITING.store(editing, Ordering::Relaxed);
    for label in OVERLAY_WINDOWS {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.set_ignore_cursor_events(!editing);
        }
    }
    let _ = app.emit("edit-mode", editing);
}

fn show_controls(app: &tauri::AppHandle, pos: Option<PhysicalPosition<f64>>) {
    if let Some(w) = app.get_webview_window("controls") {
        let next_pos = pos
            .and_then(|p| controls_position_for_click(&w, p))
            .or_else(|| default_controls_position(app, &w));
        if let Some(p) = next_pos {
            let _ = w.set_position(p);
        }
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn controls_window_size(w: &tauri::WebviewWindow) -> PhysicalSize<u32> {
    w.outer_size().unwrap_or(PhysicalSize::new(320, 356))
}

fn controls_position_for_click(
    w: &tauri::WebviewWindow,
    click: PhysicalPosition<f64>,
) -> Option<PhysicalPosition<i32>> {
    let area = work_area_for_point(w, click).or_else(|| {
        w.current_monitor()
            .ok()
            .flatten()
            .map(|monitor| *monitor.work_area())
    })?;
    let size = controls_window_size(w);
    Some(place_controls_near_click(area, size, click))
}

fn default_controls_position(
    app: &tauri::AppHandle,
    w: &tauri::WebviewWindow,
) -> Option<PhysicalPosition<i32>> {
    let area = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|monitor| *monitor.work_area())
        .or_else(|| {
            w.current_monitor()
                .ok()
                .flatten()
                .map(|monitor| *monitor.work_area())
        })?;
    let size = controls_window_size(w);
    let margin = 12;
    let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
    let y = area.position.y + margin;
    Some(PhysicalPosition::new(
        clamp_i32(
            x,
            area.position.x + margin,
            area.position.x + area.size.width as i32 - size.width as i32 - margin,
        ),
        clamp_i32(
            y,
            area.position.y + margin,
            area.position.y + area.size.height as i32 - size.height as i32 - margin,
        ),
    ))
}

fn work_area_for_point(
    w: &tauri::WebviewWindow,
    point: PhysicalPosition<f64>,
) -> Option<PhysicalRect<i32, u32>> {
    for monitor in w.available_monitors().ok()? {
        let area = *monitor.work_area();
        let left = area.position.x as f64;
        let top = area.position.y as f64;
        let right = left + area.size.width as f64;
        let bottom = top + area.size.height as f64;
        if point.x >= left && point.x <= right && point.y >= top && point.y <= bottom {
            return Some(area);
        }
    }
    None
}

fn place_controls_near_click(
    area: PhysicalRect<i32, u32>,
    size: PhysicalSize<u32>,
    click: PhysicalPosition<f64>,
) -> PhysicalPosition<i32> {
    let margin = 8;
    let width = size.width as i32;
    let height = size.height as i32;
    let left = area.position.x + margin;
    let top = area.position.y + margin;
    let right = area.position.x + area.size.width as i32 - width - margin;
    let bottom = area.position.y + area.size.height as i32 - height - margin;
    let click_x = click.x.round() as i32;
    let click_y = click.y.round() as i32;
    let below = click_y + margin;
    let above = click_y - height - margin;
    let preferred_y = if below + height <= area.position.y + area.size.height as i32 {
        below
    } else {
        above
    };

    PhysicalPosition::new(
        clamp_i32(click_x - width + margin, left, right),
        clamp_i32(preferred_y, top, bottom),
    )
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    if max < min {
        min
    } else {
        value.max(min).min(max)
    }
}

fn hide_controls(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("controls") {
        let _ = w.hide();
    }
}

fn show_hud(app: &tauri::AppHandle) {
    let editing = EDITING.load(Ordering::Relaxed);
    for label in OVERLAY_WINDOWS {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.show();
            let _ = w.set_ignore_cursor_events(!editing);
        }
    }
}

fn toggle_hud_visibility(app: &tauri::AppHandle) {
    let any_visible = OVERLAY_WINDOWS.iter().any(|label| {
        app.get_webview_window(label)
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false)
    });

    if any_visible {
        for label in OVERLAY_WINDOWS {
            if let Some(w) = app.get_webview_window(label) {
                let _ = w.hide();
            }
        }
    } else {
        show_hud(app);
    }
}

fn toggle_controls(app: &tauri::AppHandle, pos: Option<PhysicalPosition<f64>>) {
    if let Some(w) = app.get_webview_window("controls") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_controls(app, pos);
        }
    }
}

fn toggle_edit(app: &tauri::AppHandle) {
    let editing = !EDITING.load(Ordering::Relaxed);
    apply_edit_mode(app, editing);
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn bind_error_message(port: u16, error: &std::io::Error) -> String {
    match error.kind() {
        ErrorKind::AddrInUse => format!("端口 {port} 已被占用"),
        ErrorKind::PermissionDenied => format!("没有权限监听端口 {port}"),
        _ => format!("绑定端口 {port} 失败: {error}"),
    }
}

fn set_udp_status(app: &tauri::AppHandle, status_store: &Arc<Mutex<UdpStatus>>, next: UdpStatus) {
    let should_emit = {
        let mut current = status_store.lock().unwrap();
        if *current == next {
            false
        } else {
            *current = next.clone();
            true
        }
    };
    if should_emit {
        let _ = app.emit("udp-status", &next);
    }
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let open_controls =
        MenuItem::with_id(app, "open_controls", "打开控制面板", true, None::<&str>)?;
    let toggle_hud = MenuItem::with_id(app, "toggle_hud", "隐藏/显示 HUD", true, None::<&str>)?;
    let edit_layout = MenuItem::with_id(app, "edit_layout", "编辑 HUD 布局", true, None::<&str>)?;
    let lock_hud = MenuItem::with_id(app, "lock_hud", "锁定 HUD", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open_controls,
            &toggle_hud,
            &edit_layout,
            &lock_hud,
            &sep,
            &quit,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("horizon-data")
        .menu(&menu)
        .tooltip("horizon-data")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id();
            if id == "open_controls" {
                show_controls(app, None);
            } else if id == "toggle_hud" {
                toggle_hud_visibility(app);
            } else if id == "edit_layout" {
                show_hud(app);
                apply_edit_mode(app, true);
                show_controls(app, None);
            } else if id == "lock_hud" {
                apply_edit_mode(app, false);
                hide_controls(app);
            } else if id == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                toggle_controls(tray.app_handle(), Some(position));
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}

fn spawn_udp_listener(
    app: tauri::AppHandle,
    desired: Arc<AtomicU16>,
    udp_status: Arc<Mutex<UdpStatus>>,
) {
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
                        set_udp_status(
                            &app,
                            &udp_status,
                            UdpStatus {
                                port: want,
                                listening: true,
                                error: None,
                            },
                        );
                    }
                    Err(e) => {
                        eprintln!("[udp] 绑定 {want} 失败: {e}");
                        set_udp_status(
                            &app,
                            &udp_status,
                            UdpStatus {
                                port: want,
                                listening: false,
                                error: Some(bind_error_message(want, &e)),
                            },
                        );
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
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                    }
                    Err(e) => eprintln!("[udp] recv 错误: {e}"),
                }
            }
        }
    });
}

fn main() {
    let mut cfg = load_config_file();
    let desired_port = Arc::new(AtomicU16::new(cfg.port));
    let udp_status = Arc::new(Mutex::new(UdpStatus {
        port: cfg.port,
        listening: false,
        error: None,
    }));
    let state = AppState {
        config: Mutex::new(cfg.clone()),
        desired_port: desired_port.clone(),
        udp_status: udp_status.clone(),
        shortcut_status: Mutex::new(ShortcutStatus {
            registered: false,
            error: None,
        }),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            save_window_pos,
            save_window_size,
            get_udp_status,
            get_shortcut_status,
            set_edit_mode,
            quit_app
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // 默认布局：
            // - 主 HUD 在屏幕底部中心
            // - 设置全部由状态栏/托盘菜单承接
            place_window(&handle, &mut cfg, "main", 0.0, -200.0);

            for label in OVERLAY_WINDOWS {
                if let Some(w) = app.get_webview_window(label) {
                    let _ = w.set_ignore_cursor_events(true);
                    let _ = w.show();
                }
            }

            build_tray(&handle)?;

            let shortcut_status = match app.global_shortcut().on_shortcut(
                "CmdOrCtrl+Shift+H",
                |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_edit(app);
                    }
                },
            ) {
                Ok(()) => ShortcutStatus {
                    registered: true,
                    error: None,
                },
                Err(e) => {
                    let message = format!("注册快捷键 CmdOrCtrl+Shift+H 失败: {e}");
                    eprintln!("[shortcut] {message}");
                    ShortcutStatus {
                        registered: false,
                        error: Some(message),
                    }
                }
            };
            if let Ok(mut status) = app.state::<AppState>().shortcut_status.lock() {
                *status = shortcut_status.clone();
            }
            let _ = app.emit("shortcut-status", &shortcut_status);

            spawn_udp_listener(handle, desired_port.clone(), udp_status.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用出错");
}
#[tauri::command]
fn set_edit_mode(app: tauri::AppHandle, editing: bool) {
    apply_edit_mode(&app, editing);
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
