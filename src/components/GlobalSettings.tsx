import type { AppLanguage, MeterAnchor, MeterSettings } from "../lib/settings";
import { anchorOptions, getCopy } from "../lib/i18n";
import { FormInput, FormSelect } from "./ui";

const textSizeOptions = Array.from({ length: 13 }, (_, index) => {
  const value = 8 + index;
  return { value: String(value), label: `${value}px` };
});

type GlobalSettingsProps = {
  settings: MeterSettings;
  status: string;
  updateStatus: string;
  onChangeSettings: (settings: MeterSettings) => void;
  onCheckForUpdates: () => void;
};

export function GlobalSettings({
  settings,
  status,
  updateStatus,
  onChangeSettings,
  onCheckForUpdates,
}: GlobalSettingsProps) {
  const t = getCopy(settings.language);
  const localizedAnchorOptions = anchorOptions(settings.language) as Array<{
    value: MeterAnchor;
    label: string;
  }>;

  function updateLanguage(language: AppLanguage) {
    onChangeSettings({ ...settings, language });
  }

  function updateAnchor(anchor: MeterAnchor) {
    onChangeSettings({ ...settings, anchor });
  }

  function updateOffset(key: keyof MeterSettings["offsets"], value: string) {
    const nextValue = Number.parseInt(value, 10);
    onChangeSettings({
      ...settings,
      offsets: {
        ...settings.offsets,
        [key]: Number.isFinite(nextValue) ? nextValue : 0,
      },
    });
  }

  function updateScrollSeconds(value: string) {
    const nextValue = Number.parseFloat(value);
    onChangeSettings({
      ...settings,
      taskbarScrollSeconds: Number.isFinite(nextValue) ? Math.max(0.5, nextValue) : 3.2,
    });
  }

  function updateScrollAnimationSeconds(value: string) {
    const nextValue = Number.parseFloat(value);
    onChangeSettings({
      ...settings,
      taskbarScrollAnimationSeconds: Number.isFinite(nextValue)
        ? Math.max(0.05, Math.min(2, nextValue))
        : 0.35,
    });
  }

  function updateQueryRefreshSeconds(value: string) {
    const nextValue = Number.parseFloat(value);
    onChangeSettings({
      ...settings,
      queryRefreshSeconds: Number.isFinite(nextValue)
        ? Math.max(5, Math.min(3600, nextValue))
        : 60,
    });
  }

  function updateAppearance(patch: Partial<MeterSettings["taskbarAppearance"]>) {
    onChangeSettings({
      ...settings,
      taskbarAppearance: {
        ...settings.taskbarAppearance,
        ...patch,
      },
    });
  }

  function updateTextSize(value: string) {
    const nextValue = Number.parseInt(value, 10);
    updateAppearance({
      textSizePx: Number.isFinite(nextValue) ? Math.max(8, Math.min(20, nextValue)) : 9,
    });
  }

  function updateResetTextSize(value: string) {
    const nextValue = Number.parseInt(value, 10);
    updateAppearance({
      resetTextSizePx: Number.isFinite(nextValue) ? Math.max(8, Math.min(20, nextValue)) : 8,
    });
  }

  return (
    <section className="settings-panel" aria-label={t("globalSettings")}>
      <div className="settings-header">
        <div className="settings-title">
          <h2>{t("globalSettings")}</h2>
          <span>{status}</span>
        </div>
      </div>

      <div className="settings-section">
        <div className="settings-section-header">
          <h3>{t("appUpdates")}</h3>
        </div>
        <div className="update-row">
          <div>
            <strong>{t("autoUpdates")}</strong>
            <span>{t("autoUpdatesDescription")}</span>
          </div>
          <button className="compact-button" onClick={onCheckForUpdates} type="button">
            {t("checkForUpdates")}
          </button>
        </div>
        {updateStatus && <p className="compact-notice">{updateStatus}</p>}
      </div>

      <div className="settings-section">
        <div className="settings-section-header">
          <h3>{t("settings")}</h3>
        </div>
        <div className="form-grid compact-grid">
          <label className="field-row">
            {t("language")}
            <FormSelect
              onValueChange={(value) => updateLanguage(value as AppLanguage)}
              options={[
                { value: "en", label: t("languageEnglish") },
                { value: "zh", label: t("languageChinese") },
              ]}
              value={settings.language}
            />
          </label>
          <label className="field-row">
            {settings.language === "zh" ? "查询接口时间间隔" : "Query refresh interval"}
            <FormInput
              max="3600"
              min="5"
              onChange={(event) => updateQueryRefreshSeconds(event.target.value)}
              step="1"
              type="number"
              value={settings.queryRefreshSeconds}
            />
          </label>
          <label className="field-row">
            {settings.language === "zh" ? "滚动时间间隔" : "Scroll interval"}
            <FormInput
              min="0.5"
              onChange={(event) => updateScrollSeconds(event.target.value)}
              step="0.1"
              type="number"
              value={settings.taskbarScrollSeconds}
            />
          </label>
          <label className="field-row">
            {settings.language === "zh" ? "滚动动画时长" : "Scroll animation duration"}
            <FormInput
              max="2"
              min="0.05"
              onChange={(event) => updateScrollAnimationSeconds(event.target.value)}
              step="0.05"
              type="number"
              value={settings.taskbarScrollAnimationSeconds}
            />
          </label>
        </div>
      </div>

      <div className="settings-section">
        <div className="settings-section-header">
          <h3>{t("taskbarAppearance")}</h3>
        </div>
        <div className="form-grid compact-grid">
          <label className="field-row">
            {t("textSize")}
            <FormSelect
              onValueChange={(value) => updateTextSize(value)}
              options={textSizeOptions}
              value={String(settings.taskbarAppearance.textSizePx)}
            />
          </label>
          <label className="field-row">
            {t("resetTextSize")}
            <FormSelect
              onValueChange={(value) => updateResetTextSize(value)}
              options={textSizeOptions}
              value={String(settings.taskbarAppearance.resetTextSizePx)}
            />
          </label>
          <label className="field-row color-field">
            {t("textColor")}
            <input
              onChange={(event) => updateAppearance({ textColor: event.target.value })}
              type="color"
              value={settings.taskbarAppearance.textColor}
            />
          </label>
          <label className="field-row color-field">
            {t("progressColor")}
            <input
              onChange={(event) => updateAppearance({ progressColor: event.target.value })}
              type="color"
              value={settings.taskbarAppearance.progressColor}
            />
          </label>
        </div>
      </div>

      <div className="settings-section">
        <div className="settings-section-header">
          <h3>{t("taskbarPosition")}</h3>
        </div>

        <div className="segmented-control" role="group" aria-label={t("meterAnchor")}>
          {localizedAnchorOptions.map((option) => (
            <button
              aria-pressed={settings.anchor === option.value}
              key={option.value}
              onClick={() => updateAnchor(option.value)}
              type="button"
            >
              {option.label}
            </button>
          ))}
        </div>

        <div className="offset-grid">
          <label>
            {settings.language === "zh" ? "左偏移" : "Left offset"}
            <FormInput
              max="400"
              min="-400"
              onChange={(event) => updateOffset("left", event.target.value)}
              step="1"
              type="number"
              value={settings.offsets.left}
            />
          </label>
          <label>
            {settings.language === "zh" ? "右偏移" : "Right offset"}
            <FormInput
              max="400"
              min="-400"
              onChange={(event) => updateOffset("right", event.target.value)}
              step="1"
              type="number"
              value={settings.offsets.right}
            />
          </label>
          <label>
            {settings.language === "zh" ? "上偏移" : "Top offset"}
            <FormInput
              max="160"
              min="-160"
              onChange={(event) => updateOffset("top", event.target.value)}
              step="1"
              type="number"
              value={settings.offsets.top}
            />
          </label>
          <label>
            {settings.language === "zh" ? "下偏移" : "Bottom offset"}
            <FormInput
              max="160"
              min="-160"
              onChange={(event) => updateOffset("bottom", event.target.value)}
              step="1"
              type="number"
              value={settings.offsets.bottom}
            />
          </label>
        </div>
      </div>

      <p className="notice">
        {t("taskbarNotice")}
      </p>
    </section>
  );
}
