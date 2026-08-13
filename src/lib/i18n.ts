import type { AppLanguage } from "./settings";
import type { ProviderUsage } from "./usage";

type CopyKey =
  | "addCustomHttp"
  | "addSource"
  | "apiAuth"
  | "apiKeyNeeded"
  | "apiKeyNotNeeded"
  | "auto"
  | "cancel"
  | "codexCli"
  | "codexHome"
  | "confirmDelete"
  | "copyDiagnostics"
  | "currentSelection"
  | "dataSourceAccount"
  | "dataSourceConfig"
  | "dataSources"
  | "deleteSource"
  | "deleteSourceDescription"
  | "displayName"
  | "endpoint"
  | "fetchToken"
  | "globalSettings"
  | "hasUnsavedSettings"
  | "language"
  | "languageEnglish"
  | "languageChinese"
  | "lowBalanceThreshold"
  | "meterAnchor"
  | "noExtraHeaders"
  | "plan"
  | "refresh"
  | "refreshing"
  | "requestHeaders"
  | "resetDash"
  | "saveSettings"
  | "saveSettingsLabel"
  | "serviceAddress"
  | "settings"
  | "settingsLoaded"
  | "settingsLoading"
  | "settingsSavedRefreshing"
  | "settingsSaving"
  | "showInTaskbar"
  | "status"
  | "subtitle"
  | "taskbarAppearance"
  | "taskbarDisplayWindow"
  | "taskbarNotice"
  | "taskbarPosition"
  | "textSize"
  | "resetTextSize"
  | "textColor"
  | "progressColor"
  | "timeoutSeconds"
  | "transformDescription"
  | "transformJson"
  | "usageDetails"
  | "usagePool"
  | "usageWindow"
  | "usageWindows"
  | "updatedAt"
  | "unknown";

