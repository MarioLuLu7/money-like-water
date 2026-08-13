export type ProviderUsageStatus =
  | "ok"
  | "codex_missing"
  | "logged_out"
  | "unauthorized"
  | "unavailable"
  | "error";

export type UsageWindow = {
  id: string;
  label: string;
  usedPercent: number;
  remainingPercent: number;
  valueLabel?: string;
  resetsAt?: string;
  limitLabel?: string;
  bucketId?: string;
  windowKey?: string;
};

export type MeterDisplayItem = {
  id: string;
  label: string;
  remainingPercent: number;
  valueLabel?: string | null;
  resetLabel?: string | null;
  status: ProviderUsageStatus;
};

export type UsageDiagnosticBucket = {
  id: string;
  name?: string;
  primaryWindowId?: string;
  secondaryWindowId?: string;
};

export type UsageDiagnostics = {
  codexPath?: string;
  codexHome?: string;
  selectedWindowId?: string;
  buckets: UsageDiagnosticBucket[];
  rawAccountKind?: string;
  requiresOpenaiAuth: boolean;
};

export type ProviderUsage = {
  provider: string;
  accountLabel?: string;
  planLabel?: string;
  creditBalance?: number;
  status: ProviderUsageStatus;
  windows: UsageWindow[];
  meterItems: MeterDisplayItem[];
  updatedAt?: string;
  message?: string;
  diagnostics?: UsageDiagnostics;
};
