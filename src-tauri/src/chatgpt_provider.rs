use crate::{
    codex_bridge::{self, CodexError, CodexSnapshot},
    config,
    usage::{ProviderUsage, ProviderUsageStatus, UsageWindow},
};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

pub fn usage_snapshot(app: &tauri::AppHandle) -> ProviderUsage {
    let settings = config::load_meter_settings(app);

    match codex_bridge::read_snapshot() {
        Ok(snapshot) => map_snapshot(snapshot, &settings),
        Err(err) => error_snapshot(ProviderUsageStatus::CodexMissing, err),
    }
}

fn map_snapshot(
    snapshot: CodexSnapshot,
    settings: &config::MeterSettings,
) -> ProviderUsage {
    let account = snapshot
        .account
        .get("account")
        .cloned()
        .unwrap_or(Value::Null);
    let requires_auth = snapshot
        .account
        .get("requiresOpenaiAuth")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if account.is_null() {
        return ProviderUsage {
            provider: "chatgpt".to_string(),
            account_label: Some("ChatGPT 账号".to_string()),
            plan_label: None,
            credit_balance: None,
            status: ProviderUsageStatus::LoggedOut,
            windows: empty_windows(),
            meter_items: Vec::new(),
            updated_at: Some(Utc::now().to_rfc3339()),
            message: Some("Codex 已安装，但 app-server 当前没有登录 ChatGPT 账号。请先运行 `codex login`，然后刷新。".to_string()),
            diagnostics: Some(diagnostics(
                &snapshot,
                settings.selected_usage_window_id.clone(),
            )),
        };
    }

    let account_label = account_label(&account);
    let plan_label = account
        .get("planType")
        .and_then(Value::as_str)
        .map(plan_label);

    let selected_rate_snapshot = snapshot.rate_limits.as_ref().and_then(|value| {
        select_rate_limit_snapshot(value, settings.selected_usage_window_id.as_deref())
    });
    let mut windows = snapshot
        .rate_limits
        .as_ref()
        .map(all_rate_limit_windows)
        .filter(|windows| !windows.is_empty())
        .unwrap_or_else(empty_windows);
    sort_selected_window(&mut windows, settings.selected_usage_window_id.as_deref());
    let credit_balance = selected_rate_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("credits"))
        .and_then(|credits| credits.get("balance"))
        .and_then(Value::as_str)
        .and_then(|balance| balance.parse::<f64>().ok());

    let mut message_parts = Vec::new();
    if let Some(limit_name) = selected_rate_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.get("limitName"))
        .and_then(Value::as_str)
    {
        message_parts.push(format!("正在使用当前连接器用量池：{limit_name}。"));
    }
    if requires_auth {
        message_parts.push(
            "当前数据源账号已识别，API 认证状态不可用；当前优先显示连接器返回的用量窗口。"
                .to_string(),
        );
    }
    if let Some(usage) = snapshot
        .usage
        .as_ref()
        .and_then(|value| value.get("summary"))
    {
        if let Some(tokens) = usage.get("lifetimeTokens").and_then(number_like_to_string) {
            message_parts.push(format!("累计 token：{tokens}。"));
        }
    }
    message_parts.extend(snapshot.warnings.iter().map(|warning| {
        if warning.len() > 240 {
            format!("{}...", &warning[..240])
        } else {
            warning.clone()
        }
    }));

    ProviderUsage {
        provider: "chatgpt".to_string(),
        account_label,
        plan_label,
        credit_balance,
        status: if !windows.is_empty() && windows[0].id != "agentic-primary" {
            ProviderUsageStatus::Ok
        } else {
            ProviderUsageStatus::Unavailable
        },
        windows,
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: if message_parts.is_empty() {
            None
        } else {
            Some(message_parts.join(" "))
        },
        diagnostics: Some(diagnostics(&snapshot, settings.selected_usage_window_id.clone())),
    }
}

fn error_snapshot(status: ProviderUsageStatus, err: CodexError) -> ProviderUsage {
    ProviderUsage {
        provider: "chatgpt".to_string(),
        account_label: Some("当前 AI 数据源".to_string()),
        plan_label: None,
        credit_balance: None,
        status,
        windows: empty_windows(),
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: Some(err.message),
        diagnostics: None,
    }
}

fn account_label(account: &Value) -> Option<String> {
    match account.get("type").and_then(Value::as_str) {
        Some("chatgpt") => account
            .get("email")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| Some("ChatGPT 账号".to_string())),
        Some("apiKey") => Some("OpenAI API Key".to_string()),
        Some("amazonBedrock") => Some("Amazon Bedrock".to_string()),
        _ => Some("AI 数据源账号".to_string()),
    }
}

fn plan_label(value: &str) -> String {
    match value {
        "free" => "Free",
        "go" => "Go",
        "plus" => "Plus",
        "pro" => "Pro",
        "prolite" => "Pro Lite",
        "team" => "Team",
        "business" => "Business",
        "enterprise" => "Enterprise",
        "edu" => "Edu",
        "unknown" => "未知",
        other => other,
    }
    .to_string()
}

fn select_rate_limit_snapshot(value: &Value, selected_window_id: Option<&str>) -> Option<Value> {
    if let Some(selected_window_id) = selected_window_id {
        if let Some(bucket_id) = selected_window_id
            .strip_prefix("codex-")
            .and_then(|rest| rest.rsplit_once('-').map(|(bucket, _)| bucket))
        {
            if let Some(snapshot) = value
                .get("rateLimitsByLimitId")
                .and_then(|buckets| buckets.get(bucket_id))
                .cloned()
            {
                return Some(snapshot);
            }
        }
    }

    value
        .get("rateLimitsByLimitId")
        .and_then(|buckets| buckets.get("codex"))
        .cloned()
        .or_else(|| {
            value
                .get("rateLimitsByLimitId")
                .and_then(Value::as_object)
                .and_then(|buckets| buckets.values().next().cloned())
        })
        .or_else(|| value.get("rateLimits").cloned())
}