const copy: Record<AppLanguage, Record<CopyKey, string>> = {
  en: {
    addCustomHttp: "Custom HTTP",
    addSource: "Add data source",
    apiAuth: "API auth",
    apiKeyNeeded: "Required",
    apiKeyNotNeeded: "Not required",
    auto: "Auto",
    cancel: "Cancel",
    codexCli: "Connector CLI",
    codexHome: "Connector home",
    confirmDelete: "Confirm delete",
    copyDiagnostics: "Copy diagnostics",
    currentSelection: "Current selection",
    dataSourceAccount: "Data source account",
    dataSourceConfig: "Data source settings",
    dataSources: "Data sources",
    deleteSource: "Delete data source",
    deleteSourceDescription:
      "After deleting this source, it will be removed from the top list and taskbar rotation. Before saving, you can still discard this deletion by exiting without saving.",
    displayName: "Display name",
    endpoint: "Endpoint",
    fetchToken: "Fetch token",
    globalSettings: "Global settings",
    hasUnsavedSettings: "Unsaved settings",
    language: "Language",
    languageEnglish: "English",
    languageChinese: "Chinese",
    lowBalanceThreshold: "Low balance threshold",
    meterAnchor: "Usage meter anchor",
    noExtraHeaders: "No extra headers",
    plan: "Plan",
    refresh: "Refresh",
    refreshing: "Refreshing",
    requestHeaders: "Request headers",
    resetDash: "Reset -",
    saveSettings: "Save settings",
    saveSettingsLabel: "Save settings",
    serviceAddress: "Service address",
    settings: "Settings",
    settingsLoaded: "Settings loaded",
    settingsLoading: "Loading settings",
    settingsSavedRefreshing: "Saved, refreshing in the background",
    settingsSaving: "Saving",
    showInTaskbar: "Show in taskbar",
    status: "Status",
    subtitle: "AI usage and balance monitor",
    taskbarAppearance: "Taskbar appearance",
    taskbarDisplayWindow: "Taskbar display window",
    taskbarNotice:
      "Set whether each data source appears in the taskbar from that source's tab. Overall taskbar position, scrolling, and appearance stay here.",
    taskbarPosition: "Taskbar position",
    textSize: "Status text size",
    resetTextSize: "Reset time text size",
    textColor: "Status text color",
    progressColor: "Progress color",
    timeoutSeconds: "Timeout seconds",
    transformDescription:
      "Transform can only access json and source, and must return an object containing a windows array.",
    transformJson: "Response JSON to normalized windows",
    usageDetails: "Usage details",
    usagePool: "Usage pools",
    usageWindow: "Usage window",
    usageWindows: "Usage windows",
    updatedAt: "Updated at",
    unknown: "Unknown",
  },
  zh: {
    addCustomHttp: "自定义 HTTP",
    addSource: "添加数据源",
    apiAuth: "API 认证",
    apiKeyNeeded: "需要",
    apiKeyNotNeeded: "不需要",
    auto: "自动",
    cancel: "取消",
    codexCli: "连接器 CLI",
    codexHome: "连接器目录",
    confirmDelete: "确认删除",
    copyDiagnostics: "复制诊断信息",
    currentSelection: "当前选择",
    dataSourceAccount: "数据源账号",
    dataSourceConfig: "数据源配置",
    dataSources: "数据源",
    deleteSource: "删除数据源",
    deleteSourceDescription:
      "删除该数据源后，它会从顶部列表和任务栏轮播中移除。未保存前仍可通过不保存退出放弃这次删除。",
    displayName: "显示名称",
    endpoint: "接口路径",
    fetchToken: "一键获取",
    globalSettings: "全局设置",
    hasUnsavedSettings: "有未保存的设置",
    language: "语言",
    languageEnglish: "英文",
    languageChinese: "中文",
    lowBalanceThreshold: "低余额阈值",
    meterAnchor: "用量条锚点",
    noExtraHeaders: "没有额外请求头",
    plan: "套餐",
    refresh: "刷新",
    refreshing: "刷新中",
    requestHeaders: "请求头",
    resetDash: "重置 -",
    saveSettings: "保存设置",
    saveSettingsLabel: "保存设置",
    serviceAddress: "服务地址",
    settings: "设置",
    settingsLoaded: "设置已加载",
    settingsLoading: "正在加载设置",
    settingsSavedRefreshing: "已保存，正在后台刷新",
    settingsSaving: "正在保存",
    showInTaskbar: "在任务栏显示",
    status: "状态",
    subtitle: "AI 用量与额度监控",
    taskbarAppearance: "任务栏外观",
    taskbarDisplayWindow: "任务栏显示窗口",
    taskbarNotice:
      "每个数据源是否显示在任务栏，请到对应数据源 tab 下设置。这里保留任务栏整体位置、滚动和外观。",
    taskbarPosition: "任务栏位置",
    textSize: "状态栏文字大小",
    resetTextSize: "重置时间文字大小",
    textColor: "状态栏文字颜色",
    progressColor: "进度条颜色",
    timeoutSeconds: "超时秒数",
    transformDescription:
      "Transform 只能访问 json 和 source，必须 return 包含 windows 数组的对象。",
    transformJson: "响应 JSON 转标准窗口",
    usageDetails: "用量详情",
    usagePool: "用量池",
    usageWindow: "用量窗口",
    usageWindows: "用量窗口",
    updatedAt: "更新时间",
    unknown: "未知",
  },
};

export function getCopy(language: AppLanguage) {
  return (key: CopyKey) => copy[language]?.[key] ?? copy.en[key];
}

export function anchorOptions(language: AppLanguage) {
  return language === "zh"
    ? [
        { value: "left", label: "靠左" },
        { value: "center", label: "居中" },
        { value: "right", label: "靠右" },
      ]
    : [
        { value: "left", label: "Left" },
        { value: "center", label: "Center" },
        { value: "right", label: "Right" },
      ];
}

export function statusLabel(status: ProviderUsage["status"] | "loading", language: AppLanguage) {
  const labels: Record<AppLanguage, Record<ProviderUsage["status"] | "loading", string>> = {
    en: {
      ok: "OK",
      codex_missing: "Connector unavailable",
      logged_out: "Logged out",
      unauthorized: "Unauthorized",
      unavailable: "Unavailable",
      error: "Error",
      loading: "Loading",
    },
    zh: {
      ok: "正常",
      codex_missing: "连接器不可用",
      logged_out: "未登录",
      unauthorized: "无权限",
      unavailable: "不可用",
      error: "错误",
      loading: "加载中",
    },
  };

  return labels[language][status];
}

