export type MeterAnchor = "left" | "center" | "right";
export type MeterSource = string;
export type DataSourceKind = "chatgpt" | "http";
export type ValueFormat = "money" | "percent";
export type AppLanguage = "en" | "zh";

export type ValuePathSetting = {
  path: string;
  divisor: number;
};

export type HeaderSetting = {
  name: string;
  value: string;
};

export type HttpParserSettings = {
  valuePaths: ValuePathSetting[];
  currencyPaths: string[];
  currencyDefault?: string | null;
  resetPaths: string[];
  valueFormat: ValueFormat;
  windows: HttpWindowSettings[];
  arrayWindows: HttpArrayWindowSettings[];
};

export type HttpWindowSettings = {
  id: string;
  label: string;
  rootPath: string;
  limitPath: string;
  remainingPath: string;
  usedPath?: string | null;
  resetPath?: string | null;
  limitLabel?: string | null;
  windowKey?: string | null;
};

export type HttpArrayWindowSettings = {
  idPrefix: string;
  label: string;
  arrayPath: string;
  itemRootPath: string;
  limitPath: string;
  remainingPath: string;
  usedPath?: string | null;
  resetPath?: string | null;
  limitLabel?: string | null;
  windowKeyPrefix?: string | null;
};

export type DataSourceSettings = {
  id: string;
  kind: DataSourceKind;
  enabled: boolean;
  label: string;
  baseUrl: string;
  endpoint: string;
  apiKey: string;
  authMode: "bearer" | "raw";
  headers: HeaderSetting[];
  parser: HttpParserSettings;
  transformScript: string;
  lowBalanceThreshold?: number | null;
  timeoutSeconds: number;
};

export type AiMemberSettings = {
  enabled: boolean;
  label: string;
  baseUrl: string;
  apiKey: string;
  balanceEndpoint: string;
  lowBalanceThreshold?: number | null;
  balanceDivisor: number;
};

export type TaskbarAppearance = {
  textSizePx: number;
  resetTextSizePx: number;
  textColor: string;
  progressColor: string;
};

export type KimiSettings = {
  enabled: boolean;
  label: string;
  baseUrl: string;
  apiKey: string;
  usageEndpoint: string;
  lowBalanceThreshold?: number | null;
};

export type DeepSeekSettings = {
  enabled: boolean;
  label: string;
  baseUrl: string;
  apiKey: string;
  balanceEndpoint: string;
  lowBalanceThreshold?: number | null;
};

export type MeterSettings = {
  language: AppLanguage;
  anchor: MeterAnchor;
  offsets: {
    left: number;
    right: number;
    top: number;
    bottom: number;
  };
  selectedUsageWindowId?: string | null;
  selectedMeterSource: MeterSource;
  sources: DataSourceSettings[];
  taskbarSourceIds: string[];
  aiMember: AiMemberSettings;
  kimi: KimiSettings;
  deepseek: DeepSeekSettings;
  taskbarSources: {
    chatgpt: boolean;
    aiMember: boolean;
    kimi: boolean;
    deepseek: boolean;
  };
  taskbarScrollSeconds: number;
  taskbarScrollAnimationSeconds: number;
  taskbarAppearance: TaskbarAppearance;
  queryRefreshSeconds: number;
};

