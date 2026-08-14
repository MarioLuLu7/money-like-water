use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};
use tauri::Manager;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MeterAnchor {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterOffsets {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterSettings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub anchor: MeterAnchor,
    #[serde(default)]
    pub offsets: MeterOffsets,
    #[serde(default)]
    pub selected_usage_window_id: Option<String>,
    #[serde(default)]
    pub selected_meter_source: String,
    #[serde(default = "default_sources")]
    pub sources: Vec<DataSourceSettings>,
    #[serde(default)]
    pub ai_member: AiMemberSettings,
    #[serde(default)]
    pub kimi: KimiSettings,
    #[serde(default)]
    pub deepseek: DeepSeekSettings,
    #[serde(default)]
    pub taskbar_sources: TaskbarSources,
    #[serde(default)]
    pub taskbar_source_ids: Vec<String>,
    #[serde(default = "default_taskbar_scroll_seconds")]
    pub taskbar_scroll_seconds: f64,
    #[serde(default = "default_taskbar_scroll_animation_seconds")]
    pub taskbar_scroll_animation_seconds: f64,
    #[serde(default)]
    pub taskbar_appearance: TaskbarAppearance,
    #[serde(default = "default_query_refresh_seconds")]
    pub query_refresh_seconds: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceSettings {
    pub id: String,
    #[serde(default)]
    pub kind: DataSourceKind,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String,
    #[serde(default)]
    pub headers: Vec<HeaderSetting>,
    #[serde(default)]
    pub parser: HttpParserSettings,
    #[serde(default)]
    pub transform_script: String,
    #[serde(default)]
    pub low_balance_threshold: Option<f64>,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceKind {
    Chatgpt,
    Kimi,
    Http,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpParserSettings {
    #[serde(default)]
    pub value_paths: Vec<ValuePathSetting>,
    #[serde(default)]
    pub currency_paths: Vec<String>,
    #[serde(default)]
    pub currency_default: Option<String>,
    #[serde(default)]
    pub reset_paths: Vec<String>,
    #[serde(default = "default_value_format")]
    pub value_format: String,
    #[serde(default)]
    pub windows: Vec<HttpWindowSettings>,
    #[serde(default)]
    pub array_windows: Vec<HttpArrayWindowSettings>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValuePathSetting {
    pub path: String,
    #[serde(default = "default_path_divisor")]
    pub divisor: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpWindowSettings {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub root_path: String,
    #[serde(default = "default_limit_path")]
    pub limit_path: String,
    #[serde(default = "default_remaining_path")]
    pub remaining_path: String,
    #[serde(default)]
    pub used_path: Option<String>,
    #[serde(default = "default_reset_path")]
    pub reset_path: Option<String>,
    #[serde(default)]
    pub limit_label: Option<String>,
    #[serde(default)]
    pub window_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpArrayWindowSettings {
    pub id_prefix: String,
    pub label: String,
    pub array_path: String,
    #[serde(default)]
    pub item_root_path: String,
    #[serde(default = "default_limit_path")]
    pub limit_path: String,
    #[serde(default = "default_remaining_path")]
    pub remaining_path: String,
    #[serde(default)]
    pub used_path: Option<String>,
    #[serde(default = "default_reset_path")]
    pub reset_path: Option<String>,
    #[serde(default)]
    pub limit_label: Option<String>,
    #[serde(default)]
    pub window_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiMemberSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ai_member_label")]
    pub label: String,
    #[serde(default = "default_ai_member_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_ai_member_balance_endpoint")]
    pub balance_endpoint: String,
    #[serde(default)]
    pub low_balance_threshold: Option<f64>,
    #[serde(default = "default_ai_member_balance_divisor")]
    pub balance_divisor: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KimiSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_kimi_label")]
    pub label: String,
    #[serde(default = "default_kimi_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_kimi_balance_endpoint")]
    pub usage_endpoint: String,
    #[serde(default)]
    pub low_balance_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_deepseek_label")]
    pub label: String,
    #[serde(default = "default_deepseek_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_deepseek_balance_endpoint")]
    pub balance_endpoint: String,
    #[serde(default)]
    pub low_balance_threshold: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskbarSources {
    #[serde(default = "default_true")]
    pub chatgpt: bool,
    #[serde(default)]
    pub ai_member: bool,
    #[serde(default)]
    pub kimi: bool,
    #[serde(default)]
    pub deepseek: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskbarAppearance {
    #[serde(default = "default_taskbar_text_size_px")]
    pub text_size_px: f64,
    #[serde(default = "default_taskbar_reset_text_size_px")]
    pub reset_text_size_px: f64,
    #[serde(default = "default_taskbar_text_color")]
    pub text_color: String,
    #[serde(default = "default_taskbar_progress_color")]
    pub progress_color: String,
}

impl Default for MeterSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            anchor: MeterAnchor::Right,
            offsets: MeterOffsets {
                left: 0,
                right: 120,
                top: 0,
                bottom: 0,
            },
            selected_usage_window_id: None,
           selected_meter_source: "chatgpt".to_string(),
            sources: default_sources(),
            ai_member: AiMemberSettings::default(),
           kimi: KimiSettings::default(),
           deepseek: DeepSeekSettings::default(),
           taskbar_sources: TaskbarSources::default(),
            taskbar_source_ids: Vec::new(),
           taskbar_scroll_seconds: default_taskbar_scroll_seconds(),
            taskbar_scroll_animation_seconds: default_taskbar_scroll_animation_seconds(),
            taskbar_appearance: TaskbarAppearance::default(),
            query_refresh_seconds: default_query_refresh_seconds(),
        }
    }
}

impl Default for MeterAnchor {
    fn default() -> Self {
        Self::Right
    }
}

impl Default for MeterOffsets {
    fn default() -> Self {
        Self {
            left: 0,
            right: 120,
            top: 0,
            bottom: 0,
        }
    }
}

impl Default for DataSourceKind {
    fn default() -> Self {
        Self::Http
    }
}

impl Default for HttpParserSettings {
    fn default() -> Self {
        Self {
            value_paths: Vec::new(),
            currency_paths: Vec::new(),
            currency_default: Some("USD".to_string()),
            reset_paths: Vec::new(),
            value_format: default_value_format(),
            windows: Vec::new(),
            array_windows: Vec::new(),
        }
    }
}

impl Default for AiMemberSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            label: default_ai_member_label(),
            base_url: default_ai_member_base_url(),
            api_key: String::new(),
            balance_endpoint: default_ai_member_balance_endpoint(),
            low_balance_threshold: Some(10.0),
            balance_divisor: default_ai_member_balance_divisor(),
        }
    }
}

impl Default for KimiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            label: default_kimi_label(),
            base_url: default_kimi_base_url(),
            api_key: String::new(),
            usage_endpoint: default_kimi_balance_endpoint(),
            low_balance_threshold: Some(30.0),
        }
    }
}

