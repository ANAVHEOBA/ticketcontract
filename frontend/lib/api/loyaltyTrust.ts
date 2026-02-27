import { request } from "./http";
import { postTx } from "./onchain";
import type { TxRequestBase } from "./types";

type LoyaltyTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  wallet: string;
};

type TrustSignalTx = LoyaltyTx & { signal_id?: string };

export type LoyaltyTrustAction = {
  action: string;
  organizer_id?: string;
  event_id?: string;
  wallet?: string;
  signal_id?: string;
  signature: string;
  confirmation_status?: string;
};

export const accruePoints = (token: string, payload: LoyaltyTx) =>
  postTx<LoyaltyTrustAction>("/loyalty/accrue", token, payload);

export const redeemPoints = (token: string, payload: LoyaltyTx) =>
  postTx<LoyaltyTrustAction>("/loyalty/redeem", token, payload);

export const recordPurchaseSignal = (token: string, payload: TrustSignalTx) =>
  postTx<LoyaltyTrustAction>("/trust/purchase", token, payload);

export const recordAttendanceSignal = (token: string, payload: TrustSignalTx) =>
  postTx<LoyaltyTrustAction>("/trust/attendance", token, payload);

export const flagTrustAbuse = (token: string, payload: TrustSignalTx) =>
  postTx<LoyaltyTrustAction>("/trust/abuse", token, payload);

export const setTrustSchemaVersion = (
  token: string,
  payload: TxRequestBase & { organizer_id?: string; schema_version: number }
) => postTx<LoyaltyTrustAction>("/trust/schema-version", token, payload);

export const getLoyalty = (
  token: string,
  query: { wallet: string; organizer_id?: string }
) => {
  const params = new URLSearchParams({ wallet: query.wallet });
  if (query.organizer_id) params.set("organizer_id", query.organizer_id);
  return request<{ rows: unknown[] }>(`/loyalty?${params.toString()}`, { token });
};

export const getTrustSignals = (
  token: string,
  query?: {
    wallet?: string;
    organizer_id?: string;
    event_id?: string;
    limit?: number;
  }
) => {
  const params = new URLSearchParams();
  if (query?.wallet) params.set("wallet", query.wallet);
  if (query?.organizer_id) params.set("organizer_id", query.organizer_id);
  if (query?.event_id) params.set("event_id", query.event_id);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<{ rows: unknown[] }>(`/trust/signals${suffix}`, { token });
};
