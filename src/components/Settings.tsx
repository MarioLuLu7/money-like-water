import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertDialog } from "@radix-ui/themes";
import type { DataSourceSettings, HeaderSetting, MeterSettings } from "../lib/settings";
import type { ProviderUsage, UsageWindow } from "../lib/usage";
import { getCopy } from "../lib/i18n";
import { FormInput, FormSelect, FormSwitch, FormTextArea } from "./ui";

type SettingsProps = {
  activeSource: string;
  settings: MeterSettings;
  status: string;
  usage: ProviderUsage | null;
  onChangeSettings: (settings: MeterSettings) => void;
  onDeleteSource: (sourceId: string) => void;
};

export function Settings({
  activeSource,
  settings,
  status,
  usage,
  onChangeSettings,
  onDeleteSource,
}: SettingsProps) {
  const [copyStatus, setCopyStatus] = useState("");
  const [tokenStatus, setTokenStatus] = useState("");
  const source = settings.sources.find((item) => item.id === activeSource) ?? settings.sources[0];
  const t = getCopy(settings.language);

  if (!source) return null;

  function updateSource(patch: Partial<DataSourceSettings>) {
    onChangeSettings({
      ...settings,
      sources: settings.sources.map((item) =>
        item.id === source.id ? { ...item, ...patch, enabled: true } : item,
      ),
    });
  }

  function updateHeader(index: number, patch: Partial<HeaderSetting>) {
    updateSource({
      headers: source.headers.map((item, itemIndex) =>
        itemIndex === index ? { ...item, ...patch } : item,
      ),
    });
  }

  function updateTaskbarDisplay(checked: boolean) {
    const nextTaskbarSourceIds = checked
      ? [...settings.taskbarSourceIds, source.id].filter((id, index, ids) => ids.indexOf(id) === index)
      : settings.taskbarSourceIds.filter((id) => id !== source.id);

    onChangeSettings({
      ...settings,
      taskbarSourceIds: nextTaskbarSourceIds.length ? nextTaskbarSourceIds : settings.taskbarSourceIds,
    });
  }

  function updateSelectedUsageWindow(selectedUsageWindowId: string) {
    onChangeSettings({
      ...settings,
      selectedUsageWindowId: selectedUsageWindowId || null,
    });
  }

  async function copyDiagnostics() {
    const payload = {
      activeSource,
      selectedUsageWindowId: settings.selectedUsageWindowId,
      source: {
        ...source,
        apiKey: source.apiKey ? "***" : "",
      },
      status: usage?.status,
      updatedAt: usage?.updatedAt,
      windows: usage?.windows,
      diagnostics: usage?.diagnostics,
      message: usage?.message,
    };

    try {
      await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
      setCopyStatus(settings.language === "zh" ? "已复制" : "Copied");
    } catch (err) {
      setCopyStatus(err instanceof Error ? err.message : String(err));
    }
  }

  async function fillChatgptToken() {
    setTokenStatus(settings.language === "zh" ? "正在读取本机 Codex token" : "Reading local Codex token");

    try {
      const token = await invoke<string>("get_chatgpt_access_token");
      updateSource({ apiKey: token });
      setTokenStatus(
        settings.language === "zh" ? "已填入 access token，保存后生效" : "Access token filled. Save to apply.",
      );
    } catch (err) {
      setTokenStatus(err instanceof Error ? err.message : String(err));
    }
  }

  async function fillAiMemberToken() {
    setTokenStatus(settings.language === "zh" ? "请在打开的 AI-MEMBER 窗口完成登录" : "Complete login in the AI-MEMBER window");

    try {
      const token = await invoke<string>("fetch_ai_member_auth_token");
      updateSource({ apiKey: token });
      setTokenStatus(
        settings.language === "zh" ? "已填入 auth_token，保存后生效" : "auth_token filled. Save to apply.",
      );
    } catch (err) {
      setTokenStatus(err instanceof Error ? err.message : String(err));
    }
  }

  const diagnosticWindows = source.kind === "chatgpt" ? usage?.windows ?? [] : [];
  const canDeleteSource = settings.sources.length > 1;
  const isShownInTaskbar = settings.taskbarSourceIds.includes(source.id);
  const canDisableTaskbarDisplay = !isShownInTaskbar || settings.taskbarSourceIds.length > 1;
  const isAiMemberSource =
    source.id === "ai-member" ||
    source.id.startsWith("ai-member-") ||
    source.baseUrl.trim().replace(/\/$/, "") === "https://proxy.ai-member.icu";

  return (
    <section className="settings-panel" aria-label={t("dataSourceConfig")}>
      <div className="settings-header">
        <div className="settings-title">
          <h2>
            {source.label} {t("dataSourceConfig")}
          </h2>
          <span>{status}</span>
        </div>
        <AlertDialog.Root>
          <AlertDialog.Trigger>
            <button className="danger-button compact-button" disabled={!canDeleteSource} type="button">
              {t("deleteSource")}
            </button>
          </AlertDialog.Trigger>
          <AlertDialog.Content className="delete-source-dialog" maxWidth="420px">
            <AlertDialog.Title>{t("deleteSource")}</AlertDialog.Title>
            <AlertDialog.Description size="2">
              {t("deleteSourceDescription")}
            </AlertDialog.Description>
            <div className="dialog-actions">
              <AlertDialog.Cancel>
                <button className="compact-button" type="button">
                  {t("cancel")}
                </button>
              </AlertDialog.Cancel>
              <AlertDialog.Action>
                <button
                  className="danger-button compact-button"
                  onClick={() => onDeleteSource(source.id)}
                  type="button"
                >
                  {t("confirmDelete")}
                </button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Root>
      </div>

      <div className="settings-control-strip">
        <label className="source-switch-row">
          <FormSwitch
            checked={isShownInTaskbar}
            disabled={!canDisableTaskbarDisplay}
            label={t("showInTaskbar")}
            onCheckedChange={(checked) => updateTaskbarDisplay(checked)}
          />
        </label>
      </div>

      {source.kind === "chatgpt" ? (
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>{t("usageWindows")}</h3>
            <button className="compact-button" onClick={copyDiagnostics} type="button">
              {t("copyDiagnostics")}
            </button>
          </div>

          <label className="field-row field-wide">
            {t("taskbarDisplayWindow")}
            <FormSelect
              onValueChange={(value) => updateSelectedUsageWindow(value === "auto" ? "" : value)}
              options={[
                { value: "auto", label: t("auto") },
                ...diagnosticWindows.map((item) => ({
                  value: item.id,
                  label: windowOptionLabel(item, settings.language),
                })),
              ]}
              value={settings.selectedUsageWindowId ?? "auto"}
            />
          </label>

          <Diagnostics language={settings.language} usage={usage} />
       </div>
     ) : (
        <>
          <div className="settings-section">
            <div className="settings-section-header">
              <h3>{settings.language === "zh" ? "通用 HTTP 接口" : "Generic HTTP endpoint"}</h3>
            </div>

            <div className="form-grid">
              <label className="field-row">
                {t("displayName")}
                <FormInput
                  onChange={(event) => updateSource({ label: event.target.value })}
                  type="text"
                  value={source.label}
                />
              </label>
              <label className="field-row">
                {t("timeoutSeconds")}
                <FormInput
                  min="1"
                  onChange={(event) =>
                    updateSource({ timeoutSeconds: Number.parseInt(event.target.value, 10) || 8 })
                  }
                  step="1"
                  type="number"
                  value={source.timeoutSeconds}
                />
              </label>
              <label className="field-row field-wide">
                Base URL
                <FormInput
                  onChange={(event) => updateSource({ baseUrl: event.target.value })}
                  type="url"
                  value={source.baseUrl}
                />
              </label>
              <label className="field-row field-wide">
                API Key / Token
                <span className="token-input-row">
                  <FormInput
                    onChange={(event) => updateSource({ apiKey: event.target.value })}
                    type="password"
                    value={source.apiKey}
                  />
                  {source.id === "chatgpt" && (
                    <button className="compact-button" onClick={fillChatgptToken} type="button">
                      {t("fetchToken")}
                    </button>
                  )}
                  {isAiMemberSource && (
                    <button className="compact-button" onClick={fillAiMemberToken} type="button">
                      {t("fetchToken")}
                    </button>
                  )}
                </span>
                {(source.id === "chatgpt" || isAiMemberSource) && tokenStatus && (
                  <span className="field-hint">{tokenStatus}</span>
                )}
              </label>
              <label className="field-row">
                {t("endpoint")}
                <FormInput
                  onChange={(event) => updateSource({ endpoint: event.target.value })}
                  type="text"
                  value={source.endpoint}
                />
              </label>
              <label className="field-row">
                {t("lowBalanceThreshold")}
                <FormInput
                  min="0.01"
                  onChange={(event) =>
                    updateSource({ lowBalanceThreshold: Number.parseFloat(event.target.value) || null })
                  }
                  step="0.01"
                  type="number"
                  value={source.lowBalanceThreshold ?? ""}
                />
              </label>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-section-header">
              <h3>Transform JS</h3>
            </div>

            <label className="field-row script-field">
              {t("transformJson")}
              <FormTextArea
                onChange={(event) => updateSource({ transformScript: event.target.value })}
                rows={14}
                spellCheck={false}
                value={source.transformScript}
              />
            </label>

            <p className="notice compact-notice">
              {t("transformDescription")}
            </p>
          </div>

          <div className="settings-section">
            <div className="settings-section-header">
              <h3>{t("requestHeaders")}</h3>
              <button
                className="compact-button"
                onClick={() => updateSource({ headers: [...source.headers, { name: "", value: "" }] })}
                type="button"
              >
                {settings.language === "zh" ? "添加请求头" : "Add header"}
              </button>
            </div>

            <div className="path-list">
              {source.headers.map((header, index) => (
                <div className="path-row" key={`${index}-${header.name}`}>
                  <FormInput
                    aria-label="Header name"
                    onChange={(event) => updateHeader(index, { name: event.target.value })}
                    placeholder="x-api-version"
                    type="text"
                    value={header.name}
                  />
                  <FormInput
                    aria-label="Header value"
                    onChange={(event) => updateHeader(index, { value: event.target.value })}
                    placeholder="value"
                    type="text"
                    value={header.value}
                  />
                  <button
                    aria-label={settings.language === "zh" ? "删除请求头" : "Delete header"}
                    className="icon-button"
                    onClick={() =>
                      updateSource({ headers: source.headers.filter((_, itemIndex) => itemIndex !== index) })
                    }
                    type="button"
                  >
                    x
                  </button>
                </div>
              ))}
              {!source.headers.length && <p className="empty-state">{t("noExtraHeaders")}</p>}
            </div>
          </div>
        </>
      )}

      {copyStatus && <p className="notice">{copyStatus}</p>}
    </section>
  );
}