impl Default for DeepSeekSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            label: default_deepseek_label(),
            base_url: default_deepseek_base_url(),
            api_key: String::new(),
            balance_endpoint: default_deepseek_balance_endpoint(),
            low_balance_threshold: Some(10.0),
        }
    }
}

impl Default for TaskbarSources {
    fn default() -> Self {
        Self {
            chatgpt: true,
            ai_member: false,
            kimi: false,
            deepseek: false,
        }
    }
}

impl Default for TaskbarAppearance {
    fn default() -> Self {
        Self {
            text_size_px: default_taskbar_text_size_px(),
            reset_text_size_px: default_taskbar_reset_text_size_px(),
            text_color: default_taskbar_text_color(),
            progress_color: default_taskbar_progress_color(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_language() -> String {
    "en".to_string()
}

fn default_taskbar_scroll_seconds() -> f64 {
    3.2
}

fn default_taskbar_scroll_animation_seconds() -> f64 {
    0.35
}

fn default_query_refresh_seconds() -> f64 {
    60.0
}

fn default_auth_mode() -> String {
    "bearer".to_string()
}

fn default_timeout_seconds() -> u64 {
    8
}

fn default_path_divisor() -> f64 {
    1.0
}

fn default_value_format() -> String {
    "money".to_string()
}

fn default_limit_path() -> String {
    "limit".to_string()
}

fn default_remaining_path() -> String {
    "remaining".to_string()
}

fn default_reset_path() -> Option<String> {
    Some("resetTime".to_string())
}

fn default_sources() -> Vec<DataSourceSettings> {
    vec![
        DataSourceSettings {
            id: "chatgpt".to_string(),
            kind: DataSourceKind::Http,
            enabled: true,
            label: "ChatGPT".to_string(),
            base_url: default_chatgpt_base_url(),
            endpoint: default_chatgpt_usage_endpoint(),
            api_key: String::new(),
            auth_mode: default_auth_mode(),
            headers: chatgpt_headers(),
            parser: HttpParserSettings::default(),
            transform_script: chatgpt_transform_script(),
            low_balance_threshold: Some(30.0),
            timeout_seconds: default_timeout_seconds(),
        },
        ai_member_source(AiMemberSettings::default()),
        kimi_source(KimiSettings::default()),
        deepseek_source(DeepSeekSettings::default()),
        siliconflow_source(),
        glm_source(),
    ]
}

fn ai_member_source(settings: AiMemberSettings) -> DataSourceSettings {
    let mut value_paths: Vec<ValuePathSetting> = [
        "data.user_usd_available",
        "data.total_usd_available",
        "data.remaining_usd",
        "data.available_usd",
        "user_usd_available",
        "total_usd_available",
        "remaining_usd",
        "available_usd",
        "data.remaining",
        "data.balance",
        "data.available",
        "balance",
        "remaining",
        "available",
        "usd",
    ]
    .into_iter()
    .map(|path| ValuePathSetting {
        path: path.to_string(),
        divisor: 1.0,
    })
    .collect();
    value_paths.extend(
        [
            "data.total_available",
            "data.remaining_quota",
            "total_available",
            "remaining_quota",
        ]
        .into_iter()
        .map(|path| ValuePathSetting {
            path: path.to_string(),
            divisor: settings.balance_divisor,
        }),
    );
    value_paths.push(ValuePathSetting {
        path: "cents".to_string(),
        divisor: 100.0,
    });

    DataSourceSettings {
        id: "ai-member".to_string(),
        kind: DataSourceKind::Http,
        enabled: settings.enabled,
        label: settings.label,
        base_url: settings.base_url,
        endpoint: settings.balance_endpoint,
        api_key: settings.api_key,
        auth_mode: default_auth_mode(),
        headers: vec![
            HeaderSetting {
                name: "accept-language".to_string(),
                value: "zh".to_string(),
            },
            HeaderSetting {
                name: "x-user-ui-request".to_string(),
                value: "1".to_string(),
            },
        ],
        parser: HttpParserSettings {
            value_paths,
            currency_paths: Vec::new(),
            currency_default: Some("USD".to_string()),
            reset_paths: Vec::new(),
            value_format: "money".to_string(),
            windows: Vec::new(),
            array_windows: Vec::new(),
        },
        transform_script: ai_member_transform_script(),
        low_balance_threshold: settings.low_balance_threshold,
        timeout_seconds: default_timeout_seconds(),
    }
}

fn kimi_source(settings: KimiSettings) -> DataSourceSettings {
    DataSourceSettings {
        id: "kimi".to_string(),
        kind: DataSourceKind::Http,
        enabled: settings.enabled,
        label: settings.label,
        base_url: settings.base_url,
        endpoint: settings.usage_endpoint,
        api_key: settings.api_key,
        auth_mode: default_auth_mode(),
        headers: Vec::new(),
        parser: HttpParserSettings {
            value_paths: Vec::new(),
            currency_paths: Vec::new(),
            currency_default: None,
            reset_paths: Vec::new(),
            value_format: "percent".to_string(),
            windows: vec![HttpWindowSettings {
               id: "weekly".to_string(),
                label: "周额度".to_string(),
               root_path: "usage".to_string(),
               limit_path: "limit".to_string(),
               remaining_path: "remaining".to_string(),
               used_path: Some("used".to_string()),
               reset_path: Some("resetTime".to_string()),
                limit_label: Some("Kimi Code 周额度".to_string()),
               window_key: Some("weekly".to_string()),
            }],
            array_windows: Vec::new(),
        },
        transform_script: kimi_transform_script(),
        low_balance_threshold: settings.low_balance_threshold,
        timeout_seconds: default_timeout_seconds(),
    }
}

fn deepseek_source(settings: DeepSeekSettings) -> DataSourceSettings {
    DataSourceSettings {
        id: "deepseek".to_string(),
        kind: DataSourceKind::Http,
        enabled: settings.enabled,
        label: settings.label,
        base_url: settings.base_url,
        endpoint: settings.balance_endpoint,
        api_key: settings.api_key,
        auth_mode: default_auth_mode(),
        headers: Vec::new(),
        parser: HttpParserSettings {
            value_paths: vec![
                ValuePathSetting {
                    path: "balance_infos[currency=CNY].total_balance".to_string(),
                    divisor: 1.0,
                },
                ValuePathSetting {
                    path: "balance_infos[0].total_balance".to_string(),
                    divisor: 1.0,
                },
            ],
            currency_paths: vec![
                "balance_infos[currency=CNY].currency".to_string(),
                "balance_infos[0].currency".to_string(),
            ],
            currency_default: Some("CNY".to_string()),
            reset_paths: Vec::new(),
            value_format: "money".to_string(),
            windows: Vec::new(),
            array_windows: Vec::new(),
        },
        transform_script: deepseek_transform_script(),
        low_balance_threshold: settings.low_balance_threshold,
        timeout_seconds: default_timeout_seconds(),
    }
}

fn siliconflow_source() -> DataSourceSettings {
    DataSourceSettings {
        id: "siliconflow".to_string(),
        kind: DataSourceKind::Http,
        enabled: true,
        label: "SiliconFlow".to_string(),
        base_url: default_siliconflow_base_url(),
        endpoint: default_siliconflow_user_info_endpoint(),
        api_key: String::new(),
        auth_mode: default_auth_mode(),
        headers: Vec::new(),
        parser: HttpParserSettings {
            value_paths: vec![
                ValuePathSetting {
                    path: "data.balance".to_string(),
                    divisor: 1.0,
                },
                ValuePathSetting {
                    path: "balance".to_string(),
                    divisor: 1.0,
                },
            ],
            currency_paths: vec!["data.currency".to_string(), "currency".to_string()],
            currency_default: Some("CNY".to_string()),
            reset_paths: Vec::new(),
            value_format: "money".to_string(),
            windows: Vec::new(),
            array_windows: Vec::new(),
        },
        transform_script: siliconflow_transform_script(),
        low_balance_threshold: Some(10.0),
        timeout_seconds: default_timeout_seconds(),
    }
}

fn glm_source() -> DataSourceSettings {
    DataSourceSettings {
        id: "glm".to_string(),
        kind: DataSourceKind::Http,
        enabled: true,
        label: "GLM".to_string(),
        base_url: default_glm_quota_base_url(),
        endpoint: default_glm_quota_endpoint(),
        api_key: String::new(),
        auth_mode: "raw".to_string(),
        headers: Vec::new(),
        parser: HttpParserSettings {
            value_paths: Vec::new(),
            currency_paths: Vec::new(),
            currency_default: None,
            reset_paths: Vec::new(),
            value_format: "percent".to_string(),
            windows: Vec::new(),
            array_windows: Vec::new(),
        },
        transform_script: glm_transform_script(),
        low_balance_threshold: Some(10.0),
        timeout_seconds: default_timeout_seconds(),
    }
}

fn default_chatgpt_base_url() -> String {
    "https://chatgpt.com/backend-api".to_string()
}

fn default_chatgpt_usage_endpoint() -> String {
    "/wham/usage".to_string()
}

fn chatgpt_headers() -> Vec<HeaderSetting> {
    vec![
        HeaderSetting {
            name: "user-agent".to_string(),
            value: "codex-cli".to_string(),
        },
    ]
}

fn default_taskbar_text_size_px() -> f64 {
    9.0
}
fn default_taskbar_reset_text_size_px() -> f64 {
    8.0
}

fn default_taskbar_text_color() -> String {
    "#f6f8fb".to_string()
}

fn default_taskbar_progress_color() -> String {
    "#45d483".to_string()
}

fn default_ai_member_label() -> String {
    "AI-MEMBER".to_string()
}

fn default_ai_member_base_url() -> String {
    "https://proxy.ai-member.icu".to_string()
}

fn default_ai_member_balance_endpoint() -> String {
    "/api/v1/auth/me?timezone=Asia%2FShanghai".to_string()
}

fn default_ai_member_balance_divisor() -> f64 {
    500_000.0
}

fn default_kimi_label() -> String {
    "Kimi Code".to_string()
}

fn default_kimi_base_url() -> String {
    "https://api.kimi.com/coding/v1".to_string()
}

fn default_kimi_balance_endpoint() -> String {
    "/usages".to_string()
}

fn default_deepseek_label() -> String {
    "DeepSeek".to_string()
}

fn default_deepseek_base_url() -> String {
    "https://api.deepseek.com".to_string()
}

fn default_deepseek_balance_endpoint() -> String {
    "/user/balance".to_string()
}

fn default_siliconflow_base_url() -> String {
    "https://api.siliconflow.cn/v1".to_string()
}

fn default_siliconflow_user_info_endpoint() -> String {
    "/user/info".to_string()
}

fn default_glm_quota_base_url() -> String {
    "https://bigmodel.cn".to_string()
}

fn default_glm_quota_endpoint() -> String {
    "/api/monitor/usage/quota/limit".to_string()
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|err| err.to_string())?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load_meter_settings(app: &tauri::AppHandle) -> MeterSettings {
    let Ok(path) = config_path(app) else {
        return MeterSettings::default();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return MeterSettings::default();
    };

    let mut settings: MeterSettings = serde_json::from_str(&content).unwrap_or_default();
    migrate_settings(&mut settings);
    settings
}

pub fn save_meter_settings(
    app: &tauri::AppHandle,
    mut settings: MeterSettings,
) -> Result<MeterSettings, String> {
    migrate_settings(&mut settings);
    let path = config_path(app)?;
    let content = serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| err.to_string())?;
    Ok(settings)
}

pub fn load_chatgpt_access_token() -> Result<String, String> {
    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Unable to resolve the user home directory.".to_string())?;
    let codex_dir = home.join(".codex");
    let candidates = [codex_dir.join("auth.json"), codex_dir.join("auth - 副本.json")];
    let mut errors = Vec::new();

    for path in candidates {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                errors.push(format!("{}: {err}", path.display()));
                continue;
            }
        };
        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(value) => value,
            Err(err) => {
                errors.push(format!("{}: invalid JSON: {err}", path.display()));
                continue;
            }
        };

        if let Some(token) = value
            .get("tokens")
            .and_then(|tokens| tokens.get("access_token"))
            .and_then(serde_json::Value::as_str)
            .filter(|token| !token.trim().is_empty())
        {
            return Ok(token.to_string());
        }

        errors.push(format!("{}: tokens.access_token is empty", path.display()));
    }

