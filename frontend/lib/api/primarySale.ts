import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type PrimaryTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
  buyer_wallet?: string;
  ticket_pda?: string;
  gross_amount?: number;
  protocol_fee_amount?: number;
  net_amount?: number;
};

type PrimaryAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  class_id: string;
  confirmation_status?: string;
  receipt: {
    signature: string;
    ticket_pda?: string;
    buyer_wallet?: string;
    gross_amount?: number;
    protocol_fee_amount?: number;
    net_amount?: number;
  };
};

type PrimarySim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
};

export const buyTicket = (token: string, payload: PrimaryTx) =>
  postTx<PrimaryAction>("/primary-sale/buy", token, payload);
export const simulateBuyTicket = (token: string, payload: PrimaryTx) =>
  postSim<PrimarySim>("/primary-sale/buy/simulate", token, payload);

export const issueCompTicket = (token: string, payload: PrimaryTx) =>
  postTx<PrimaryAction>("/primary-sale/comp", token, payload);
export const simulateIssueCompTicket = (token: string, payload: PrimaryTx) =>
  postSim<PrimarySim>("/primary-sale/comp/simulate", token, payload);
