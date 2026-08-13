use crate::usage::{ProviderUsage, ProviderUsageStatus, UsageWindow};
use chrono::Utc;
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct HttpSourceConfig {
    pub provider_id: String,
    pub label: String,
    pub base_url: String,
    pub endpoint: String,
    pub endpoint_fallbacks: Vec<String>,
    pub enabled: bool,
    pub api_key: String,
    pub auth: AuthMode,
    pub headers: Vec<(String, String)>,
    pub timeout_seconds: u64,
    pub window_id: String,
    pub window_key: String,
    pub limit_label: String,
    pub empty_limit_label: String,
    pub value_kind: ValueKind,
    pub low_threshold: Option<f64>,
    pub extraction_rules: Vec<ExtractionRule>,
    pub currency_paths: Vec<String>,
    pub currency_default: Option<String>,
    pub reset_paths: Vec<String>,
    pub transform_script: String,
    pub window_rules: Vec<WindowRule>,
    pub array_window_rules: Vec<ArrayWindowRule>,
}

#[derive(Clone, Copy)]
pub enum AuthMode {
    Bearer,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ValueKind {
    Money,
    Percent,
}

#[derive(Clone)]
pub struct ExtractionRule {
    pub path: String,
    pub divisor: f64,
}

#[derive(Clone)]
pub struct WindowRule {
    pub id: String,
    pub label: String,
    pub root_path: String,
    pub limit_path: String,
    pub remaining_path: String,
    pub used_path: Option<String>,
    pub reset_path: Option<String>,
    pub limit_label: Option<String>,
    pub window_key: Option<String>,
}

#[derive(Clone)]
pub struct ArrayWindowRule {
    pub id_prefix: String,
    pub label: String,
    pub array_path: String,
    pub item_root_path: String,
    pub limit_path: String,
    pub remaining_path: String,
    pub used_path: Option<String>,
    pub reset_path: Option<String>,
    pub limit_label: Option<String>,
    pub window_key_prefix: Option<String>,
}

pub async fn usage_snapshot(config: &HttpSourceConfig) -> ProviderUsage {
    if !config.enabled {
        return unavailable_snapshot(config, format!("{} data source is disabled.", config.label));
    }

    if config.api_key.trim().is_empty() {
        return unavailable_snapshot(config, format!("{} API key is empty.", config.label));
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1)))
        .build()
    {
        Ok(client) => client,
        Err(err) => return error_snapshot(config, format!("Unable to create HTTP client: {err}")),
    };

    let mut failures = Vec::new();
    for endpoint in endpoint_candidates(config) {
        let url = join_url(&config.base_url, &endpoint);
        let mut request = client
            .get(&url)
            .header("accept", "application/json")
            .header("user-agent", "MoneyLikeWater/0.1");

        match config.auth {
            AuthMode::Bearer => {
                request = request.bearer_auth(config.api_key.trim());
            }
        }

        for (name, value) in &config.headers {
            request = request.header(name, value);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(err) => {
                failures.push(format!("{url}: {err}"));
                continue;
            }
        };

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return ProviderUsage {
                provider: config.provider_id.clone(),
                account_label: Some(config.label.clone()),
                plan_label: Some(config.base_url.clone()),
                credit_balance: None,
                status: ProviderUsageStatus::Unauthorized,
                windows: empty_windows(config),
                meter_items: Vec::new(),
                updated_at: Some(Utc::now().to_rfc3339()),
                message: Some(format!("{url} returned {status}; check the API key.")),
                diagnostics: None,
            };
        }

        if !status.is_success() {
            failures.push(format!("{url}: HTTP {status}"));
            continue;
        }

        let value = match response.json::<Value>().await {
            Ok(value) => value,
            Err(err) => {
                failures.push(format!("{url}: invalid JSON: {err}"));
                continue;
            }
        };

        if !config.transform_script.trim().is_empty() {
            match transform_snapshot(config, &value) {
                Ok(snapshot) => return snapshot,
                Err(err) => {
                    failures.push(format!("{url}: transform failed: {err}"));
                    continue;
                }
            }
        }

        let windows = mapped_windows(config, &value);
        if !windows.is_empty() {
            return ok_snapshot_with_windows(config, windows);
        }

        let Some(amount) = extract_number(&value, &config.extraction_rules) else {
            failures.push(format!("{url}: no configured value path matched"));
            continue;
        };

        return ok_snapshot(config, &value, amount, endpoint);
    }

    error_snapshot(
        config,
        format!("{} query failed. {}", config.label, failures.join("; ")),
    )
}

