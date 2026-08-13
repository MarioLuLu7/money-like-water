use crate::{usage::ProviderUsage, UsageState};
use std::{sync::Arc, thread};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/tray.png");

fn bind_main_close_to_hide(window: &WebviewWindow) {
    let window = window.clone();
    window.clone().on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
    });
}

fn main_window(app: &tauri::AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok(window);
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .ok_or(tauri::Error::WindowNotFound)?;
    let window = WebviewWindowBuilder::from_config(app, config)?.build()?;
    bind_main_close_to_hide(&window);
    Ok(window)
}

fn show_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    window.show()?;
    if window.is_minimized().unwrap_or(false) {
        window.unminimize()?;
    }
    window.set_focus()?;
    Ok(())
}

pub fn setup(app: &mut tauri::App) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        bind_main_close_to_hide(&window);
    }

    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "刷新用量", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &refresh, &separator, &quit])?;
    let icon = tauri::image::Image::from_bytes(TRAY_ICON_PNG)?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Money Like Water")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let _ = show_main_window(app);
                if let Some(window) = app.get_webview_window("meter") {
                    let _ = window.show();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                if let Some(window) = app.get_webview_window("meter") {
                    let _ = window.hide();
                }
            }
            "refresh" => {
                let app = app.clone();
                let state = app.state::<Arc<UsageState>>().inner().clone();
                thread::spawn(move || {
                    let _ = crate::refresh_usage_and_emit(&app, &state);
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                match main_window(&app) {
                    Ok(window) if window.is_visible().unwrap_or(false) => {
                        let _ = window.hide();
                    }
                    Ok(_) => {
                        let _ = show_main_window(&app);
                    }
                    Err(_) => {}
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_usage_tooltip(app: &tauri::AppHandle, usage: &ProviderUsage) -> Result<(), String> {
    let primary = usage.windows.first();
    let remaining = primary
        .map(|window| format!("剩余 {}%", window.remaining_percent))
        .unwrap_or_else(|| "暂无用量窗口".to_string());
    let account = usage.account_label.as_deref().unwrap_or("AI 用量监控");
    let reset = primary
        .and_then(|window| window.resets_at.as_deref())
        .map(|value| format!(" 重置 {value}"))
        .unwrap_or_default();
    let tooltip = format!(
        "Money Like Water\n{account}\n{}: {remaining}{reset}",
        usage.status.label()
    );

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_tooltip(Some(&tooltip))
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}