export const builtInSources = {
  chatgpt: (): DataSourceSettings => ({
    id: "chatgpt",
    kind: "http",
    enabled: true,
    label: "ChatGPT",
    baseUrl: "https://chatgpt.com/backend-api",
    endpoint: "/wham/usage",
    apiKey: "",
    authMode: "bearer",
    headers: [{ name: "user-agent", value: "codex-cli" }],
    parser: defaultParser(),
    transformScript: chatgptTransformScript(),
    lowBalanceThreshold: 30,
    timeoutSeconds: 8,
  }),
  aiMember: (): DataSourceSettings => ({
    id: "ai-member",
    kind: "http",
    enabled: true,
    label: "AI-MEMBER",
    baseUrl: "https://proxy.ai-member.icu",
    endpoint: "/api/v1/auth/me?timezone=Asia%2FShanghai",
    apiKey: "",
    authMode: "bearer",
    headers: [
      { name: "accept-language", value: "zh" },
      { name: "x-user-ui-request", value: "1" },
    ],
    parser: {
      valueFormat: "money",
      valuePaths: [
        { path: "data.balance", divisor: 1 },
        { path: "balance", divisor: 1 },
        { path: "data.total_available", divisor: 500000 },
        { path: "cents", divisor: 100 },
      ],
      currencyPaths: [],
      currencyDefault: "USD",
      resetPaths: [],
      windows: [],
      arrayWindows: [],
    },
    transformScript: aiMemberTransformScript(),
    lowBalanceThreshold: 10,
    timeoutSeconds: 8,
  }),
  kimi: (): DataSourceSettings => ({
    id: "kimi",
    kind: "http",
    enabled: true,
    label: "Kimi Code",
    baseUrl: "https://api.kimi.com/coding/v1",
    endpoint: "/usages",
    apiKey: "",
    authMode: "bearer",
    headers: [],
    parser: {
      ...defaultParser(),
      valueFormat: "percent",
      currencyDefault: null,
      windows: [
        {
          id: "weekly",
          label: "周额度",
          rootPath: "usage",
          limitPath: "limit",
          remainingPath: "remaining",
          usedPath: "used",
          resetPath: "resetTime",
          limitLabel: "Kimi Code 周额度",
          windowKey: "weekly",
        },
      ],
      arrayWindows: [],
    },
    transformScript: kimiTransformScript(),
    lowBalanceThreshold: 30,
    timeoutSeconds: 8,
  }),
  deepseek: (): DataSourceSettings => ({
    id: "deepseek",
    kind: "http",
    enabled: true,
    label: "DeepSeek",
    baseUrl: "https://api.deepseek.com",
    endpoint: "/user/balance",
    apiKey: "",
    authMode: "bearer",
    headers: [],
    parser: {
      valueFormat: "money",
      valuePaths: [
        { path: "balance_infos[currency=CNY].total_balance", divisor: 1 },
        { path: "balance_infos[0].total_balance", divisor: 1 },
      ],
      currencyPaths: ["balance_infos[currency=CNY].currency", "balance_infos[0].currency"],
      currencyDefault: "CNY",
      resetPaths: [],
      windows: [],
      arrayWindows: [],
    },
    transformScript: deepseekTransformScript(),
    lowBalanceThreshold: 10,
    timeoutSeconds: 8,
  }),
  siliconflow: (): DataSourceSettings => ({
    id: "siliconflow",
    kind: "http",
    enabled: true,
    label: "SiliconFlow",
    baseUrl: "https://api.siliconflow.cn/v1",
    endpoint: "/user/info",
    apiKey: "",
    authMode: "bearer",
    headers: [],
    parser: {
      valueFormat: "money",
      valuePaths: [
        { path: "data.balance", divisor: 1 },
        { path: "balance", divisor: 1 },
      ],
      currencyPaths: ["data.currency", "currency"],
      currencyDefault: "CNY",
      resetPaths: [],
      windows: [],
      arrayWindows: [],
    },
    transformScript: siliconflowTransformScript(),
    lowBalanceThreshold: 10,
    timeoutSeconds: 8,
  }),
  glm: (): DataSourceSettings => ({
    id: "glm",
    kind: "http",
    enabled: true,
    label: "GLM",
    baseUrl: "https://bigmodel.cn",
    endpoint: "/api/monitor/usage/quota/limit",
    apiKey: "",
    authMode: "raw",
    headers: [],
    parser: {
      ...defaultParser(),
      valueFormat: "percent",
      currencyDefault: null,
    },
    transformScript: glmTransformScript(),
    lowBalanceThreshold: 10,
    timeoutSeconds: 8,
  }),
  custom: (): DataSourceSettings => ({
    id: "custom-source",
    kind: "http",
    enabled: true,
    label: "Custom Source",
    baseUrl: "",
    endpoint: "",
    apiKey: "",
    authMode: "bearer",
    headers: [],
    parser: {
      ...defaultParser(),
      valuePaths: [{ path: "data.balance", divisor: 1 }],
    },
    transformScript: customTransformScript(),
    lowBalanceThreshold: 10,
    timeoutSeconds: 8,
  }),
};