fn transform_snapshot(config: &HttpSourceConfig, raw: &Value) -> Result<ProviderUsage, String> {
    let source = serde_json::json!({
        "id": config.provider_id,
        "label": config.label,
        "baseUrl": config.base_url,
        "endpoint": config.endpoint,
        "lowBalanceThreshold": config.low_threshold,
    });
    let input = serde_json::json!({
        "script": config.transform_script,
        "json": raw,
        "source": source,
    });
    let output = run_transform_process(&input)?;
    provider_usage_from_transform(config, &output)
}

fn run_transform_process(input: &Value) -> Result<Value, String> {
    let runner = r#"
const vm = require('vm');
const chunks = [];
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => chunks.push(chunk));
process.stdin.on('end', () => {
  try {
    const input = JSON.parse(chunks.join(''));
    const sandbox = {
      json: input.json,
      source: input.source,
      Math,
      Number,
      String,
      Boolean,
      Array,
      Object,
      Date,
      JSON,
    };
    const wrapped = `(function(){${input.script}\n})()`;
    const result = vm.runInNewContext(wrapped, sandbox, { timeout: 1000 });
    process.stdout.write(JSON.stringify(result));
  } catch (error) {
    process.stderr.write(error && error.stack ? error.stack : String(error));
    process.exit(1);
  }
});
"#;

    let mut child = Command::new("node")
        .arg("-e")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Node.js is required to run transform scripts: {err}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(input.to_string().as_bytes())
            .map_err(|err| err.to_string())?;
    }
    drop(child.stdin.take());

    let started = Instant::now();
    loop {
        match child.try_wait().map_err(|err| err.to_string())? {
            Some(_) => break,
            None if started.elapsed() > Duration::from_secs(2) => {
                let _ = child.kill();
                return Err("transform timed out".to_string());
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

fn provider_usage_from_transform(
    config: &HttpSourceConfig,
    value: &Value,
) -> Result<ProviderUsage, String> {
    let windows_value = value
        .get("windows")
        .and_then(Value::as_array)
        .ok_or_else(|| "transform result must include windows[]".to_string())?;
    let mut windows = Vec::new();

    for (index, item) in windows_value.iter().enumerate() {
        let remaining_percent = item
            .get("remainingPercent")
            .and_then(number_like)
            .ok_or_else(|| format!("windows[{index}].remainingPercent is required"))?
            .round()
            .clamp(0.0, 100.0) as u8;
        let used_percent = item
            .get("usedPercent")
            .and_then(number_like)
            .map(|value| value.round().clamp(0.0, 100.0) as u8)
            .unwrap_or_else(|| 100u8.saturating_sub(remaining_percent));

        windows.push(UsageWindow {
            id: item
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("{}-{index}", config.provider_id)),
            label: item
                .get("label")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| config.label.clone()),
            used_percent,
            remaining_percent,
            value_label: item
                .get("valueLabel")
                .and_then(Value::as_str)
                .map(str::to_string),
            resets_at: item
                .get("resetsAt")
                .and_then(Value::as_str)
                .map(str::to_string),
            limit_label: item
                .get("limitLabel")
                .and_then(Value::as_str)
                .map(str::to_string),
            bucket_id: item
                .get("bucketId")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| Some(config.provider_id.clone())),
            window_key: item
                .get("windowKey")
                .and_then(Value::as_str)
                .map(str::to_string),
        });
    }

    if windows.is_empty() {
        return Err("transform returned no windows".to_string());
    }

    Ok(ProviderUsage {
        provider: config.provider_id.clone(),
        account_label: value
            .get("accountLabel")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| Some(config.label.clone())),
        plan_label: value
            .get("planLabel")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| Some(config.base_url.clone())),
        credit_balance: value.get("creditBalance").and_then(number_like),
        status: ProviderUsageStatus::Ok,
        windows,
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: value
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| Some(format!("{} updated.", config.label))),
        diagnostics: None,
    })
}

fn ok_snapshot_with_windows(config: &HttpSourceConfig, windows: Vec<UsageWindow>) -> ProviderUsage {
    let message = windows
        .first()
        .map(|window| {
            format!(
                "{} {}: {}",
                config.label,
                window.label,
                window
                    .value_label
                    .clone()
                    .unwrap_or_else(|| format!("{}%", window.remaining_percent))
            )
        })
        .unwrap_or_else(|| format!("{} updated.", config.label));

    ProviderUsage {
        provider: config.provider_id.clone(),
        account_label: Some(config.label.clone()),
        plan_label: Some(config.base_url.clone()),
        credit_balance: None,
        status: ProviderUsageStatus::Ok,
        windows,
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: Some(message),
        diagnostics: None,
    }
}

