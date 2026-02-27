import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type EventTx = TxRequestBase & { organizer_id: string; event_id: string };
type EventAction = {
  action: string;
  organizer_id: string;
  event_id: string;
  signature: string;
  confirmation_status?: string;
};

type EventSim = SimResponseBase & { organizer_id: string; event_id: string };

export const createEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events", token, payload);
export const simulateCreateEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/simulate", token, payload);

export const updateEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/update", token, payload);
export const simulateUpdateEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/update/simulate", token, payload);

export const freezeEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/freeze", token, payload);
export const simulateFreezeEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/freeze/simulate", token, payload);

export const cancelEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/cancel", token, payload);
export const simulateCancelEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/cancel/simulate", token, payload);

export const pauseEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/pause", token, payload);
export const simulatePauseEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/pause/simulate", token, payload);

export const closeEvent = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/close", token, payload);
export const simulateCloseEvent = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/close/simulate", token, payload);

export const setEventRestrictions = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/restrictions", token, payload);
export const simulateSetEventRestrictions = (token: string, payload: EventTx) =>
  postSim<EventSim>("/events/restrictions/simulate", token, payload);

export const setEventLoyaltyMultiplier = (token: string, payload: EventTx) =>
  postTx<EventAction>("/events/loyalty-multiplier", token, payload);
export const simulateSetEventLoyaltyMultiplier = (
  token: string,
  payload: EventTx
) => postSim<EventSim>("/events/loyalty-multiplier/simulate", token, payload);

export const listEvents = (
  token: string,
  query?: { organizer_id?: string; status?: string }
) => {
  const params = new URLSearchParams();
  if (query?.organizer_id) params.set("organizer_id", query.organizer_id);
  if (query?.status) params.set("status", query.status);
  const suffix = params.toString() ? `?${params.toString()}` : "";
  return request<{ events: unknown[] }>(`/events${suffix}`, { token });
};

export const getEvent = (token: string, eventId: string) =>
  request<{ event: unknown }>(`/events/${eventId}`, { token });
