import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type ClassTx = TxRequestBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
};
type ClassAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  class_id: string;
  signature: string;
  confirmation_status?: string;
};
type ClassSim = SimResponseBase & {
  organizer_id: string;
  event_id: string;
  class_id: string;
};

export const createTicketClass = (token: string, payload: ClassTx) =>
  postTx<ClassAction>("/ticket-classes", token, payload);
export const simulateCreateTicketClass = (token: string, payload: ClassTx) =>
  postSim<ClassSim>("/ticket-classes/simulate", token, payload);

export const updateTicketClass = (token: string, payload: ClassTx) =>
  postTx<ClassAction>("/ticket-classes/update", token, payload);
export const simulateUpdateTicketClass = (token: string, payload: ClassTx) =>
  postSim<ClassSim>("/ticket-classes/update/simulate", token, payload);

export const reserveInventory = (token: string, payload: ClassTx) =>
  postTx<ClassAction>("/ticket-classes/reserve", token, payload);
export const simulateReserveInventory = (token: string, payload: ClassTx) =>
  postSim<ClassSim>("/ticket-classes/reserve/simulate", token, payload);

export const getTicketClass = (token: string, classId: string) =>
  request<{ class: unknown }>(`/ticket-classes/${classId}`, { token });

export const getTicketClassAnalytics = (token: string, classId: string) =>
  request<{ analytics: unknown }>(`/ticket-classes/${classId}/analytics`, { token });

export const listTicketClasses = (
  token: string,
  query?: { organizer_id?: string; event_id?: string },
) => {
  const params = new URLSearchParams();
  if (query?.organizer_id) params.set("organizer_id", query.organizer_id);
  if (query?.event_id) params.set("event_id", query.event_id);
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<{ classes: unknown[] }>(`/ticket-classes${suffix}`, { token });
};
