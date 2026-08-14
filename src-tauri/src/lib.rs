mod chatgpt_provider;
mod codex_bridge;
mod config;
mod generic_http_provider;
mod taskbar_window;
mod tray;
mod usage;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{Emitter, Manager};

pub(crate) struct UsageState {
    latest: Mutex<Option<usage::ProviderUsage>>,
    latest_success: Mutex<Option<usage::ProviderUsage>>,
    refreshing: AtomicBool,
}

impl Default for UsageState {
    fn default() -> Self {
        Self {
            latest: Mutex::new(None),
            latest_success: Mutex::new(None),
            refreshing: AtomicBool::new(false),
        }
    }
}

#[tauri::command]
async fn get_usage_snapshot(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<UsageState>>,
) -> Result<usage::ProviderUsage, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || refresh_usage_and_emit(&app, &state))
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn get_usage_snapshot_for_source(
    app: tauri::AppHandle,
    source: String,
) -> Result<usage::ProviderUsage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = config::load_meter_settings(&app);
        snapshot_for_source(&app, &settings, &source)
    })
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn update_tray_tooltip(app: tauri::AppHandle, tooltip: String) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_tooltip(Some(&tooltip))
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

pub(crate) fn refresh_usage_and_emit(
    app: &tauri::AppHandle,
    state: &Arc<UsageState>,
) -> usage::ProviderUsage {
    let usage = refresh_usage_state(app, state);
    let _ = tray::update_usage_tooltip(app, &usage);
    let _ = app.emit("usage-updated", usage.clone());
    usage
}

fn refresh_usage_soon(app: tauri::AppHandle, state: Arc<UsageState>) {
    tauri::async_runtime::spawn(async move {
        let _ = tauri::async_runtime::spawn_blocking(move || refresh_usage_and_emit(&app, &state))
            .await;
    });
}

fn refresh_usage_state(app: &tauri::AppHandle, state: &UsageState) -> usage::ProviderUsage {
    if state.refreshing.swap(true, Ordering::AcqRel) {
        if let Ok(latest) = state.latest.lock() {
            if let Some(snapshot) = latest.clone() {
                return snapshot;
            }
        }
    }

    let settings = config::load_meter_settings(app);
    let mut snapshot = snapshot_for_source(app, &settings, &settings.selected_meter_source);
    snapshot.meter_items = meter_items_for_settings(app, &settings, &snapshot);
    if snapshot.status == usage::ProviderUsageStatus::Ok {
        if let Ok(mut latest_success) = state.latest_success.lock() {
            *latest_success = Some(snapshot.clone());
        }
    } else if let Ok(latest_success) = state.latest_success.lock() {
        if let Some(success) = latest_success.clone() {
            if success.provider == snapshot.provider {
                snapshot.windows = success.windows;
                if snapshot.account_label.is_none() {
                    snapshot.account_label = success.account_label;
                }
                if snapshot.plan_label.is_none() {
                    snapshot.plan_label = success.plan_label;
                }
                if snapshot.credit_balance.is_none() {
                    snapshot.credit_balance = success.credit_balance;
                }
                snapshot.message = Some(match snapshot.message {
                    Some(message) => format!("{message} 正在沿用上次成功获取的用量数据。"),
                    None => "正在沿用上次成功获取的用量数据。".to_string(),
                });
            }
        }
    }

    if let Ok(mut latest) = state.latest.lock() {
        *latest = Some(snapshot.clone());
    }
    state.refreshing.store(false, Ordering::Release);
    snapshot
}

fn snapshot_for_source(
    app: &tauri::AppHandle,
    settings: &config::MeterSettings,
    source_id: &str,
) -> usage::ProviderUsage {
    let Some(source) = settings
        .sources
        .iter()
        .find(|source| source.id == source_id)
    else {
        return unavailable_source_snapshot(source_id);
    };

    match source.kind {
        config::DataSourceKind::Chatgpt => chatgpt_provider::usage_snapshot(app),
        config::DataSourceKind::Kimi | config::DataSourceKind::Http => {
            tauri::async_runtime::block_on(generic_http_provider::usage_snapshot(
                &http_config_for_source(source),
            ))
        }
    }
}

