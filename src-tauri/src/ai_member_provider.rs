use crate::{
    config::AiMemberSettings,
    generic_http_provider::{self, AuthMode, ExtractionRule, HttpSourceConfig, ValueKind},
    usage::ProviderUsage,
};

const DEFAULT_ENDPOINTS: [&str; 4] = [
    "/api/usage/token/",
    "/api/usage/token",
    "/v1/balance",
    "/api/v1/balance",
];

const AMOUNT_PATHS: [&str; 15] = [
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
];

const QUOTA_PATHS: [&str; 4] = [
    "data.total_available",
    "data.remaining_quota",
    "total_available",
    "remaining_quota",
];

const NESTED_PATHS: [&str; 2] = ["result.balance", "result.remaining"];

pub async fn usage_snapshot(settings: &AiMemberSettings) -> ProviderUsage {
    generic_http_provider::usage_snapshot(&source_config(settings)).await
}

fn source_config(settings: &AiMemberSettings) -> HttpSourceConfig {
    let mut extraction_rules = Vec::new();
    extraction_rules.extend(AMOUNT_PATHS.iter().map(|path| ExtractionRule {
        path: path.to_string(),
        divisor: 1.0,
    }));
    extraction_rules.extend(QUOTA_PATHS.iter().map(|path| ExtractionRule {
        path: path.to_string(),
        divisor: settings.balance_divisor,
    }));
    extraction_rules.extend(NESTED_PATHS.iter().map(|path| ExtractionRule {
        path: path.to_string(),
        divisor: 1.0,
    }));
    extraction_rules.push(ExtractionRule {
        path: "cents".to_string(),
        divisor: 100.0,
    });

    HttpSourceConfig {
        provider_id: "ai_member".to_string(),
        label: settings.label.clone(),
        base_url: settings.base_url.clone(),
        endpoint: settings.balance_endpoint.clone(),
        endpoint_fallbacks: DEFAULT_ENDPOINTS
            .iter()
            .map(|endpoint| endpoint.to_string())
            .collect(),
        enabled: settings.enabled,
        api_key: settings.api_key.clone(),
        auth: AuthMode::Bearer,
        headers: vec![
            ("accept-language".to_string(), "zh".to_string()),
            ("referer".to_string(), join_url(&settings.base_url, "/keys")),
            ("x-user-ui-request".to_string(), "1".to_string()),
        ],
        timeout_seconds: 8,
        window_id: "ai-member-balance".to_string(),
        window_key: "balance".to_string(),
        limit_label: "AI-MEMBER balance endpoint".to_string(),
        empty_limit_label: "AI-MEMBER balance".to_string(),
        value_kind: ValueKind::Money,
        low_threshold: settings.low_balance_threshold,
        extraction_rules,
        currency_paths: Vec::new(),
        currency_default: Some("USD".to_string()),
        reset_paths: Vec::new(),
    }
}

fn join_url(base_url: &str, endpoint: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim().trim_end_matches('/'),
        endpoint.trim().trim_start_matches('/')
    )
}