    Err(format!(
        "Unable to find ChatGPT access token. {}",
        errors.join("; ")
    ))
}

fn migrate_settings(settings: &mut MeterSettings) {
    if settings.language != "zh" {
        settings.language = default_language();
    }

    let old_ai_member_endpoints = [
        "/api/usage/token/",
        "/api/usage/token",
        "/v1/balance",
        "/api/v1/balance",
    ];

    if old_ai_member_endpoints
        .iter()
        .any(|endpoint| settings.ai_member.balance_endpoint == *endpoint)
    {
        settings.ai_member.balance_endpoint = default_ai_member_balance_endpoint();
    }

    if settings.sources.is_empty() {
        settings.sources = vec![
            DataSourceSettings {
                id: "chatgpt".to_string(),
                kind: DataSourceKind::Http,
                enabled: true,
                label: "ChatGPT".to_string(),
                base_url: default_chatgpt_base_url(),
                endpoint: default_chatgpt_usage_endpoint(),
                api_key: String::new(),
                auth_mode: default_auth_mode(),
                headers: chatgpt_headers(),
                parser: HttpParserSettings::default(),
                transform_script: chatgpt_transform_script(),
            low_balance_threshold: Some(30.0),
            timeout_seconds: default_timeout_seconds(),
        },
        ai_member_source(settings.ai_member.clone()),
        kimi_source(settings.kimi.clone()),
        deepseek_source(settings.deepseek.clone()),
        siliconflow_source(),
        glm_source(),
        ];
    }

    if !settings.sources.iter().any(|source| source.id == "siliconflow") {
        settings.sources.push(siliconflow_source());
    }

    if !settings.sources.iter().any(|source| source.id == "glm") {
        settings.sources.push(glm_source());
    }

    for source in &mut settings.sources {
        if source.id == "chatgpt" && source.kind == DataSourceKind::Chatgpt {
            source.kind = DataSourceKind::Http;
            source.base_url = default_chatgpt_base_url();
            source.endpoint = default_chatgpt_usage_endpoint();
            source.headers = chatgpt_headers();
            source.transform_script = chatgpt_transform_script();
        }

        if source.kind == DataSourceKind::Http
            && source.id == "chatgpt"
            && source.base_url.trim_end_matches('/') == default_chatgpt_base_url()
            && source.endpoint == "/accounts/check/v4-2024-01-01"
        {
            source.endpoint = default_chatgpt_usage_endpoint();
            source.headers = chatgpt_headers();
            source.transform_script = chatgpt_transform_script();
        }

        if source.kind == DataSourceKind::Kimi {
            let migrated = kimi_source(KimiSettings {
                enabled: source.enabled,
                label: source.label.clone(),
                base_url: source.base_url.clone(),
                api_key: source.api_key.clone(),
                usage_endpoint: source.endpoint.clone(),
                low_balance_threshold: source.low_balance_threshold,
            });
            source.kind = DataSourceKind::Http;
            source.parser = migrated.parser;
            source.transform_script = migrated.transform_script;
        }

        if is_glm_preset_source(source) {
            source.auth_mode = "raw".to_string();
        }

        source.enabled = true;

        let should_use_default_transform = source.transform_script.trim().is_empty()
            || (is_kimi_preset_source(source)
                && is_legacy_kimi_transform_script(&source.transform_script));

        if source.kind == DataSourceKind::Http && should_use_default_transform {
            if let Some(transform_script) = default_transform_script_for_source(source) {
                source.transform_script = transform_script;
            }
        }
    }

    settings.sources.retain(|source| source.id != "chatgpt-local");

    if settings.taskbar_source_ids.is_empty() {
        if settings.taskbar_sources.chatgpt {
            settings.taskbar_source_ids.push("chatgpt".to_string());
        }
        if settings.taskbar_sources.ai_member {
            settings.taskbar_source_ids.push("ai-member".to_string());
        }
        if settings.taskbar_sources.kimi {
            settings.taskbar_source_ids.push("kimi".to_string());
        }
        if settings.taskbar_sources.deepseek {
            settings.taskbar_source_ids.push("deepseek".to_string());
        }
    }

    if !settings
        .sources
        .iter()
        .any(|source| source.id == settings.selected_meter_source)
    {
        settings.selected_meter_source = settings
            .sources
            .first()
            .map(|source| source.id.clone())
            .unwrap_or_else(|| "chatgpt".to_string());
    }
}