export const defaultSettings: MeterSettings = {
  language: "en",
  anchor: "right",
  offsets: {
    left: 0,
    right: 120,
    top: 0,
    bottom: 0,
  },
  selectedUsageWindowId: null,
  selectedMeterSource: "chatgpt",
  sources: [
    builtInSources.chatgpt(),
    builtInSources.aiMember(),
    builtInSources.kimi(),
    builtInSources.deepseek(),
    builtInSources.siliconflow(),
    builtInSources.glm(),
  ],
  taskbarSourceIds: ["chatgpt"],
  taskbarSources: {
    chatgpt: true,
    aiMember: false,
    kimi: false,
    deepseek: false,
  },
  taskbarScrollSeconds: 3.2,
  taskbarScrollAnimationSeconds: 0.35,
  queryRefreshSeconds: 60,
  taskbarAppearance: {
    textSizePx: 9,
    resetTextSizePx: 8,
    textColor: "#f6f8fb",
    progressColor: "#45d483",
  },
  aiMember: {
    enabled: false,
    label: "AI-MEMBER",
    baseUrl: "https://proxy.ai-member.icu",
    apiKey: "",
    balanceEndpoint: "/api/v1/auth/me?timezone=Asia%2FShanghai",
    lowBalanceThreshold: 10,
    balanceDivisor: 500000,
  },
  kimi: {
    enabled: false,
    label: "Kimi Code",
    baseUrl: "https://api.kimi.com/coding/v1",
    apiKey: "",
    usageEndpoint: "/usages",
    lowBalanceThreshold: 30,
  },
  deepseek: {
    enabled: false,
    label: "DeepSeek",
    baseUrl: "https://api.deepseek.com",
    apiKey: "",
    balanceEndpoint: "/user/balance",
    lowBalanceThreshold: 10,
  },
};

export function mergeSettings(settings: Partial<MeterSettings>): MeterSettings {
  const legacySources = migrateLegacySources(settings);
  const sources = withRequiredBuiltInSources(
    normalizeSources(settings.sources?.length ? settings.sources : legacySources),
  );
  const selectedMeterSource = sources.some((source) => source.id === settings.selectedMeterSource)
    ? settings.selectedMeterSource!
    : sources[0]?.id ?? "chatgpt";
  const taskbarSourceIds =
    settings.taskbarSourceIds?.filter((id) => sources.some((source) => source.id === id)) ??
    legacyTaskbarIds(settings);

  return {
    ...defaultSettings,
    ...settings,
    language: settings.language === "zh" ? "zh" : "en",
    selectedMeterSource,
    sources,
    taskbarSourceIds: taskbarSourceIds.length ? taskbarSourceIds : [selectedMeterSource],
    offsets: {
      ...defaultSettings.offsets,
      ...settings.offsets,
    },
    taskbarSources: {
      ...defaultSettings.taskbarSources,
      ...settings.taskbarSources,
    },
    taskbarScrollSeconds:
      Number.isFinite(settings.taskbarScrollSeconds) && settings.taskbarScrollSeconds! > 0
        ? settings.taskbarScrollSeconds!
        : defaultSettings.taskbarScrollSeconds,
    taskbarScrollAnimationSeconds:
      Number.isFinite(settings.taskbarScrollAnimationSeconds) &&
      settings.taskbarScrollAnimationSeconds! > 0
        ? settings.taskbarScrollAnimationSeconds!
        : defaultSettings.taskbarScrollAnimationSeconds,
    queryRefreshSeconds:
      Number.isFinite(settings.queryRefreshSeconds) && settings.queryRefreshSeconds! > 0
        ? settings.queryRefreshSeconds!
        : defaultSettings.queryRefreshSeconds,
    taskbarAppearance: {
      ...defaultSettings.taskbarAppearance,
      ...settings.taskbarAppearance,
      textSizePx:
        Number.isFinite(settings.taskbarAppearance?.textSizePx) &&
        settings.taskbarAppearance!.textSizePx > 0
          ? settings.taskbarAppearance!.textSizePx
          : defaultSettings.taskbarAppearance.textSizePx,
      resetTextSizePx:
        Number.isFinite(settings.taskbarAppearance?.resetTextSizePx) &&
        settings.taskbarAppearance!.resetTextSizePx > 0
          ? settings.taskbarAppearance!.resetTextSizePx
          : defaultSettings.taskbarAppearance.resetTextSizePx,
      textColor: settings.taskbarAppearance?.textColor || defaultSettings.taskbarAppearance.textColor,
      progressColor:
        settings.taskbarAppearance?.progressColor || defaultSettings.taskbarAppearance.progressColor,
    },
    aiMember: {
      ...defaultSettings.aiMember,
      ...settings.aiMember,
    },
    kimi: {
      ...defaultSettings.kimi,
      ...settings.kimi,
    },
    deepseek: {
      ...defaultSettings.deepseek,
      ...settings.deepseek,
    },
  };
}

