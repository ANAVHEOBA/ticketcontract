import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type FinancingTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  offer_id?: string;
};

type FinancingAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  offer_id?: string;
  signature: string;
  confirmation_status?: string;
};

type FinancingSim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  offer_id?: string;
};

export const createFinancingOffer = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/offers", token, payload);
export const simulateCreateFinancingOffer = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/offers/simulate", token, payload);

export const acceptFinancingOffer = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/offers/accept", token, payload);
export const simulateAcceptFinancingOffer = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/offers/accept/simulate", token, payload);

export const rejectFinancingOffer = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/offers/reject", token, payload);
export const simulateRejectFinancingOffer = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/offers/reject/simulate", token, payload);

export const disburseAdvance = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/disburse", token, payload);
export const simulateDisburseAdvance = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/disburse/simulate", token, payload);

export const clawbackDisbursement = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/clawback", token, payload);
export const simulateClawbackDisbursement = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/clawback/simulate", token, payload);

export const setFinancingFreeze = (token: string, payload: FinancingTx) =>
  postTx<FinancingAction>("/financing/freeze", token, payload);
export const simulateSetFinancingFreeze = (token: string, payload: FinancingTx) =>
  postSim<FinancingSim>("/financing/freeze/simulate", token, payload);

export const getFinancingOffer = (token: string, offerId: string) =>
  request<{ offer: unknown }>(`/financing/offers/${offerId}`, { token });
