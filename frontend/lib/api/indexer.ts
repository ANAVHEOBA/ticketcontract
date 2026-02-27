import { request } from "./http";

export const getIndexerStatus = (token: string) =>
  request<{
    enabled: boolean;
    running: boolean;
    last_poll_epoch: number;
    last_processed_slot: number;
    last_signature?: string;
    backfill_active: boolean;
    backfill_pending: number;
  }>("/indexer/status", { token });

export const runIndexerBackfill = (token: string, payload: { start_slot: number; end_slot: number }) =>
  request<{ queued: boolean }>("/indexer/backfill", {
    method: "POST",
    token,
    body: payload,
  });

export const refreshKpis = (token: string) =>
  request<{ refreshed: boolean }>("/indexer/kpis/refresh", {
    method: "POST",
    token,
  });

export const getEventSalesKpi = (token: string, eventId: string) =>
  request<Record<string, unknown>>(`/kpis/event-sales/${eventId}`, { token });

export const getEventSalesKpiQuery = (
  token: string,
  query: { event_id?: string; organizer_id?: string }
) => {
  const params = new URLSearchParams();
  if (query.event_id) params.set("event_id", query.event_id);
  if (query.organizer_id) params.set("organizer_id", query.organizer_id);
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<Record<string, unknown>>(`/kpis/event-sales${suffix}`, { token });
};

export const getResaleHealthKpi = (token: string, eventId: string) =>
  request<Record<string, unknown>>(`/kpis/resale-health/${eventId}`, { token });

export const getFanQualityKpi = (
  token: string,
  query: { event_id?: string; organizer_id?: string; wallet?: string }
) => {
  const params = new URLSearchParams();
  if (query.event_id) params.set("event_id", query.event_id);
  if (query.organizer_id) params.set("organizer_id", query.organizer_id);
  if (query.wallet) params.set("wallet", query.wallet);
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<Record<string, unknown>>(`/kpis/fan-quality${suffix}`, { token });
};

export const getFinancingCashKpi = (
  token: string,
  query: { organizer_id: string; event_id: string }
) =>
  request<Record<string, unknown>>(
    `/kpis/financing-cash?organizer_id=${encodeURIComponent(query.organizer_id)}&event_id=${encodeURIComponent(query.event_id)}`,
    { token }
  );