export function uniqueSourceId(base: string, sources: DataSourceSettings[]) {
  const slug = base
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "") || "source";
  let next = slug;
  let index = 2;
  while (sources.some((source) => source.id === next)) {
    next = `${slug}-${index}`;
    index += 1;
  }
  return next;
}

function normalizeSources(sources: DataSourceSettings[]): DataSourceSettings[] {
  const seen = new Set<string>();
  return sources.filter((source) => source.id !== "chatgpt-local").map((source, index) => {
    const sourceKind =
      (source.kind as string) === "kimi"
        ? "http"
        : source.id === "chatgpt" && source.kind === "chatgpt"
          ? "http"
          : source.kind;
    const migratedSource =
      (source.kind as string) === "kimi"
        ? { ...builtInSources.kimi(), ...source }
        : source.id === "chatgpt" && source.kind === "chatgpt"
          ? { ...builtInSources.chatgpt(), apiKey: source.apiKey || builtInSources.chatgpt().apiKey }
          : source;
    if (
      migratedSource.id === "chatgpt" &&
      migratedSource.kind === "http" &&
      migratedSource.baseUrl.replace(/\/+$/, "") === "https://chatgpt.com/backend-api" &&
      migratedSource.endpoint === "/accounts/check/v4-2024-01-01"
    ) {
      migratedSource.endpoint = builtInSources.chatgpt().endpoint;
      migratedSource.headers = builtInSources.chatgpt().headers;
      migratedSource.transformScript = builtInSources.chatgpt().transformScript;
    }
    const parser = migratedSource.parser ?? defaultParser();
    const hasWindowMappings = Boolean(parser.windows?.length || parser.arrayWindows?.length);
    const fallback = source.label || source.id || `Source ${index + 1}`;
    const id = source.id && !seen.has(source.id) ? source.id : uniqueSourceId(fallback, sources.slice(0, index));
    const transformScript = shouldUseDefaultTransformScript(migratedSource)
      ? defaultTransformScriptForSource(migratedSource)
      : migratedSource.transformScript;
    seen.add(id);
    return {
      ...builtInSources.custom(),
      ...migratedSource,
      id,
      kind: sourceKind,
      enabled: true,
      label: source.label || fallback,
      authMode: isGlmPresetSource(migratedSource)
        ? "raw"
        : migratedSource.authMode === "raw"
          ? "raw"
          : "bearer",
      headers: source.headers ?? [],
      parser: {
        ...defaultParser(),
        ...parser,
        valuePaths: parser.valuePaths?.length
          ? parser.valuePaths
          : hasWindowMappings
            ? []
            : [{ path: "data.balance", divisor: 1 }],
        currencyPaths: parser.currencyPaths ?? [],
        resetPaths: parser.resetPaths ?? [],
        windows: parser.windows ?? [],
        arrayWindows: parser.arrayWindows ?? [],
      },
      transformScript,
      timeoutSeconds:
        Number.isFinite(source.timeoutSeconds) && source.timeoutSeconds > 0 ? source.timeoutSeconds : 8,
    };
  });
}

function withRequiredBuiltInSources(sources: DataSourceSettings[]) {
  const nextSources = [...sources];
  if (!nextSources.some((source) => source.id === "siliconflow")) {
    nextSources.push(builtInSources.siliconflow());
  }
  if (!nextSources.some((source) => source.id === "glm")) {
    nextSources.push(builtInSources.glm());
  }
  return nextSources;
}

function shouldUseDefaultTransformScript(source: DataSourceSettings) {
  const script = source.transformScript ?? "";
  if (!script.trim()) return true;
  return isKimiPresetSource(source) && isLegacyKimiTransformScript(script);
}

