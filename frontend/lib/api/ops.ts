import { request } from "./http";

export const getOpsMetrics = (token: string) =>
  request<{ metrics: Record<string, unknown> }>("/ops/metrics", { token });

export const getOpsAlerts = (token: string) =>
  request<{ alerts: unknown[] }>("/ops/alerts", { token });

export const getAuditLogs = (token: string, limit = 100) =>
  request<{ logs: unknown[] }>(`/ops/audit-logs?limit=${limit}`, { token });
