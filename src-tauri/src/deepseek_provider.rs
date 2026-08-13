use crate::{
    config::DeepSeekSettings,
    generic_http_provider::{self, AuthMode, ExtractionRule, HttpSourceConfig, ValueKind},
    usage::ProviderUsage,
};

pub async fn usage_snapshot(settings: &DeepSeekSettings) -> ProviderUsage {
    generic_http_provider::usage_snapshot(&HttpSourceConfig {
        provider_id: "deepseek".to_string(),
        label: settings.label.clone(),
        base_url: settings.base_url.clone(),
        endpoint: settings.balance_endpoint.clone(),
        endpoint_fallbacks: Vec::new(),
        enabled: settings.enabled,
        api_key: settings.api_key.clone(),
        auth: AuthMode::Bearer,
        headers: Vec::new(),
        timeout_seconds: 8,
        window_id: "deepseek-balance".to_string(),
        window_key: "balance".to_string(),
        limit_label: "DeepSeek API balance".to_string(),
        empty_limit_label: "DeepSeek API balance".to_string(),
        value_kind: ValueKind::Money,
        low_threshold: settings.low_balance_threshold,
        extraction_rules: vec![
            ExtractionRule {
                path: "balance_infos[currency=CNY].total_balance".to_string(),
                divisor: 1.0,
            },
            ExtractionRule {
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
    })
    .await
}