function Diagnostics({ language, usage }: { language: MeterSettings["language"]; usage: ProviderUsage | null }) {
  const t = getCopy(language);

  return (
    <>
      <dl className="diagnostic-grid">
        <div>
          <dt>{t("codexCli")}</dt>
          <dd>{usage?.diagnostics?.codexPath ?? t("unknown")}</dd>
        </div>
        <div>
          <dt>{t("codexHome")}</dt>
          <dd>{usage?.diagnostics?.codexHome ?? t("unknown")}</dd>
        </div>
        <div>
          <dt>{language === "zh" ? "账号类型" : "Account type"}</dt>
          <dd>{accountKindLabel(usage?.diagnostics?.rawAccountKind, language)}</dd>
        </div>
        <div>
          <dt>{t("apiAuth")}</dt>
          <dd>{usage?.diagnostics?.requiresOpenaiAuth ? t("apiKeyNeeded") : t("apiKeyNotNeeded")}</dd>
        </div>
        <div>
          <dt>{t("currentSelection")}</dt>
          <dd>{usage?.diagnostics?.selectedWindowId ?? t("auto")}</dd>
        </div>
        <div>
          <dt>{t("usagePool")}</dt>
          <dd>{usage?.diagnostics?.buckets.length ?? 0}</dd>
        </div>
      </dl>

      <div className="bucket-list">
        {(usage?.diagnostics?.buckets ?? []).map((bucket) => (
          <article className="bucket-item" key={bucket.id}>
            <strong>{bucket.name ?? bucket.id}</strong>
            <span>{bucket.id}</span>
            <span>
              {bucket.primaryWindowId ? (language === "zh" ? "主窗口" : "Primary window") : ""}
              {bucket.primaryWindowId && bucket.secondaryWindowId ? " / " : ""}
              {bucket.secondaryWindowId ? (language === "zh" ? "次窗口" : "Secondary window") : ""}
            </span>
          </article>
        ))}
      </div>
    </>
  );
}

