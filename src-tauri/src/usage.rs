use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub id: String,
    pub label: String,
    pub used_percent: u8,
    pub remaining_percent: u8,
    pub value_label: Option<String>,
    pub resets_at: Option<String>,
    pub limit_label: Option<String>,
    pub bucket_id: Option<String>,
    pub window_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ProviderUsageStatus {
    Ok,
    CodexMissing,
    LoggedOut,
    Unauthorized,
    Unavailable,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider: String,
    pub account_label: Option<String>,
    pub plan_label: Option<String>,
    pub credit_balance: Option<f64>,
    pub status: ProviderUsageStatus,
    pub windows: Vec<UsageWindow>,
    pub meter_items: Vec<MeterDisplayItem>,
    pub updated_at: Option<String>,
    pub message: Option<String>,
    pub diagnostics: Option<UsageDiagnostics>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterDisplayItem {
    pub id: String,
    pub label: String,
    pub remaining_percent: u8,
    pub value_label: Option<String>,
    pub reset_label: Option<String>,
    pub status: ProviderUsageStatus,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageDiagnostics {
    pub codex_path: Option<String>,
    pub codex_home: Option<String>,
    pub selected_window_id: Option<String>,
    pub buckets: Vec<UsageDiagnosticBucket>,
    pub raw_account_kind: Option<String>,
    pub requires_openai_auth: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageDiagnosticBucket {
    pub id: String,
    pub name: Option<String>,
    pub primary_window_id: Option<String>,
    pub secondary_window_id: Option<String>,
}

impl ProviderUsageStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ok => "正常",
            Self::CodexMissing => "连接器不可用",
            Self::LoggedOut => "未登录",
            Self::Unauthorized => "无权限",
            Self::Unavailable => "不可用",
            Self::Error => "错误",
        }
    }
}