fn default_transform_script_for_source(source: &DataSourceSettings) -> Option<String> {
    let id = source.id.to_ascii_lowercase();
    let label = source.label.to_ascii_lowercase();
    let base_url = source.base_url.to_ascii_lowercase();

    if id == "ai-member"
        || id.starts_with("ai-member-")
        || label.starts_with("ai-member")
        || base_url == default_ai_member_base_url()
    {
        return Some(ai_member_transform_script());
    }

    if id == "chatgpt"
        || id.starts_with("chatgpt-official")
        || (label.starts_with("chatgpt") && base_url == default_chatgpt_base_url())
    {
        return Some(chatgpt_transform_script());
    }

    if id == "deepseek"
        || id.starts_with("deepseek-")
        || label.starts_with("deepseek")
        || base_url == default_deepseek_base_url()
    {
        return Some(deepseek_transform_script());
    }

    if id == "siliconflow"
        || id.starts_with("siliconflow-")
        || label.starts_with("siliconflow")
        || label.starts_with("硅基流动")
        || base_url == default_siliconflow_base_url()
    {
        return Some(siliconflow_transform_script());
    }

    if is_glm_preset_source(source) {
        return Some(glm_transform_script());
    }

    if is_kimi_preset_source(source) {
        return Some(kimi_transform_script());
    }

    if id == "custom-source"
        || id.starts_with("custom-source-")
        || label.starts_with("custom source")
    {
        return Some(custom_transform_script());
    }

    None
}