function windowOptionLabel(item: UsageWindow, language: MeterSettings["language"]) {
  const pieces = [
    item.bucketId ?? (language === "zh" ? "默认" : "Default"),
    item.windowKey ? windowKeyLabel(item.windowKey, language) : item.label,
    item.valueLabel ?? `${item.remainingPercent}% ${language === "zh" ? "剩余" : "remaining"}`,
  ];
  return pieces.join(" / ");
}

function windowKeyLabel(value: string, language: MeterSettings["language"]) {
  const labels: Record<string, Record<string, string>> = {
    en: {
      primary: "Primary window",
      secondary: "Secondary window",
    },
    zh: {
      primary: "主窗口",
      secondary: "次窗口",
    },
  };

  return labels[language][value] ?? value;
}

function accountKindLabel(value: string | undefined, language: MeterSettings["language"]) {
  const labels: Record<MeterSettings["language"], Record<string, string>> = {
    en: {
      chatgpt: "ChatGPT account",
      apiKey: "OpenAI API key",
      amazonBedrock: "Amazon Bedrock",
    },
    zh: {
      chatgpt: "ChatGPT 账号",
      apiKey: "OpenAI API Key",
      amazonBedrock: "Amazon Bedrock",
    },
  };

  return value ? labels[language][value] ?? value : getCopy(language)("unknown");
}
