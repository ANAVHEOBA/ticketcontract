import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type SecondaryTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
  ticket_id: string;
  listing_id?: string;
};

type SecondaryAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  class_id: string;
  ticket_id: string;
  listing_id?: string;
  signature: string;
  confirmation_status?: string;
};

type SecondarySim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
  ticket_id: string;
  listing_id?: string;
};

export const listTicket = (token: string, payload: SecondaryTx) =>
  postTx<SecondaryAction>("/secondary-sale/list", token, payload);
export const simulateListTicket = (token: string, payload: SecondaryTx) =>
  postSim<SecondarySim>("/secondary-sale/list/simulate", token, payload);

export const buyResaleTicket = (token: string, payload: SecondaryTx) =>
  postTx<SecondaryAction>("/secondary-sale/buy", token, payload);
export const simulateBuyResaleTicket = (token: string, payload: SecondaryTx) =>
  postSim<SecondarySim>("/secondary-sale/buy/simulate", token, payload);

export const cancelListing = (token: string, payload: SecondaryTx) =>
  postTx<SecondaryAction>("/secondary-sale/cancel", token, payload);
export const simulateCancelListing = (token: string, payload: SecondaryTx) =>
  postSim<SecondarySim>("/secondary-sale/cancel/simulate", token, payload);

export const expireListing = (token: string, payload: SecondaryTx) =>
  postTx<SecondaryAction>("/secondary-sale/expire", token, payload);
export const simulateExpireListing = (token: string, payload: SecondaryTx) =>
  postSim<SecondarySim>("/secondary-sale/expire/simulate", token, payload);

export const getListing = (token: string, listingId: string) =>
  request<{ listing: unknown }>(`/secondary-sale/listings/${listingId}`, { token });