fn chatgpt_transform_script() -> String {
    r#"function numberLike(value) {
  if (typeof value === "number") return value;
  if (typeof value === "string" && value.trim()) return Number(value);
  return NaN;
}

function get(path, root = json) {
  return path.split(".").reduce((value, key) => value == null ? undefined : value[key], root);
}

function timestamp(value) {
  if (value == null || value === "") return null;
  if (typeof value === "string" && Number.isNaN(Number(value))) return value;
  const raw = Number(value);
  if (!Number.isFinite(raw)) return null;
  const millis = raw > 10000000000 ? raw : raw * 1000;
  return new Date(millis).toISOString();
}

function durationLabel(window) {
  const minutes = numberLike(window.windowDurationMins)
    || numberLike(window.window_duration_mins)
    || numberLike(window.window_duration_ms) / 60000
    || numberLike(window.window_duration_seconds) / 60
    || numberLike(window.limit_window_seconds) / 60
    || numberLike(window.period_seconds) / 60;
  return Number.isFinite(minutes) && minutes > 0 ? Math.round(minutes) + " 分钟" : "用量窗口";
}

function windowPercent(window) {
  const direct = numberLike(window.usedPercent ?? window.used_percent);
  if (Number.isFinite(direct)) return Math.max(0, Math.min(100, Math.round(direct)));

  const limit = numberLike(window.limit);
  const remaining = numberLike(window.remaining);
  const used = Number.isFinite(numberLike(window.used)) ? numberLike(window.used) : limit - remaining;
  if (!Number.isFinite(limit) || limit <= 0 || !Number.isFinite(remaining)) return null;
  return Math.max(0, Math.min(100, Math.round((Math.max(0, used) / limit) * 100)));
}