fn ok_snapshot(
    config: &HttpSourceConfig,
    raw: &Value,
    amount: f64,
    endpoint: String,
) -> ProviderUsage {
    let remaining_percent = percent(amount, config.low_threshold);
    let currency =
        extract_string(raw, &config.currency_paths).or_else(|| config.currency_default.clone());
    let value_label = format_value(amount, config.value_kind, currency.as_deref());
    let resets_at = extract_string(raw, &config.reset_paths);

    ProviderUsage {
        provider: config.provider_id.clone(),
        account_label: Some(config.label.clone()),
        plan_label: Some(config.base_url.clone()),
        credit_balance: Some(amount),
        status: ProviderUsageStatus::Ok,
        windows: vec![UsageWindow {
            id: config.window_id.clone(),
            label: config.label.clone(),
            used_percent: 100u8.saturating_sub(remaining_percent),
            remaining_percent,
            value_label: Some(value_label.clone()),
            resets_at,
            limit_label: Some(format!("{} {}", config.limit_label, endpoint)),
            bucket_id: Some(config.provider_id.clone()),
            window_key: Some(config.window_key.clone()),
        }],
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: Some(format!("{} current value: {value_label}", config.label)),
        diagnostics: None,
    }
}

fn unavailable_snapshot(config: &HttpSourceConfig, message: impl Into<String>) -> ProviderUsage {
    ProviderUsage {
        provider: config.provider_id.clone(),
        account_label: Some(config.label.clone()),
        plan_label: Some(config.base_url.clone()),
        credit_balance: None,
        status: ProviderUsageStatus::Unavailable,
        windows: empty_windows(config),
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: Some(message.into()),
        diagnostics: None,
    }
}

fn error_snapshot(config: &HttpSourceConfig, message: impl Into<String>) -> ProviderUsage {
    ProviderUsage {
        provider: config.provider_id.clone(),
        account_label: Some(config.label.clone()),
        plan_label: Some(config.base_url.clone()),
        credit_balance: None,
        status: ProviderUsageStatus::Error,
        windows: empty_windows(config),
        meter_items: Vec::new(),
        updated_at: Some(Utc::now().to_rfc3339()),
        message: Some(message.into()),
        diagnostics: None,
    }
}

fn empty_windows(config: &HttpSourceConfig) -> Vec<UsageWindow> {
    vec![UsageWindow {
        id: config.window_id.clone(),
        label: config.label.clone(),
        used_percent: 0,
        remaining_percent: 0,
        value_label: None,
        resets_at: None,
        limit_label: Some(config.empty_limit_label.clone()),
        bucket_id: Some(config.provider_id.clone()),
        window_key: Some(config.window_key.clone()),
    }]
}

fn endpoint_candidates(config: &HttpSourceConfig) -> Vec<String> {
    let mut endpoints = Vec::new();
    for endpoint in std::iter::once(&config.endpoint).chain(config.endpoint_fallbacks.iter()) {
        let normalized = normalize_endpoint(endpoint);
        if !normalized.is_empty() && !endpoints.contains(&normalized) {
            endpoints.push(normalized);
        }
    }
    endpoints
}

fn normalize_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        String::new()
    } else if endpoint.starts_with('/') {
        endpoint.to_string()
    } else {
        format!("/{endpoint}")
    }
}

fn join_url(base_url: &str, endpoint: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim().trim_end_matches('/'),
        endpoint.trim().trim_start_matches('/')
    )
}

fn extract_number(value: &Value, rules: &[ExtractionRule]) -> Option<f64> {
    for rule in rules {
        if let Some(number) = value_at_path(value, &rule.path).and_then(number_like) {
            return Some(number / rule.divisor.max(1.0));
        }
    }
    None
}

fn extract_string(value: &Value, paths: &[String]) -> Option<String> {
    paths.iter().find_map(|path| {
        let value = value_at_path(value, path)?;
        value
            .as_str()
            .map(str::to_string)
            .or_else(|| number_like(value).map(|number| number.to_string()))
    })
}

fn mapped_windows(config: &HttpSourceConfig, raw: &Value) -> Vec<UsageWindow> {
    let mut windows = Vec::new();

    for rule in &config.window_rules {
        let root = if rule.root_path.trim().is_empty() {
            raw
        } else if let Some(root) = value_at_path(raw, &rule.root_path) {
            root
        } else {
            continue;
        };

        if let Some(window) = map_window(config, root, rule) {
            windows.push(window);
        }
    }

    for rule in &config.array_window_rules {
        let Some(items) = value_at_path(raw, &rule.array_path).and_then(Value::as_array) else {
            continue;
        };

        for (index, item) in items.iter().enumerate() {
            let root = if rule.item_root_path.trim().is_empty() {
                item
            } else if let Some(root) = value_at_path(item, &rule.item_root_path) {
                root
            } else {
                continue;
            };

            if let Some(window) = map_array_window(config, root, rule, index) {
                windows.push(window);
            }
        }
    }

    windows
}