function defaultTransformScriptForSource(source: DataSourceSettings) {
  const id = source.id.toLowerCase();
  const label = source.label.toLowerCase();
  const baseUrl = source.baseUrl.toLowerCase();

  if (
    id === "chatgpt" ||
    id.startsWith("chatgpt-official") ||
    (label.startsWith("chatgpt") && baseUrl === "https://chatgpt.com/backend-api")
  ) {
    return builtInSources.chatgpt().transformScript;
  }

  if (
    id === "ai-member" ||
    id.startsWith("ai-member-") ||
    label.startsWith("ai-member") ||
    baseUrl === "https://proxy.ai-member.icu"
  ) {
    return builtInSources.aiMember().transformScript;
  }

  if (
    id === "deepseek" ||
    id.startsWith("deepseek-") ||
    label.startsWith("deepseek") ||
    baseUrl === "https://api.deepseek.com"
  ) {
    return builtInSources.deepseek().transformScript;
  }

  if (
    id === "siliconflow" ||
    id.startsWith("siliconflow-") ||
    label.startsWith("siliconflow") ||
    label.startsWith("硅基流动") ||
    baseUrl === "https://api.siliconflow.cn/v1"
  ) {
    return builtInSources.siliconflow().transformScript;
  }

  if (isGlmPresetSource(source)) {
    return builtInSources.glm().transformScript;
  }

  if (isKimiPresetSource(source)) {
    return builtInSources.kimi().transformScript;
  }

  if (id === "custom-source" || id.startsWith("custom-source-") || label.startsWith("custom source")) {
    return builtInSources.custom().transformScript;
  }

  return "";
}

function isKimiPresetSource(source: DataSourceSettings) {
  const id = source.id.toLowerCase();
  const label = source.label.toLowerCase();
  const endpoint = source.endpoint.toLowerCase();
  const baseUrl = source.baseUrl.toLowerCase();

  return (
    id === "kimi" ||
    id.startsWith("kimi-") ||
    label.startsWith("kimi code") ||
    baseUrl === "https://api.kimi.com/coding/v1" ||
    endpoint === "/usages"
  );
}

function isGlmPresetSource(source: DataSourceSettings) {
  const id = source.id.toLowerCase();
  const label = source.label.toLowerCase();
  const baseUrl = source.baseUrl.toLowerCase();

  return (
    id === "glm" ||
    id.startsWith("glm-") ||
    label.startsWith("glm") ||
    label.startsWith("智谱") ||
    baseUrl === "https://bigmodel.cn"
  );
}

function isLegacyKimiTransformScript(script: string) {
  return (
    script.includes("json.limits") &&
    script.includes("No Kimi usage windows found") &&
    script.includes("toWindow")
  );
}

