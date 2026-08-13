import type { ProviderUsage } from "../lib/usage";
import type { AppLanguage } from "../lib/settings";
import { getCopy, statusLabel } from "../lib/i18n";

type UsagePanelProps = {
  usage: ProviderUsage | null;
  error: string | null;
  loading: boolean;
  language: AppLanguage;
  onRefresh: () => void;
};

export function UsagePanel({ usage, error, loading, language, onRefresh }: UsagePanelProps) {
  const t = getCopy(language);

  return (
    <section className="usage-panel" aria-label={t("usageDetails")}>
      <header className="panel-header">
        <div>
          <h1>Money Like Water</h1>
          <p>{t("subtitle")}</p>
        </div>
        <button type="button" onClick={onRefresh} disabled={loading}>
          {loading ? t("refreshing") : t("refresh")}
        </button>
      </header>

      {error && <p className="notice error">{error}</p>}

      <dl className="status-grid">
        <div>
          <dt>{t("status")}</dt>
          <dd>
            <span className="status-pill" data-status={usage?.status ?? "loading"}>
              {statusLabel(usage?.status ?? "loading", language)}
            </span>
          </dd>
        </div>
        <div>
          <dt>{t("dataSourceAccount")}</dt>
          <dd>{usage?.accountLabel ?? t("unknown")}</dd>
        </div>
        <div>
          <dt>
            {usage?.provider === "ai_member" || usage?.provider === "kimi" || usage?.provider === "deepseek"
              ? t("serviceAddress")
              : t("plan")}
          </dt>
          <dd>{usage?.planLabel ?? t("unknown")}</dd>
        </div>
        {usage?.creditBalance != null && (
          <div>
            <dt>{language === "zh" ? "剩余金额" : "Remaining balance"}</dt>
            <dd>{formatMoney(usage.creditBalance, usage.provider)}</dd>
          </div>
        )}
        <div>
          <dt>{t("updatedAt")}</dt>
          <dd>
            {usage?.updatedAt
              ? new Date(usage.updatedAt).toLocaleString()
              : language === "zh"
                ? "尚未更新"
                : "Not updated yet"}
          </dd>
        </div>
      </dl>

      <div className="window-list">
        {(usage?.windows ?? []).map((window) => (
          <article
            className="usage-window"
            data-tone={window.remainingPercent <= 10 ? "danger" : window.remainingPercent <= 30 ? "warn" : "good"}
            key={window.id}
          >
            <div>
              <strong>{window.label}</strong>
              <span>
                {window.limitLabel ?? t("usageWindow")}
                {formatWindowResetLabel(window.resetsAt, usage?.provider, language)}
              </span>
            </div>
            <span>
              {window.valueLabel
                ? `${window.valueLabel} ${language === "zh" ? "剩余" : "remaining"}`
                : `${window.remainingPercent}% ${language === "zh" ? "剩余" : "remaining"} / ${window.usedPercent}% ${language === "zh" ? "已用" : "used"}`}
            </span>
          </article>
        ))}
      </div>

      {usage?.message && <p className="notice">{usage.message}</p>}
    </section>
  );
}

function formatMoney(value: number | null | undefined, provider?: ProviderUsage["provider"]) {
  if (value == null || !Number.isFinite(value)) {
    return "Unknown";
  }

  const symbol = provider === "ai_member" ? "$" : "¥";
  return Number.isInteger(value) ? `${symbol}${value}` : `${symbol}${value.toFixed(2)}`;
}

function formatWindowResetLabel(
  value?: string,
  provider?: ProviderUsage["provider"],
  language: AppLanguage = "en",
) {
  if (value) {
    return language === "zh"
      ? ` / 重置 ${new Date(value).toLocaleString()}`
      : ` / reset ${new Date(value).toLocaleString()}`;
  }

  return provider === "ai_member" || provider === "deepseek"
    ? language === "zh"
      ? " / 重置 -"
      : " / reset -"
    : "";
}