fn meter_items_for_settings(
    app: &tauri::AppHandle,
    settings: &config::MeterSettings,
    active_snapshot: &usage::ProviderUsage,
) -> Vec<usage::MeterDisplayItem> {
    let mut sources = settings.taskbar_source_ids.clone();
    if sources.is_empty() {
        sources.push(settings.selected_meter_source.clone());
    }

    sources
        .into_iter()
        .map(|source_id| {
            if source_id == settings.selected_meter_source {
                active_snapshot.clone()
            } else {
                snapshot_for_source(app, settings, &source_id)
            }
        })
        .map(|snapshot| meter_item_from_usage(&snapshot))
        .collect()
}

fn meter_item_from_usage(usage: &usage::ProviderUsage) -> usage::MeterDisplayItem {
    let primary = usage.windows.first();
    usage::MeterDisplayItem {
        id: usage.provider.clone(),
        label: provider_label(&usage.provider),
        remaining_percent: primary.map(|window| window.remaining_percent).unwrap_or(0),
        value_label: primary.and_then(|window| window.value_label.clone()),
        reset_label: primary.and_then(|window| window.resets_at.clone()),
        status: usage.status.clone(),
    }
}

fn provider_label(provider: &str) -> String {
    provider.to_string()
}

fn unavailable_source_snapshot(source_id: &str) -> usage::ProviderUsage {
    usage::ProviderUsage {
        provider: source_id.to_string(),
        account_label: Some(source_id.to_string()),
        plan_label: None,
        credit_balance: None,
        status: usage::ProviderUsageStatus::Unavailable,
        windows: vec![usage::UsageWindow {
            id: format!("{source_id}-unavailable"),
            label: source_id.to_string(),
            used_percent: 0,
            remaining_percent: 0,
            value_label: None,
            resets_at: None,
            limit_label: Some("data source not found".to_string()),
            bucket_id: Some(source_id.to_string()),
            window_key: Some("unavailable".to_string()),
        }],
        meter_items: Vec::new(),
        updated_at: None,
        message: Some("Data source not found in settings.".to_string()),
        diagnostics: None,
    }
}

fn http_config_for_source(
    source: &config::DataSourceSettings,
) -> generic_http_provider::HttpSourceConfig {
    let value_kind = match source.parser.value_format.as_str() {
        "percent" => generic_http_provider::ValueKind::Percent,
        _ => generic_http_provider::ValueKind::Money,
    };
    let auth = match source.auth_mode.as_str() {
        "raw" => generic_http_provider::AuthMode::Raw,
        _ => generic_http_provider::AuthMode::Bearer,
    };

    generic_http_provider::HttpSourceConfig {
        provider_id: source.id.clone(),
        label: source.label.clone(),
        base_url: source.base_url.clone(),
        endpoint: source.endpoint.clone(),
        endpoint_fallbacks: Vec::new(),
        enabled: true,
        api_key: source.api_key.clone(),
        auth,
        headers: source
            .headers
            .iter()
            .map(|header| (header.name.clone(), header.value.clone()))
            .collect(),
        timeout_seconds: source.timeout_seconds,
        window_id: format!("{}-value", source.id),
        window_key: "value".to_string(),
        limit_label: "HTTP value".to_string(),
        empty_limit_label: "HTTP value".to_string(),
        value_kind,
        low_threshold: source.low_balance_threshold,
        extraction_rules: source
            .parser
            .value_paths
            .iter()
            .map(|rule| generic_http_provider::ExtractionRule {
                path: rule.path.clone(),
                divisor: rule.divisor,
            })
            .collect(),
        currency_paths: source.parser.currency_paths.clone(),
        currency_default: source.parser.currency_default.clone(),
        reset_paths: source.parser.reset_paths.clone(),
        transform_script: source.transform_script.clone(),
        window_rules: source
            .parser
            .windows
            .iter()
            .map(|rule| generic_http_provider::WindowRule {
                id: rule.id.clone(),
                label: rule.label.clone(),
                root_path: rule.root_path.clone(),
                limit_path: rule.limit_path.clone(),
                remaining_path: rule.remaining_path.clone(),
                used_path: rule.used_path.clone(),
                reset_path: rule.reset_path.clone(),
                limit_label: rule.limit_label.clone(),
                window_key: rule.window_key.clone(),
            })
            .collect(),
        array_window_rules: source
            .parser
            .array_windows
            .iter()
            .map(|rule| generic_http_provider::ArrayWindowRule {
                id_prefix: rule.id_prefix.clone(),
                label: rule.label.clone(),
                array_path: rule.array_path.clone(),
                item_root_path: rule.item_root_path.clone(),
                limit_path: rule.limit_path.clone(),
                remaining_path: rule.remaining_path.clone(),
                used_path: rule.used_path.clone(),
                reset_path: rule.reset_path.clone(),
                limit_label: rule.limit_label.clone(),
                window_key_prefix: rule.window_key_prefix.clone(),
            })
            .collect(),
    }
}