function aiMemberTransformScript() {
  return `const candidates = [
  "data.balance",
  "data.user_usd_available",
  "data.total_usd_available",
  "balance",
  "usd",
  "cents"
];

function get(path) {
  return path.split(".").reduce((value, key) => value == null ? undefined : value[key], json);
}


let balance = null;
for (const path of candidates) {
  const value = get(path);
  if (value !== undefined && value !== null && value !== "") {
    balance = Number(value);
    if (path === "cents") balance = balance / 100;
    break;
  }
}

if (!Number.isFinite(balance)) throw new Error("No balance field found");
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));
const valueLabel = "$" + balance.toFixed(balance % 1 === 0 ? 0 : 2);

return {
  creditBalance: balance,
  message: source.label + " current value: " + valueLabel,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel,
    limitLabel: "HTTP transform",
    bucketId: source.id,
    windowKey: "balance"
  }]
};`;
}
function chatgptTransformScript() {
  return `function numberLike(value) {
  if (typeof value === "number") return value;
  if (typeof value === "string" && value.trim()) return Number(value);
  return NaN;
}

function timestamp(value) {
  if (value == null || value === "") return null;
  if (typeof value === "string" && Number.isNaN(Number(value))) return value;
  const raw = Number(value);
  if (!Number.isFinite(raw)) return null;
  const millis = raw > 10000000000 ? raw : raw * 1000;
  return new Date(millis).toISOString();
}

function durationLabel(window) {
  const minutes = numberLike(window.windowDurationMins)
    || numberLike(window.window_duration_mins)
    || numberLike(window.window_duration_ms) / 60000
    || numberLike(window.window_duration_seconds) / 60
    || numberLike(window.limit_window_seconds) / 60
    || numberLike(window.period_seconds) / 60;
  return Number.isFinite(minutes) && minutes > 0 ? Math.round(minutes) + " 分钟" : "用量窗口";
}

function windowPercent(window) {
  const direct = numberLike(window.usedPercent ?? window.used_percent);
  if (Number.isFinite(direct)) return Math.max(0, Math.min(100, Math.round(direct)));

  const limit = numberLike(window.limit);
  const remaining = numberLike(window.remaining);
  const used = Number.isFinite(numberLike(window.used)) ? numberLike(window.used) : limit - remaining;
  if (!Number.isFinite(limit) || limit <= 0 || !Number.isFinite(remaining)) return null;
  return Math.max(0, Math.min(100, Math.round((Math.max(0, used) / limit) * 100)));
}

function normalizeBucket(bucket, fallbackId) {
  const limitId = bucket.limitId || bucket.limit_id || bucket.id || fallbackId || "codex";
  const limitName = bucket.limitName || bucket.limit_name || bucket.name || "Codex";
  const items = [];

  for (const [key, label] of [["primary", "主用量窗口"], ["secondary", "次用量窗口"]]) {
    const window = bucket[key];
    if (!window) continue;
    const usedPercent = windowPercent(window);
    if (usedPercent == null) continue;
    items.push({
      id: "codex-" + limitId + "-" + key,
      label,
      usedPercent,
      remainingPercent: 100 - usedPercent,
      resetsAt: timestamp(window.resetsAt ?? window.resets_at ?? window.resetTime ?? window.reset_time ?? window.expiresAt ?? window.expires_at),
      limitLabel: limitName + " / " + durationLabel(window),
      bucketId: limitId,
      windowKey: key
    });
  }

  if (!items.length) {
    const usedPercent = windowPercent(bucket);
    if (usedPercent != null) {
      items.push({
        id: "codex-" + limitId + "-primary",
        label: "主用量窗口",
        usedPercent,
        remainingPercent: 100 - usedPercent,
        resetsAt: timestamp(bucket.resetsAt ?? bucket.resets_at ?? bucket.resetTime ?? bucket.reset_time ?? bucket.expiresAt ?? bucket.expires_at),
        limitLabel: limitName + " / " + durationLabel(bucket),
        bucketId: limitId,
        windowKey: "primary"
      });
    }
  }

  return items;
}

function normalizeWhamUsage() {
  const rateLimit = json.rate_limit;
  if (!rateLimit || typeof rateLimit !== "object") return [];

  const bucket = {
    limitId: "codex",
    limitName: "Codex",
    primary: rateLimit.primary_window,
    secondary: rateLimit.secondary_window
  };

  return normalizeBucket(bucket, "codex").map((window) => ({
    ...window,
    resetsAt: window.resetsAt || timestamp(
      (window.windowKey === "primary" ? rateLimit.primary_window : rateLimit.secondary_window)?.reset_at
    )
  }));
}

function collectBuckets() {
  const whamWindows = normalizeWhamUsage();
  if (whamWindows.length) return whamWindows;

  const directMap = json.rateLimitsByLimitId || json.rate_limits_by_limit_id;
  if (directMap && typeof directMap === "object" && !Array.isArray(directMap)) {
    return Object.entries(directMap).flatMap(([id, bucket]) => normalizeBucket(bucket, id));
  }

  for (const key of ["rate_limits", "rateLimits", "limits"]) {
    const value = json[key];
    if (Array.isArray(value)) {
      return value.flatMap((bucket, index) => normalizeBucket(bucket, bucket.limit_id || bucket.limitId || bucket.id || "limit-" + index));
    }
    if (value && typeof value === "object") {
      return Object.entries(value).flatMap(([id, bucket]) => normalizeBucket(bucket, id));
    }
  }

  return Object.entries(json)
    .filter(([, value]) => value && typeof value === "object")
    .flatMap(([id, bucket]) => normalizeBucket(bucket, id));
}

const windows = collectBuckets();
if (!windows.length) throw new Error("No ChatGPT Codex usage windows found");

const account = json.account || json.user || {};
const plan = json.account_plan || json.subscription || {};

return {
  accountLabel: account.email || json.email || "ChatGPT",
  planLabel: plan.plan_type || plan.name || account.planType || account.plan_type || json.plan_type || null,
  message: "ChatGPT 官方 HTTP 用量已更新",
  windows
};`;
}

