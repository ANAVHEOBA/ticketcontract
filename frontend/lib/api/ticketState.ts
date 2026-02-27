import { ApiError, request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type TicketTx = TxRequestBase & { organizer_id: string; ticket_id: string };

type TicketAction = {
  action: string;
  organizer_id: string;
  ticket_id: string;
  signature: string;
  confirmation_status?: string;
};

type TicketSim = SimResponseBase & { organizer_id: string; ticket_id: string };

export const getTicket = (token: string, ticketId: string) =>
  request<{ ticket: unknown }>(`/tickets/${ticketId}`, { token });

export async function updateTicketMetadata(token: string, payload: TicketTx) {
  try {
    return await postTx<TicketAction>("/tickets/metadata", token, payload);
  } catch (error) {
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return postTx<TicketAction>("/ticket-state/metadata", token, payload);
    }
    throw error;
  }
}

export async function simulateUpdateTicketMetadata(token: string, payload: TicketTx) {
  try {
    return await postSim<TicketSim>("/tickets/metadata/simulate", token, payload);
  } catch (error) {
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return postSim<TicketSim>("/ticket-state/metadata/simulate", token, payload);
    }
    throw error;
  }
}

export const transitionTicketStatus = (
  token: string,
  payload: TicketTx & { target_status: string }
) =>
  postTx<TicketAction>("/tickets/status", token, payload).catch((error) => {
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return postTx<TicketAction>("/ticket-state/transition", token, payload);
    }
    throw error;
  });

export const simulateTransitionTicketStatus = (
  token: string,
  payload: TicketTx & { target_status: string }
) =>
  postSim<TicketSim>("/tickets/status/simulate", token, payload).catch((error) => {
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return postSim<TicketSim>("/ticket-state/transition/simulate", token, payload);
    }
    throw error;
  });