fn rate_limit_windows(snapshot: &Value) -> Vec<UsageWindow> {
    let bucket_id = snapshot
        .get("limitId")
        .and_then(Value::as_str)
        .unwrap_or("default");
    let limit_name = snapshot
        .get("limitName")
        .and_then(Value::as_str)
        .unwrap_or("Codex");

    let mut windows = Vec::new();
    if let Some(primary) = snapshot.get("primary") {
        if let Some(window) = map_window(bucket_id, "primary", "主用量窗口", limit_name, primary)
        {
            windows.push(window);
        }
    }
    if let Some(secondary) = snapshot.get("secondary") {
        if let Some(window) =
            map_window(bucket_id, "secondary", "次用量窗口", limit_name, secondary)
        {
            windows.push(window);
        }
    }
    windows
}

fn all_rate_limit_windows(value: &Value) -> Vec<UsageWindow> {
    if let Some(buckets) = value.get("rateLimitsByLimitId").and_then(Value::as_object) {
        return buckets
            .values()
            .flat_map(rate_limit_windows)
            .collect::<Vec<_>>();
    }

    value
        .get("rateLimits")
        .map(rate_limit_windows)
        .unwrap_or_default()
}

fn map_window(
    bucket_id: &str,
    window_key: &str,
    label: &str,
    limit_name: &str,
    value: &Value,
) -> Option<UsageWindow> {
    let used_percent = value.get("usedPercent").and_then(Value::as_f64)?;
    let used_percent = used_percent.clamp(0.0, 100.0).round() as u8;
    let remaining_percent = 100u8.saturating_sub(used_percent);
    let duration = value
        .get("windowDurationMins")
        .and_then(Value::as_i64)
        .map(|mins| format!("{mins} 分钟"))
        .unwrap_or_else(|| "用量窗口".to_string());
    let resets_at = value.get("resetsAt").and_then(timestamp_to_rfc3339);

    Some(UsageWindow {
        id: format!("codex-{bucket_id}-{window_key}"),
        label: label.to_string(),
        used_percent,
        remaining_percent,
        value_label: None,
        resets_at,
        limit_label: Some(format!("{limit_name} / {duration}")),
        bucket_id: Some(bucket_id.to_string()),
        window_key: Some(window_key.to_string()),
    })
}

fn empty_windows() -> Vec<UsageWindow> {
    vec![UsageWindow {
        id: "agentic-primary".to_string(),
        label: "AI 用量".to_string(),
        used_percent: 0,
        remaining_percent: 0,
        value_label: None,
        resets_at: None,
        limit_label: Some("当前数据源".to_string()),
        bucket_id: None,
        window_key: None,
    }]
}

fn sort_selected_window(windows: &mut [UsageWindow], selected_window_id: Option<&str>) {
    if let Some(selected_window_id) = selected_window_id {
        windows.sort_by_key(|window| {
            if window.id == selected_window_id {
                0
            } else {
                1
            }
        });
    }
}

fn diagnostics(
    snapshot: &CodexSnapshot,
    selected_window_id: Option<String>,
) -> crate::usage::UsageDiagnostics {
    crate::usage::UsageDiagnostics {
        codex_path: Some(snapshot.codex_path.clone()),
        codex_home: snapshot
            .initialize
            .get("codexHome")
            .and_then(Value::as_str)
            .map(str::to_string),
        selected_window_id,
        buckets: diagnostic_buckets(snapshot.rate_limits.as_ref()),
        raw_account_kind: snapshot
            .account
            .get("account")
            .and_then(|account| account.get("type"))
            .and_then(Value::as_str)
            .map(str::to_string),
        requires_openai_auth: snapshot
            .account
            .get("requiresOpenaiAuth")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn diagnostic_buckets(rate_limits: Option<&Value>) -> Vec<crate::usage::UsageDiagnosticBucket> {
    let Some(rate_limits) = rate_limits else {
        return Vec::new();
    };

    if let Some(buckets) = rate_limits
        .get("rateLimitsByLimitId")
        .and_then(Value::as_object)
    {
        return buckets
            .iter()
            .map(|(bucket_id, snapshot)| diagnostic_bucket(bucket_id, snapshot))
            .collect();
    }

    rate_limits
        .get("rateLimits")
        .map(|snapshot| diagnostic_bucket("default", snapshot))
        .into_iter()
        .collect()
}

fn diagnostic_bucket(bucket_id: &str, snapshot: &Value) -> crate::usage::UsageDiagnosticBucket {
    crate::usage::UsageDiagnosticBucket {
        id: bucket_id.to_string(),
        name: snapshot
            .get("limitName")
            .and_then(Value::as_str)
            .map(str::to_string),
        primary_window_id: snapshot
            .get("primary")
            .and_then(|window| window.get("usedPercent"))
            .map(|_| format!("codex-{bucket_id}-primary")),
        secondary_window_id: snapshot
            .get("secondary")
            .and_then(|window| window.get("usedPercent"))
            .map(|_| format!("codex-{bucket_id}-secondary")),
    }
}

fn timestamp_to_rfc3339(value: &Value) -> Option<String> {
    let raw = value.as_i64()?;
    let seconds = if raw > 10_000_000_000 {
        raw.checked_div(1000)?
    } else {
        raw
    };
    let dt: DateTime<Utc> = Utc.timestamp_opt(seconds, 0).single()?;
    Some(dt.to_rfc3339())
}

fn number_like_to_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_i64().map(|number| number.to_string()))
        .or_else(|| value.as_u64().map(|number| number.to_string()))
}