function deepseekTransformScript() {
  return `const infos = Array.isArray(json.balance_infos) ? json.balance_infos : [];
const selected = infos.find((item) => item.currency === "CNY") || infos[0];
if (!selected) throw new Error("No balance_infos item found");

const balance = Number(selected.total_balance);
if (!Number.isFinite(balance)) throw new Error("No total_balance field found");

const currency = selected.currency || "CNY";
const prefix = currency === "USD" ? "$" : currency + " ";
const valueLabel = prefix + balance.toFixed(balance % 1 === 0 ? 0 : 2);
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  creditBalance: balance,
  message: source.label + " current value: " + valueLabel,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel,
    limitLabel: "HTTP transform",
    bucketId: source.id,
    windowKey: "balance"
  }]
};`;
}

function siliconflowTransformScript() {
  return `const balance = Number(
  json.data?.balance ??
  json.data?.totalBalance ??
  json.data?.total_balance ??
  json.balance ??
  json.totalBalance ??
  json.total_balance
);

if (!Number.isFinite(balance)) throw new Error("No SiliconFlow balance field found");

const currency = json.data?.currency || json.currency || "CNY";
const prefix = currency === "USD" ? "$" : currency === "CNY" ? "CNY " : currency + " ";
const valueLabel = prefix + balance.toFixed(balance % 1 === 0 ? 0 : 2);
const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  accountLabel: json.data?.name || json.data?.email || source.label,
  planLabel: json.data?.status || source.baseUrl,
  creditBalance: balance,
  message: source.label + " current balance: " + valueLabel,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel,
    limitLabel: "SiliconFlow user info",
    bucketId: source.id,
    windowKey: "balance"
  }]
};`;
}

function glmTransformScript() {
  return `function numberLike(value) {
  if (typeof value === "number") return value;
  if (typeof value === "string" && value.trim()) return Number(value);
  return NaN;
}

function timestamp(value) {
  if (value == null || value === "") return null;
  const raw = Number(value);
  if (!Number.isFinite(raw)) return typeof value === "string" ? value : null;
  return new Date(raw > 10000000000 ? raw : raw * 1000).toISOString();
}

const limits = Array.isArray(json.data?.limits) ? json.data.limits : [];
if (!limits.length) throw new Error("No GLM quota limits found");

const labels = {
  TOKENS_LIMIT: "Token quota",
  TIME_LIMIT: "Monthly quota"
};

const windows = limits.map((item, index) => {
  const limit = numberLike(item.usage ?? item.limit ?? item.total);
  const current = numberLike(item.currentValue ?? item.current_value ?? item.used);
  const directPercent = numberLike(item.percentage ?? item.usedPercent ?? item.used_percent);
  const usedPercent = Number.isFinite(directPercent)
    ? Math.max(0, Math.min(100, Math.round(directPercent)))
    : Number.isFinite(limit) && limit > 0 && Number.isFinite(current)
      ? Math.max(0, Math.min(100, Math.round((current / limit) * 100)))
      : 0;
  const type = String(item.type || "quota-" + index);
  return {
    id: source.id + "-" + type.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
    label: labels[type] || type,
    usedPercent,
    remainingPercent: 100 - usedPercent,
    valueLabel: String(100 - usedPercent) + "%",
    resetsAt: timestamp(item.nextResetTime ?? item.next_reset_time ?? item.resetTime ?? item.reset_time),
    limitLabel: "GLM quota",
    bucketId: source.id,
    windowKey: type.toLowerCase()
  };
});

return {
  accountLabel: source.label,
  planLabel: source.baseUrl,
  message: source.label + " quota updated",
  windows
};`;
}