fn map_window(config: &HttpSourceConfig, value: &Value, rule: &WindowRule) -> Option<UsageWindow> {
    let limit = value_at_path(value, &rule.limit_path).and_then(number_like)?;
    let remaining = value_at_path(value, &rule.remaining_path).and_then(number_like)?;
    let used = rule
        .used_path
        .as_deref()
        .and_then(|path| value_at_path(value, path))
        .and_then(number_like)
        .unwrap_or_else(|| (limit - remaining).max(0.0));

    let remaining_percent = percent_from_limit(remaining, limit, config.low_threshold);
    let used_percent = if limit > 0.0 {
        ((used / limit) * 100.0).clamp(0.0, 100.0).round() as u8
    } else {
        100u8.saturating_sub(remaining_percent)
    };

    Some(UsageWindow {
        id: format!("{}-{}", config.provider_id, rule.id),
        label: rule.label.clone(),
        used_percent,
        remaining_percent,
        value_label: Some(format_value(
            remaining_percent as f64,
            ValueKind::Percent,
            None,
        )),
        resets_at: rule
            .reset_path
            .as_deref()
            .and_then(|path| value_at_path(value, path))
            .and_then(|value| value.as_str().map(str::to_string)),
        limit_label: rule.limit_label.clone(),
        bucket_id: Some(config.provider_id.clone()),
        window_key: rule.window_key.clone().or_else(|| Some(rule.id.clone())),
    })
}

fn map_array_window(
    config: &HttpSourceConfig,
    value: &Value,
    rule: &ArrayWindowRule,
    index: usize,
) -> Option<UsageWindow> {
    let window_key = format!(
        "{}-{index}",
        rule.window_key_prefix
            .clone()
            .unwrap_or_else(|| rule.id_prefix.clone())
    );
    let single = WindowRule {
        id: format!("{}-{index}", rule.id_prefix),
        label: format!("{} {}", rule.label, index + 1),
        root_path: String::new(),
        limit_path: rule.limit_path.clone(),
        remaining_path: rule.remaining_path.clone(),
        used_path: rule.used_path.clone(),
        reset_path: rule.reset_path.clone(),
        limit_label: rule.limit_label.clone(),
        window_key: Some(window_key),
    };

    map_window(config, value, &single)
}

fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.').filter(|segment| !segment.trim().is_empty()) {
        current = apply_segment(current, segment.trim())?;
    }
    Some(current)
}

fn apply_segment<'a>(value: &'a Value, segment: &str) -> Option<&'a Value> {
    let Some(open) = segment.find('[') else {
        return value.get(segment);
    };
    let key = &segment[..open];
    let close = segment.rfind(']')?;
    let selector = &segment[open + 1..close];
    let selected = if key.is_empty() {
        value
    } else {
        value.get(key)?
    };
    let array = selected.as_array()?;

    if let Ok(index) = selector.parse::<usize>() {
        return array.get(index);
    }

    let (filter_key, filter_value) = selector.split_once('=')?;
    array.iter().find(|item| {
        item.get(filter_key.trim())
            .and_then(Value::as_str)
            .map(|value| value == filter_value.trim())
            .unwrap_or(false)
    })
}

fn number_like(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|number| number as f64))
        .or_else(|| value.as_u64().map(|number| number as f64))
        .or_else(|| {
            value
                .as_str()
                .and_then(|raw| raw.trim().parse::<f64>().ok())
        })
}

fn percent(value: f64, threshold: Option<f64>) -> u8 {
    let threshold = threshold.unwrap_or(10.0).max(0.01);
    ((value / threshold) * 100.0).clamp(0.0, 100.0).round() as u8
}

fn percent_from_limit(remaining: f64, limit: f64, threshold: Option<f64>) -> u8 {
    if limit > 0.0 {
        return ((remaining / limit) * 100.0).clamp(0.0, 100.0).round() as u8;
    }

    percent(remaining, threshold)
}

fn format_value(value: f64, kind: ValueKind, currency: Option<&str>) -> String {
    match kind {
        ValueKind::Percent => format!("{}%", value.round().clamp(0.0, 100.0) as u8),
        ValueKind::Money => {
            let symbol = match currency.unwrap_or("USD") {
                "USD" => "$",
                "CNY" => "CNY ",
                other => other,
            };
            if value.fract().abs() < f64::EPSILON {
                format!("{symbol}{value:.0}")
            } else {
                format!("{symbol}{value:.2}")
            }
        }
    }
}
