use crate::config::{self, MeterAnchor, MeterSettings};
use tauri::{LogicalSize, Manager};

pub const DEFAULT_METER_WIDTH: u32 = 140;
pub const DEFAULT_METER_HEIGHT: u32 = 48;
const TASKBAR_MARGIN: i32 = 12;
const TASKBAR_TOP_SAFE_MARGIN: i32 = 2;
const TASKBAR_EDGE_VISIBLE_MARGIN: i32 = 6;

#[cfg(target_os = "windows")]
fn scaled_meter_size(scale_factor: f64) -> (i32, i32) {
    let width = (DEFAULT_METER_WIDTH as f64 * scale_factor).round() as i32;
    let height = (DEFAULT_METER_HEIGHT as f64 * scale_factor).round() as i32;
    (width.max(1), height.max(1))
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct TaskbarRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[cfg(target_os = "windows")]
fn get_taskbar_rect() -> Result<TaskbarRect, String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA};

    let mut data = APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };

    let result = unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) };
    if result == 0 {
        return Err("无法读取 Windows 任务栏位置。".to_string());
    }

    Ok(TaskbarRect {
        left: data.rc.left,
        top: data.rc.top,
        right: data.rc.right,
        bottom: data.rc.bottom,
    })
}

#[cfg(target_os = "windows")]
fn clamp(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max.max(min))
}

#[cfg(target_os = "windows")]
fn calculate_meter_position(
    rect: TaskbarRect,
    settings: MeterSettings,
    meter_width: i32,
    meter_height: i32,
) -> (i32, i32) {
    let taskbar_width = rect.right - rect.left;
    let taskbar_height = rect.bottom - rect.top;
    let horizontal_taskbar = taskbar_width >= taskbar_height;

    let (base_x, base_y) = if horizontal_taskbar {
        let x = match settings.anchor {
            MeterAnchor::Left => rect.left + TASKBAR_MARGIN,
            MeterAnchor::Center => rect.left + ((taskbar_width - meter_width) / 2),
            MeterAnchor::Right => rect.right - meter_width - TASKBAR_MARGIN,
        };
        let y = rect.top + TASKBAR_TOP_SAFE_MARGIN;
        (x, y)
    } else {
        let x = rect.left + ((taskbar_width - meter_width) / 2);
        let y = match settings.anchor {
            MeterAnchor::Left => rect.top + TASKBAR_MARGIN,
            MeterAnchor::Center => rect.top + ((taskbar_height - meter_height) / 2),
            MeterAnchor::Right => rect.bottom - meter_height - TASKBAR_MARGIN,
        };
        (x, y)
    };

    let adjusted_x = base_x + settings.offsets.left - settings.offsets.right;
    let adjusted_y = base_y + settings.offsets.top - settings.offsets.bottom;

    let min_x = rect.left + 2;
    let max_x = rect.right - meter_width - 2;
    let (min_y, max_y) = if horizontal_taskbar {
        (
            rect.top - meter_height + TASKBAR_EDGE_VISIBLE_MARGIN,
            rect.bottom - TASKBAR_EDGE_VISIBLE_MARGIN,
        )
    } else {
        (rect.top + 2, rect.bottom - meter_height - 2)
    };

    (
        clamp(adjusted_x, min_x, max_x),
        clamp(adjusted_y, min_y, max_y),
    )
}

#[cfg(target_os = "windows")]
fn calculate_meter_child_position(
    rect: TaskbarRect,
    settings: MeterSettings,
    meter_width: i32,
    meter_height: i32,
) -> (i32, i32) {
    let (screen_x, screen_y) = calculate_meter_position(rect, settings, meter_width, meter_height);
    (screen_x - rect.left, screen_y - rect.top)
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn embed_meter_in_taskbar(window: &tauri::WebviewWindow, rect: TaskbarRect) -> Result<(), String> {
    use std::ptr::null;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetParent, GetWindowLongPtrW, SetParent, SetWindowLongPtrW, SetWindowPos,
        GWL_EXSTYLE, GWL_STYLE, HWND_TOP, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
        SWP_SHOWWINDOW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPSIBLINGS, WS_DLGFRAME,
        WS_EX_APPWINDOW, WS_EX_CLIENTEDGE, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_STATICEDGE,
        WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
    };

    let hwnd = window.hwnd().map_err(|err| err.to_string())?.0 as HWND;
    let taskbar_class = wide_null("Shell_TrayWnd");
    let taskbar_hwnd = unsafe { FindWindowW(taskbar_class.as_ptr(), null()) };

    if taskbar_hwnd.is_null() {
        return Err("无法找到 Windows 任务栏窗口。".to_string());
    }

    let app = window.app_handle();
    let settings = config::load_meter_settings(&app);
    let scale_factor = window.scale_factor().unwrap_or(1.0);
    let (width, height) = scaled_meter_size(scale_factor);
    let (x, y) = calculate_meter_child_position(rect, settings, width, height);

    unsafe {
        let needs_embed = GetParent(hwnd) != taskbar_hwnd;
        let mut flags = SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW;

        if needs_embed {
            SetParent(hwnd, taskbar_hwnd);

            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            let style_without_frame = style
                & !WS_POPUP
                & !WS_CAPTION
                & !WS_THICKFRAME
                & !WS_BORDER
                & !WS_DLGFRAME
                & !WS_SYSMENU;
            let child_style = style_without_frame | WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS;
            SetWindowLongPtrW(hwnd, GWL_STYLE, child_style as isize);

            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let ex_style_without_edges = ex_style
                & !WS_EX_APPWINDOW
                & !WS_EX_CLIENTEDGE
                & !WS_EX_LAYERED
                & !WS_EX_WINDOWEDGE
                & !WS_EX_STATICEDGE;
            let child_ex_style = ex_style_without_edges | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, child_ex_style as isize);

            flags |= SWP_FRAMECHANGED;
        }

        SetWindowPos(hwnd, HWND_TOP, x, y, width, height, flags);
    }

    Ok(())
}

pub fn position_meter_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("meter")
        .ok_or_else(|| "未找到任务栏用量窗口。".to_string())?;

    window
        .set_size(LogicalSize::new(DEFAULT_METER_WIDTH, DEFAULT_METER_HEIGHT))
        .map_err(|err| err.to_string())?;

    #[cfg(target_os = "windows")]
    {
        let rect = get_taskbar_rect()?;
        embed_meter_in_taskbar(&window, rect)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use tauri::PhysicalPosition;

        window
            .set_position(PhysicalPosition::new(24, 24))
            .map_err(|err| err.to_string())?;
    }

    let _ = window.set_always_on_top(true);
    let _ = window.show();
    Ok(())
}

pub fn toggle_meter_window(app: &tauri::AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("meter")
        .ok_or_else(|| "未找到任务栏用量窗口。".to_string())?;

    let is_visible = window.is_visible().map_err(|err| err.to_string())?;
    if is_visible {
        window.hide().map_err(|err| err.to_string())?;
        Ok(false)
    } else {
        position_meter_window(app)?;
        window.show().map_err(|err| err.to_string())?;
        Ok(true)
    }
}