function kimiTransformScript() {
  return `const weekly = json.usage;
if (!weekly) throw new Error("No weekly usage found");

const limit = Number(weekly.limit);
const remaining = Number(weekly.remaining);
if (!Number.isFinite(limit) || !Number.isFinite(remaining)) {
  throw new Error("Weekly usage is missing limit or remaining");
}

const used = Number.isFinite(Number(weekly.used)) ? Number(weekly.used) : Math.max(0, limit - remaining);
const remainingPercent = limit > 0 ? Math.max(0, Math.min(100, Math.round(remaining / limit * 100))) : 0;
const usedPercent = limit > 0 ? Math.max(0, Math.min(100, Math.round(used / limit * 100))) : 100 - remainingPercent;

return {
  message: source.label + " 周额度: " + remainingPercent + "%",
  windows: [{
    id: source.id + "-weekly",
    label: "周额度",
    usedPercent,
    remainingPercent,
    valueLabel: remainingPercent + "%",
    resetsAt: weekly.resetTime || null,
    limitLabel: "Kimi Code 周额度",
    bucketId: source.id,
    windowKey: "weekly"
  }]
};`;
}

function customTransformScript() {
  return `const balance = Number(json.data?.balance ?? json.balance);
if (!Number.isFinite(balance)) throw new Error("No balance field found");

const threshold = source.lowBalanceThreshold || 10;
const remainingPercent = Math.max(0, Math.min(100, Math.round(balance / threshold * 100)));

return {
  creditBalance: balance,
  windows: [{
    id: source.id + "-balance",
    label: source.label,
    usedPercent: 100 - remainingPercent,
    remainingPercent,
    valueLabel: String(balance),
    bucketId: source.id,
    windowKey: "balance"
  }]
};`;
}

function migrateLegacySources(settings: Partial<MeterSettings>): DataSourceSettings[] {
  const sources = defaultSettings.sources.map((source) => ({
    ...source,
    headers: source.headers.map((header) => ({ ...header })),
    parser: {
      ...source.parser,
      valuePaths: source.parser.valuePaths.map((path) => ({ ...path })),
      currencyPaths: [...source.parser.currencyPaths],
      resetPaths: [...source.parser.resetPaths],
      windows: source.parser.windows.map((window) => ({ ...window })),
      arrayWindows: source.parser.arrayWindows.map((window) => ({ ...window })),
    },
  }));

  if (settings.aiMember) {
    sources[1] = {
      ...builtInSources.aiMember(),
      enabled: settings.aiMember.enabled,
      label: settings.aiMember.label,
      baseUrl: settings.aiMember.baseUrl,
      endpoint: settings.aiMember.balanceEndpoint,
      apiKey: settings.aiMember.apiKey,
      lowBalanceThreshold: settings.aiMember.lowBalanceThreshold,
    };
  }

  if (settings.kimi) {
    sources[2] = {
      ...builtInSources.kimi(),
      enabled: settings.kimi.enabled,
      label: settings.kimi.label,
      baseUrl: settings.kimi.baseUrl,
      endpoint: settings.kimi.usageEndpoint,
      apiKey: settings.kimi.apiKey,
      lowBalanceThreshold: settings.kimi.lowBalanceThreshold,
    };
  }

  if (settings.deepseek) {
    sources[3] = {
      ...builtInSources.deepseek(),
      enabled: settings.deepseek.enabled,
      label: settings.deepseek.label,
      baseUrl: settings.deepseek.baseUrl,
      endpoint: settings.deepseek.balanceEndpoint,
      apiKey: settings.deepseek.apiKey,
      lowBalanceThreshold: settings.deepseek.lowBalanceThreshold,
    };
  }

  return sources;
}

function legacyTaskbarIds(settings: Partial<MeterSettings>) {
  const ids: string[] = [];
  if (settings.taskbarSources?.chatgpt ?? true) ids.push("chatgpt");
  if (settings.taskbarSources?.aiMember) ids.push("ai-member");
  if (settings.taskbarSources?.kimi) ids.push("kimi");
  if (settings.taskbarSources?.deepseek) ids.push("deepseek");
  return ids;
}

function defaultParser(): HttpParserSettings {
  return {
    valuePaths: [],
    currencyPaths: [],
    currencyDefault: "USD",
    resetPaths: [],
    valueFormat: "money",
    windows: [],
    arrayWindows: [],
  };
}