fn start_background_refresh(app: tauri::AppHandle, state: Arc<UsageState>) {
    thread::spawn(move || loop {
        let _ = refresh_usage_and_emit(&app, &state);
        let settings = config::load_meter_settings(&app);
        let interval_seconds = settings.query_refresh_seconds.clamp(5.0, 3600.0).round() as u64;
        thread::sleep(Duration::from_secs(interval_seconds));
    });
}

#[tauri::command]
fn position_meter_window(app: tauri::AppHandle) -> Result<(), String> {
    taskbar_window::position_meter_window(&app)
}

#[tauri::command]
fn toggle_meter_window(app: tauri::AppHandle) -> Result<bool, String> {
    taskbar_window::toggle_meter_window(&app)
}

#[tauri::command]
fn get_meter_settings(app: tauri::AppHandle) -> config::MeterSettings {
    config::load_meter_settings(&app)
}

#[tauri::command]
fn save_meter_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<UsageState>>,
    settings: config::MeterSettings,
) -> Result<config::MeterSettings, String> {
    let saved = config::save_meter_settings(&app, settings)?;
    let _ = app.emit("settings-updated", saved.clone());
    if let Err(err) = taskbar_window::position_meter_window(&app) {
        eprintln!("Failed to reposition meter window after saving settings: {err}");
    }
    let state = state.inner().clone();
    refresh_usage_soon(app, state);
    Ok(saved)
}

#[tauri::command]
fn save_meter_layout_settings(
    app: tauri::AppHandle,
    settings: config::MeterSettings,
) -> Result<config::MeterSettings, String> {
    let saved = config::save_meter_settings(&app, settings)?;
    let _ = app.emit("settings-updated", saved.clone());
    if let Err(err) = taskbar_window::position_meter_window(&app) {
        eprintln!("Failed to reposition meter window after saving layout settings: {err}");
    }
    Ok(saved)
}

#[tauri::command]
fn get_chatgpt_access_token() -> Result<String, String> {
    config::load_chatgpt_access_token()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(UsageState::default()))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            tray::setup(app)?;
            taskbar_window::position_meter_window(app.handle())?;
            let state = app.state::<Arc<UsageState>>().inner().clone();
            start_background_refresh(app.handle().clone(), state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_snapshot,
            get_usage_snapshot_for_source,
            update_tray_tooltip,
            position_meter_window,
            toggle_meter_window,
            get_meter_settings,
            save_meter_settings,
            save_meter_layout_settings,
            get_chatgpt_access_token
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
