import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type SettlementTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  settlement_ref: string;
};

type SettlementAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  settlement_ref: string;
  signature: string;
  confirmation_status?: string;
  idempotent_replay: boolean;
};

type SettlementSim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  settlement_ref: string;
};

export const settlePrimaryRevenue = (token: string, payload: SettlementTx) =>
  postTx<SettlementAction>("/settlement/primary", token, payload);
export const simulateSettlePrimaryRevenue = (token: string, payload: SettlementTx) =>
  postSim<SettlementSim>("/settlement/primary/simulate", token, payload);

export const settleResaleRevenue = (token: string, payload: SettlementTx) =>
  postTx<SettlementAction>("/settlement/resale", token, payload);
export const simulateSettleResaleRevenue = (token: string, payload: SettlementTx) =>
  postSim<SettlementSim>("/settlement/resale/simulate", token, payload);

export const finalizeSettlement = (token: string, payload: SettlementTx) =>
  postTx<SettlementAction>("/settlement/finalize", token, payload);
export const simulateFinalizeSettlement = (token: string, payload: SettlementTx) =>
  postSim<SettlementSim>("/settlement/finalize/simulate", token, payload);

export const getSettlement = (token: string, settlementRef: string) =>
  request<{ settlement: unknown }>(`/settlement/${settlementRef}`, { token });