function normalizeBucket(bucket, fallbackId) {
  const limitId = bucket.limitId || bucket.limit_id || bucket.id || fallbackId || "codex";
  const limitName = bucket.limitName || bucket.limit_name || bucket.name || "Codex";
  const items = [];

  for (const [key, label] of [["primary", "主用量窗口"], ["secondary", "次用量窗口"]]) {
    const window = bucket[key];
    if (!window) continue;
    const usedPercent = windowPercent(window);
    if (usedPercent == null) continue;
    items.push({
      id: "codex-" + limitId + "-" + key,
      label,
      usedPercent,
      remainingPercent: 100 - usedPercent,
      resetsAt: timestamp(window.resetsAt ?? window.resets_at ?? window.resetTime ?? window.reset_time ?? window.expiresAt ?? window.expires_at),
      limitLabel: limitName + " / " + durationLabel(window),
      bucketId: limitId,
      windowKey: key
    });
  }

  if (!items.length) {
    const usedPercent = windowPercent(bucket);
    if (usedPercent != null) {
      items.push({
        id: "codex-" + limitId + "-primary",
        label: "主用量窗口",
        usedPercent,
        remainingPercent: 100 - usedPercent,
        resetsAt: timestamp(bucket.resetsAt ?? bucket.resets_at ?? bucket.resetTime ?? bucket.reset_time ?? bucket.expiresAt ?? bucket.expires_at),
        limitLabel: limitName + " / " + durationLabel(bucket),
        bucketId: limitId,
        windowKey: "primary"
      });
    }
  }

  return items;
}

