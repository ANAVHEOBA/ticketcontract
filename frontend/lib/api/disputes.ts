import { request } from "./http";
import { postTx } from "./onchain";
import type { TxRequestBase } from "./types";

type DisputeTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  ticket_id: string;
  dispute_id?: string;
};

type DisputeAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  ticket_id: string;
  dispute_id?: string;
  signature: string;
  confirmation_status?: string;
};

export const refundTicket = (token: string, payload: DisputeTx) =>
  postTx<DisputeAction>("/disputes/refund", token, payload);

export const flagDispute = (token: string, payload: DisputeTx) =>
  postTx<DisputeAction>("/disputes/flag", token, payload);

export const flagChargeback = (token: string, payload: DisputeTx) =>
  postTx<DisputeAction>("/disputes/chargeback", token, payload);

export const getDisputeQueue = (
  token: string,
  query?: { organizer_id?: string; status?: string; limit?: number }
) => {
  const params = new URLSearchParams();
  if (query?.organizer_id) params.set("organizer_id", query.organizer_id);
  if (query?.status) params.set("status", query.status);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<{ items: unknown[] }>(`/disputes/queue${suffix}`, { token });
};
