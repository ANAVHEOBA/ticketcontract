import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type CheckinPolicyTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
};

type CheckinTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
  gate_id: string;
  ticket_id: string;
  scanner_id: string;
};

type CheckinAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  class_id: string;
  signature: string;
  confirmation_status?: string;
};

export const setCheckinPolicy = (token: string, payload: CheckinPolicyTx) =>
  postTx<CheckinAction>("/checkin/policy", token, payload);

export const checkinTicket = (token: string, payload: CheckinTx) =>
  postTx<
    CheckinAction & {
      gate_payload: {
        gate_id: string;
        ticket_id: string;
        scanner_id: string;
        accepted: boolean;
        reason?: string;
        checked_in_at_epoch: number;
      };
    }
  >("/checkin/ticket", token, payload);

export const simulateCheckinTicket = (token: string, payload: CheckinTx) =>
  postSim<SimResponseBase & { organizer_id: string; event_id: string; class_id: string }>(
    "/checkin/ticket/simulate",
    token,
    payload
  );

export const sendGateResponse = (
  token: string,
  payload: {
    gate_id: string;
    ticket_id: string;
    scanner_id?: string;
    accepted: boolean;
    reason?: string;
    checked_in_at_epoch?: number;
  }
) =>
  request<{ ok: boolean; gate_payload?: unknown }>("/checkin/gate-response", {
    method: "POST",
    token,
    body: payload,
  });