function normalizeWhamUsage() {
  const rateLimit = json.rate_limit;
  if (!rateLimit || typeof rateLimit !== "object") return [];

  const bucket = {
    limitId: "codex",
    limitName: "Codex",
    primary: rateLimit.primary_window,
    secondary: rateLimit.secondary_window
  };

  return normalizeBucket(bucket, "codex").map((window) => ({
    ...window,
    resetsAt: window.resetsAt || timestamp(
      (window.windowKey === "primary" ? rateLimit.primary_window : rateLimit.secondary_window)?.reset_at
    )
  }));
}

function collectBuckets() {
  const whamWindows = normalizeWhamUsage();
  if (whamWindows.length) return whamWindows;

  const directMap = json.rateLimitsByLimitId || json.rate_limits_by_limit_id;
  if (directMap && typeof directMap === "object" && !Array.isArray(directMap)) {
    return Object.entries(directMap).flatMap(([id, bucket]) => normalizeBucket(bucket, id));
  }

  for (const key of ["rate_limits", "rateLimits", "limits"]) {
    const value = json[key];
    if (Array.isArray(value)) {
      return value.flatMap((bucket, index) => normalizeBucket(bucket, bucket.limit_id || bucket.limitId || bucket.id || "limit-" + index));
    }
    if (value && typeof value === "object") {
      return Object.entries(value).flatMap(([id, bucket]) => normalizeBucket(bucket, id));
    }
  }

  return Object.entries(json)
    .filter(([, value]) => value && typeof value === "object")
    .flatMap(([id, bucket]) => normalizeBucket(bucket, id));
}

const windows = collectBuckets();
if (!windows.length) throw new Error("No ChatGPT Codex usage windows found");

const account = json.account || json.user || {};
const plan = json.account_plan || json.subscription || {};

return {
  accountLabel: account.email || json.email || "ChatGPT",
  planLabel: plan.plan_type || plan.name || account.planType || account.plan_type || json.plan_type || null,
  message: "ChatGPT 官方 HTTP 用量已更新",
  windows
};"#
    .to_string()
}

fn is_kimi_preset_source(source: &DataSourceSettings) -> bool {
    let id = source.id.to_ascii_lowercase();
    let label = source.label.to_ascii_lowercase();
    let base_url = source.base_url.to_ascii_lowercase();
    let endpoint = source.endpoint.to_ascii_lowercase();

    id == "kimi"
        || id.starts_with("kimi-")
        || label.starts_with("kimi code")
        || base_url == default_kimi_base_url()
        || endpoint == default_kimi_balance_endpoint()
}

fn is_glm_preset_source(source: &DataSourceSettings) -> bool {
    let id = source.id.to_ascii_lowercase();
    let label = source.label.to_ascii_lowercase();
    let base_url = source.base_url.to_ascii_lowercase();

    id == "glm"
        || id.starts_with("glm-")
        || label.starts_with("glm")
        || label.starts_with("智谱")
        || base_url == default_glm_quota_base_url()
}

fn is_legacy_kimi_transform_script(script: &str) -> bool {
    script.contains("json.limits")
        && script.contains("No Kimi usage windows found")
        && script.contains("toWindow")
}

fn ai_member_transform_script() -> String {
    r#"const candidates = [
  "data.balance",
  "data.user_usd_available",
  "data.total_usd_available",
  "balance",
  "usd",
  "cents"
];

function get(path) {
  return path.split(".").reduce((value, key) => value == null ? undefined : value[key], json);
}

let balance = null;
for (const path of candidates) {
  const value = get(path);
  if (value !== undefined && value !== null && value !== "") {
    balance = Number(value);
    if (path === "cents") balance = balance / 100;
    break;
  }
}

if (!Number.isFinite(balance)) throw new Error("No balance field found");
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  creditBalance: balance,
  message: `${source.label} current value: $${balance.toFixed(balance % 1 === 0 ? 0 : 2)}`,
  windows: [{
    id: `${source.id}-balance`,
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel: `$${balance.toFixed(balance % 1 === 0 ? 0 : 2)}`,
    limitLabel: "HTTP transform",
    bucketId: source.id,
    windowKey: "balance"
  }]
};"#
    .to_string()
}

fn deepseek_transform_script() -> String {
    r#"const infos = Array.isArray(json.balance_infos) ? json.balance_infos : [];
const selected = infos.find((item) => item.currency === "CNY") || infos[0];
if (!selected) throw new Error("No balance_infos item found");

const balance = Number(selected.total_balance);
if (!Number.isFinite(balance)) throw new Error("No total_balance field found");

const currency = selected.currency || "CNY";
const prefix = currency === "USD" ? "$" : `${currency} `;
const valueLabel = `${prefix}${balance.toFixed(balance % 1 === 0 ? 0 : 2)}`;
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  creditBalance: balance,
  message: `${source.label} current value: ${valueLabel}`,
  windows: [{
    id: `${source.id}-balance`,
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel,
    limitLabel: "HTTP transform",
    bucketId: source.id,
    windowKey: "balance"
  }]
};"#
    .to_string()
}

