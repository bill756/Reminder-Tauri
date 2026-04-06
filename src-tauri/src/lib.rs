use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use std::sync::atomic::{AtomicBool, Ordering};

static INPUT_BLOCKED: AtomicBool = AtomicBool::new(false);

#[link(name = "user32")]
extern "system" {
    fn BlockInput(block: i32) -> i32;
}

#[tauri::command]
fn block_input(block: bool) -> Result<bool, String> {
    unsafe {
        let result = if block {
            INPUT_BLOCKED.store(true, Ordering::SeqCst);
            BlockInput(1)
        } else {
            INPUT_BLOCKED.store(false, Ordering::SeqCst);
            BlockInput(0)
        };

        if result != 0 {
            Ok(true)
        } else {
            Err("Failed to block input - need admin".to_string())
        }
    }
}

#[tauri::command]
fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token_handle = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
                return false;
            }
            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = 0u32;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut size,
            );
            return result.is_ok() && elevation.TokenIsElevated != 0;
        }
    }
}

#[tauri::command]
async fn create_work_window(app: AppHandle, work_minutes: i32, rest_minutes: i32, input_block: bool) -> Result<(), String> {
    let work_window = WebviewWindowBuilder::new(
        &app,
        "work",
        WebviewUrl::App(format!("index.html?mode=work&work={}&rest={}&block={}", work_minutes, rest_minutes, if input_block { 1 } else { 0 }).into()),
    )
    .title("工作时间")
    .inner_size(180.0, 100.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(true)
    .build()
    .map_err(|e: tauri::Error| e.to_string())?;

    if let Ok(monitor) = work_window.current_monitor() {
        if let Some(monitor) = monitor {
            let size = monitor.size();
            let x = (size.width as i32) - 200;
            let y = (size.height as i32) - 120;
            let _ = work_window.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition { x, y },
            ));
        }
    }
    Ok(())
}

#[tauri::command]
async fn move_work_window_center(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("work") {
        if let Ok(monitor) = window.current_monitor() {
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let window_size = window.outer_size().unwrap();
                let x = (size.width as i32 - window_size.width as i32) / 2;
                let y = (size.height as i32 - window_size.height as i32) / 2;
                let _ = window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition { x, y },
                ));
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn create_rest_window(app: AppHandle, work_minutes: i32, rest_minutes: i32, input_block: bool) -> Result<(), String> {
    if input_block {
        // Fullscreen overlay with block mode - one window per monitor
        let monitors = app.available_monitors().map_err(|e| e.to_string())?;

        for (index, monitor) in monitors.iter().enumerate() {
            let label = if index == 0 { "rest".to_string() } else { format!("rest-{}", index) };

            let window = WebviewWindowBuilder::new(
                &app,
                &label,
                WebviewUrl::App(format!("index.html?mode=rest&work={}&rest={}&block={}", work_minutes, rest_minutes, if input_block { 1 } else { 0 }).into()),
            )
            .title("休息时间")
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(true)
            .build()
            .map_err(|e: tauri::Error| e.to_string())?;

            // Set window to fullscreen on the specific monitor
            let monitor_size = monitor.size();
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: monitor.position().x,
                y: monitor.position().y,
            }));
            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: monitor_size.width,
                height: monitor_size.height,
            }));
        }
    } else {
        // Small window mode without block - similar to work window
        let rest_window = WebviewWindowBuilder::new(
            &app,
            "rest",
            WebviewUrl::App(format!("index.html?mode=rest&work={}&rest={}&block={}", work_minutes, rest_minutes, if input_block { 1 } else { 0 }).into()),
        )
        .title("休息时间")
        .inner_size(180.0, 150.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;

        // Position at bottom-right, same as work window
        if let Ok(monitor) = rest_window.current_monitor() {
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let x = (size.width as i32) - 200;
                let y = (size.height as i32) - 170;
                let _ = rest_window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition { x, y },
                ));
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn close_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn close_all_rest_windows(app: AppHandle) -> Result<(), String> {
    let windows: Vec<String> = app.webview_windows()
        .keys()
        .filter(|label| label.starts_with("rest"))
        .cloned()
        .collect();

    for label in windows {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
    }
    Ok(())
}

#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Check and request admin privileges on startup
#[cfg(target_os = "windows")]
fn check_admin_and_request() {
    if !is_admin() {
        use std::env;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        if let Ok(exe_path) = env::current_exe() {
            let exe_path_wide: Vec<u16> = OsStr::new(&exe_path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let operation: Vec<u16> = OsStr::new("runas")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            #[link(name = "shell32")]
            extern "system" {
                fn ShellExecuteW(hwnd: *mut std::ffi::c_void, operation: *const u16, file: *const u16, parameters: *const u16, directory: *const u16, show_cmd: i32) -> *mut std::ffi::c_void;
            }

            unsafe {
                const SW_SHOWNORMAL: i32 = 1;
                let _ = ShellExecuteW(
                    std::ptr::null_mut(),
                    operation.as_ptr(),
                    exe_path_wide.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    SW_SHOWNORMAL,
                );
            }
            std::process::exit(0);
        }
    }
}

pub fn run() {
    #[cfg(target_os = "windows")]
    check_admin_and_request();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            block_input,
            is_admin,
            create_work_window,
            create_rest_window,
            close_window,
            close_all_rest_windows,
            show_main_window,
            hide_main_window,
            move_work_window_center,
        ])
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;
            use tauri::image::Image;

            let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "隐藏主窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Load icon from embedded bytes
            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                .expect("Failed to load icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("久坐提醒")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}