import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type PolicyTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id?: string;
};

type PolicyAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  class_id?: string;
  signature: string;
  confirmation_status?: string;
};

type PolicySim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  class_id?: string;
};

export const setResalePolicy = (token: string, payload: PolicyTx) =>
  postTx<PolicyAction>("/resale-policy", token, payload);
export const simulateSetResalePolicy = (token: string, payload: PolicyTx) =>
  postSim<PolicySim>("/resale-policy/simulate", token, payload);

export const getResalePolicy = (
  token: string,
  query: { event_id: string; class_id?: string }
) => {
  const params = new URLSearchParams({ event_id: query.event_id });
  if (query.class_id) params.set("class_id", query.class_id);
  return request<{ policy: unknown }>(`/resale-policy?${params.toString()}`, { token });
};

export const getResalePolicyById = (token: string, policyId: string) =>
  request<{ policy: unknown }>(`/resale-policy/${policyId}`, { token });

export const validatePolicy = (
  token: string,
  payload: {
    max_markup_bps: number;
    royalty_bps: number;
    whitelist_enabled?: boolean;
    blacklist_enabled?: boolean;
  }
) => request<{ valid: boolean; reasons: string[] }>("/resale-policy/validate", {
  method: "POST",
  token,
  body: payload,
});

export const writePolicyRecommendation = (token: string, payload: Record<string, unknown>) =>
  request<{ saved: boolean; recommendation: unknown }>("/resale-policy/recommendation", {
    method: "POST",
    token,
    body: payload,
  });
