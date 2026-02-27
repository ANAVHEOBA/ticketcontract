import { request } from "./http";

export const health = () =>
  request<{ status: string; service: string; env?: string }>("/health");

export const readiness = () =>
  request<{ status: string; checks?: Record<string, unknown> }>("/ready");