fn siliconflow_transform_script() -> String {
    r#"const balance = Number(
  json.data?.balance ??
  json.data?.totalBalance ??
  json.data?.total_balance ??
  json.balance ??
  json.totalBalance ??
  json.total_balance
);

if (!Number.isFinite(balance)) throw new Error("No SiliconFlow balance field found");

const currency = json.data?.currency || json.currency || "CNY";
const prefix = currency === "USD" ? "$" : currency === "CNY" ? "CNY " : currency + " ";
const valueLabel = prefix + balance.toFixed(balance % 1 === 0 ? 0 : 2);
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  accountLabel: json.data?.name || json.data?.email || source.label,
  planLabel: json.data?.status || source.baseUrl,
  creditBalance: balance,
  message: source.label + " current balance: " + valueLabel,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel,
    limitLabel: "SiliconFlow user info",
    bucketId: source.id,
    windowKey: "balance"
  }]
};"#
    .to_string()
}

fn glm_transform_script() -> String {
    r#"function numberLike(value) {
  if (typeof value === "number") return value;
  if (typeof value === "string" && value.trim()) return Number(value);
  return NaN;
}

function timestamp(value) {
  if (value == null || value === "") return null;
  const raw = Number(value);
  if (!Number.isFinite(raw)) return typeof value === "string" ? value : null;
  return new Date(raw > 10000000000 ? raw : raw * 1000).toISOString();
}

const limits = Array.isArray(json.data?.limits) ? json.data.limits : [];
if (!limits.length) throw new Error("No GLM quota limits found");

const labels = {
  TOKENS_LIMIT: "Token quota",
  TIME_LIMIT: "Monthly quota"
};

const windows = limits.map((item, index) => {
  const limit = numberLike(item.usage ?? item.limit ?? item.total);
  const current = numberLike(item.currentValue ?? item.current_value ?? item.used);
  const directPercent = numberLike(item.percentage ?? item.usedPercent ?? item.used_percent);
  const usedPercent = Number.isFinite(directPercent)
    ? Math.max(0, Math.min(100, Math.round(directPercent)))
    : Number.isFinite(limit) && limit > 0 && Number.isFinite(current)
      ? Math.max(0, Math.min(100, Math.round((current / limit) * 100)))
      : 0;
  const type = String(item.type || "quota-" + index);
  return {
    id: source.id + "-" + type.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
    label: labels[type] || type,
    usedPercent,
    remainingPercent: 100 - usedPercent,
    valueLabel: String(100 - usedPercent) + "%",
    resetsAt: timestamp(item.nextResetTime ?? item.next_reset_time ?? item.resetTime ?? item.reset_time),
    limitLabel: "GLM quota",
    bucketId: source.id,
    windowKey: type.toLowerCase()
  };
});

return {
  accountLabel: source.label,
  planLabel: source.baseUrl,
  message: source.label + " quota updated",
  windows
};"#
    .to_string()
}

fn kimi_transform_script() -> String {
    r#"const weekly = json.usage;
if (!weekly) throw new Error("No weekly usage found");

const limit = Number(weekly.limit);
const remaining = Number(weekly.remaining);
if (!Number.isFinite(limit) || !Number.isFinite(remaining)) {
  throw new Error("Weekly usage is missing limit or remaining");
}

const used = Number.isFinite(Number(weekly.used)) ? Number(weekly.used) : Math.max(0, limit - remaining);
const remainingPercent = limit > 0 ? Math.max(0, Math.min(100, Math.round(remaining / limit * 100))) : 0;
const usedPercent = limit > 0 ? Math.max(0, Math.min(100, Math.round(used / limit * 100))) : 100 - remainingPercent;

return {
  message: `${source.label} 周额度 ${remainingPercent}%`,
  windows: [{
   id: `${source.id}-weekly`,
    label: "周额度",
   usedPercent,
   remainingPercent,
   valueLabel: `${remainingPercent}%`,
   resetsAt: weekly.resetTime || null,
    limitLabel: "Kimi Code 周额度",
   bucketId: source.id,
    windowKey: "weekly"
  }]
};"#
    .to_string()
}

fn custom_transform_script() -> String {
    r#"const balance = Number(json.data?.balance ?? json.balance);
if (!Number.isFinite(balance)) throw new Error("No balance field found");

const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  creditBalance: balance,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel: String(balance),
    bucketId: source.id,
    windowKey: "balance"
  }]
};"#
    .to_string()
}





