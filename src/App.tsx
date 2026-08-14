import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { DropdownMenu } from "@radix-ui/themes";
import type { ProviderUsage } from "./lib/usage";
import {
  builtInSources,
  defaultSettings,
  mergeSettings,
  uniqueSourceId,
  type DataSourceSettings,
  type MeterSettings,
  type MeterSource,
} from "./lib/settings";
import { TaskbarMeter } from "./components/TaskbarMeter";
import { UsagePanel } from "./components/UsagePanel";
import { Settings } from "./components/Settings";
import { GlobalSettings } from "./components/GlobalSettings";
import { getCopy } from "./lib/i18n";

type ActivePage = MeterSource | "settings";

const addSourceOptions: Array<{ label: string; create: () => DataSourceSettings }> = [
  { label: "AI-MEMBER", create: builtInSources.aiMember },
  { label: "DeepSeek", create: builtInSources.deepseek },
  { label: "GLM", create: builtInSources.glm },
  { label: "Kimi Code", create: builtInSources.kimi },
  { label: "SiliconFlow", create: builtInSources.siliconflow },
];

function App() {
  const [usage, setUsage] = useState<ProviderUsage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [settings, setSettings] = useState<MeterSettings>(defaultSettings);
  const [draftSettings, setDraftSettings] = useState<MeterSettings>(defaultSettings);
  const [settingsStatusKey, setSettingsStatusKey] = useState("settingsLoading");
  const [activePage, setActivePage] = useState<ActivePage>("chatgpt");
  const [windowLabel] = useState(() => getCurrentWindow().label);
  const isMeterWindow = windowLabel === "meter";
  const activePageRef = useRef<ActivePage>("chatgpt");

  async function refreshUsage(source?: MeterSource) {
    setLoading(true);
    setError(null);

    try {
      const nextUsage = source
        ? await invoke<ProviderUsage>("get_usage_snapshot_for_source", { source })
        : await invoke<ProviderUsage>("get_usage_snapshot");
      setUsage(nextUsage);

      if (isMeterWindow) {
        await invoke("position_meter_window");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadSettings() {
    try {
      const loaded = mergeSettings(await invoke<MeterSettings>("get_meter_settings"));
      setSettings(loaded);
      setDraftSettings(loaded);
      setActivePage(loaded.selectedMeterSource);
      activePageRef.current = loaded.selectedMeterSource;
      setSettingsStatusKey("settingsLoaded");
    } catch (err) {
      setSettingsStatusKey(err instanceof Error ? err.message : String(err));
    }
  }

  async function persistSettings(nextSettings = draftSettings, command = "save_meter_settings") {
    const normalized = mergeSettings(nextSettings);
    setSettingsStatusKey("settingsSaving");

    try {
      const saved = mergeSettings(
        await invoke<MeterSettings>(command, {
          settings: normalized,
        }),
      );
      setSettings(saved);
      setDraftSettings(saved);
      setSettingsStatusKey("settingsSavedRefreshing");
    } catch (err) {
      setSettingsStatusKey(err instanceof Error ? err.message : String(err));
    }
  }

  function switchPage(nextPage: ActivePage) {
    setActivePage(nextPage);
    activePageRef.current = nextPage;

    if (nextPage === "settings") {
      return;
    }

    setDraftSettings((current) => ({
      ...current,
      selectedMeterSource: nextPage,
    }));
    void refreshUsage(nextPage);
  }

  function addSource(createSource: () => DataSourceSettings) {
    const baseSource = createSource();
    const duplicateCount = draftSettings.sources.filter((source) =>
      source.label.startsWith(baseSource.label),
    ).length;
    const nextSource = {
      ...baseSource,
      id: uniqueSourceId(baseSource.id || baseSource.label, draftSettings.sources),
      label: duplicateCount > 0 ? `${baseSource.label} ${duplicateCount + 1}` : baseSource.label,
    };
    const nextSettings = {
      ...draftSettings,
      sources: [...draftSettings.sources, nextSource],
      selectedMeterSource: nextSource.id,
    };
    setDraftSettings(nextSettings);
    setActivePage(nextSource.id);
    activePageRef.current = nextSource.id;
    setUsage(null);
    setLoading(false);
  }

  function deleteSource(sourceId: MeterSource) {
    if (draftSettings.sources.length <= 1) {
      return;
    }

    const sourceIndex = draftSettings.sources.findIndex((source) => source.id === sourceId);
    if (sourceIndex < 0) {
      return;
    }

    const nextSources = draftSettings.sources.filter((source) => source.id !== sourceId);
    const fallbackSource = nextSources[Math.max(0, sourceIndex - 1)] ?? nextSources[0];
    const selectedMeterSource =
      draftSettings.selectedMeterSource === sourceId
        ? fallbackSource.id
        : draftSettings.selectedMeterSource;
    const nextTaskbarSourceIds = draftSettings.taskbarSourceIds.filter((id) => id !== sourceId);
    const nextSettings = {
      ...draftSettings,
      selectedMeterSource,
      sources: nextSources,
      taskbarSourceIds: nextTaskbarSourceIds.length ? nextTaskbarSourceIds : [selectedMeterSource],
      selectedUsageWindowId:
        draftSettings.selectedMeterSource === sourceId ? null : draftSettings.selectedUsageWindowId,
    };

    setDraftSettings(nextSettings);
    setActivePage(selectedMeterSource);
    activePageRef.current = selectedMeterSource;
    setUsage(null);
    setLoading(false);
  }

  useEffect(() => {
    void loadSettings();
    void refreshUsage();
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlistenUsage: (() => void) | undefined;
    let unlistenSettings: (() => void) | undefined;

    async function listenForEvents() {
      unlistenUsage = await listen<ProviderUsage>("usage-updated", (event) => {
        if (disposed) return;

        const currentPage = activePageRef.current;
        if (!isMeterWindow && currentPage !== "settings" && event.payload.provider !== currentPage) {
          return;
        }

        setUsage(event.payload);
        setLoading(false);
        setError(null);
      });

      unlistenSettings = await listen<MeterSettings>("settings-updated", (event) => {
        if (disposed) return;

        const nextSettings = mergeSettings(event.payload);
        setSettings(nextSettings);
        setDraftSettings(nextSettings);
      });
    }

    void listenForEvents();

    return () => {
      disposed = true;
      unlistenUsage?.();
      unlistenSettings?.();
    };
  }, []);

  const activeSource = draftSettings.sources.find((source) => source.id === draftSettings.selectedMeterSource);
  const primaryWindow = usage?.windows[0];
  const meterLabel = activeSource?.label ?? providerDisplayName(usage?.provider);
  const displayLanguage = isMeterWindow ? settings.language : draftSettings.language;
  const meterResetLabel = formatMeterResetLabel(primaryWindow?.resetsAt, usage?.provider, displayLanguage);
  const meterValueLabel = primaryWindow?.valueLabel;
  const meterItems = (usage?.meterItems ?? []).map((item) => ({
    ...item,
    resetLabel: formatMeterResetLabel(item.resetLabel ?? undefined, item.id, displayLanguage),
  }));
  const hasDraftChanges = JSON.stringify(settings) !== JSON.stringify(draftSettings);
  const t = getCopy(draftSettings.language);
  const settingsStatus = localizeSettingsStatus(settingsStatusKey, draftSettings.language);

  if (isMeterWindow) {
    return (
      <main
        className="meter-shell"
        data-tauri-drag-region
        onDoubleClick={() => void invoke("toggle_meter_window")}
      >
        <TaskbarMeter
          loading={loading}
          label={meterLabel}
          resetLabel={meterResetLabel}
          remainingPercent={primaryWindow?.remainingPercent ?? 0}
          valueLabel={meterValueLabel}
          items={meterItems}
          scrollSeconds={settings.taskbarScrollSeconds}
          scrollAnimationSeconds={settings.taskbarScrollAnimationSeconds}
          appearance={settings.taskbarAppearance}
          status={usage?.status ?? "unavailable"}
        />
      </main>
    );
  }

  return (
    <main className="app-shell">
      <div className="top-chrome">
        <div className="top-navigation">
          <nav className="page-tabs dynamic-tabs" aria-label={t("dataSources")}>
            {draftSettings.sources.map((source) => (
              <div className="page-tab" data-active={activePage === source.id} key={source.id}>
                <button
                  aria-current={activePage === source.id ? "page" : undefined}
                  onClick={() => switchPage(source.id)}
                  title={source.label}
                  type="button"
                >
                  {source.label}
                </button>
              </div>
            ))}

            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                <button aria-label={t("addSource")} className="add-source-button" type="button">
                  +
                </button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content align="end" className="add-source-menu" sideOffset={6} variant="solid">
                {addSourceOptions.map((option) => (
                  <DropdownMenu.Item key={option.label} onSelect={() => addSource(option.create)}>
                    {option.label}
                  </DropdownMenu.Item>
                ))}
                <DropdownMenu.Item onSelect={() => addSource(builtInSources.custom)}>
                  {t("addCustomHttp")}
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </nav>

          <button
            aria-current={activePage === "settings" ? "page" : undefined}
            className="settings-entry-button"
            onClick={() => switchPage("settings")}
            type="button"
          >
            {t("settings")}
          </button>
        </div>

        <section className="save-bar" aria-label={t("saveSettingsLabel")}>
          <span>{hasDraftChanges ? t("hasUnsavedSettings") : settingsStatus}</span>
          <button disabled={!hasDraftChanges} onClick={() => void persistSettings()} type="button">
            {t("saveSettings")}
          </button>
        </section>
      </div>

      {activePage === "settings" ? (
        <GlobalSettings onChangeSettings={setDraftSettings} settings={draftSettings} status={settingsStatus} />
      ) : (
        <>
          <section className="meter-band" aria-label={t("usageWindow")}>
            <TaskbarMeter
              loading={loading}
              label={meterLabel}
              resetLabel={meterResetLabel}
              remainingPercent={primaryWindow?.remainingPercent ?? 0}
              valueLabel={meterValueLabel}
              appearance={draftSettings.taskbarAppearance}
              status={usage?.status ?? "unavailable"}
            />
          </section>

          <UsagePanel
            usage={usage}
            error={error}
            loading={loading}
            language={draftSettings.language}
            onRefresh={() => refreshUsage(draftSettings.selectedMeterSource)}
          />
          <Settings
            activeSource={draftSettings.selectedMeterSource}
            onChangeSettings={setDraftSettings}
            onDeleteSource={deleteSource}
            settings={draftSettings}
            status={settingsStatus}
            usage={usage}
          />
        </>
      )}
    </main>
  );
}

function providerDisplayName(provider?: string) {
  const labels: Record<string, string> = {
    chatgpt: "ChatGPT",
    glm: "GLM",
    kimi: "Kimi Code",
    siliconflow: "SiliconFlow",
  };

  return provider ? labels[provider] ?? provider : "AI";
}

function formatMeterResetLabel(value?: string, provider?: string, language: MeterSettings["language"] = "en") {
  const resetTime = formatResetTime(value);
  if (resetTime) {
    return language === "zh" ? `重置 ${resetTime}` : `Reset ${resetTime}`;
  }

  return provider !== "chatgpt" && provider !== "kimi" ? (language === "zh" ? "重置 -" : "Reset -") : null;
}

function formatResetTime(value?: string) {
  if (!value) return null;

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;

  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hour = String(date.getHours()).padStart(2, "0");
  const minute = String(date.getMinutes()).padStart(2, "0");

  return `${month}/${day} ${hour}:${minute}`;
}

function localizeSettingsStatus(statusKey: string, language: MeterSettings["language"]) {
  const t = getCopy(language);
  const statuses: Record<string, string> = {
    settingsLoading: t("settingsLoading"),
    settingsLoaded: t("settingsLoaded"),
    settingsSaving: t("settingsSaving"),
    settingsSavedRefreshing: t("settingsSavedRefreshing"),
  };

  return statuses[statusKey] ?? statusKey;
}

export default App;
