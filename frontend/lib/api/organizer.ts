import { request } from "./http";
import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type OrganizerTx = TxRequestBase & { organizer_id: string };
type OrganizerAction = {
  action: string;
  organizer_id: string;
  signature: string;
  confirmation_status?: string;
};
type OrganizerSim = SimResponseBase & { organizer_id: string };

export const createOrganizer = (token: string, payload: OrganizerTx) =>
  postTx<OrganizerAction>("/organizers", token, payload);
export const simulateCreateOrganizer = (token: string, payload: OrganizerTx) =>
  postSim<OrganizerSim>("/organizers/simulate", token, payload);

export const updateOrganizer = (token: string, payload: OrganizerTx) =>
  postTx<OrganizerAction>("/organizers/update", token, payload);
export const simulateUpdateOrganizer = (token: string, payload: OrganizerTx) =>
  postSim<OrganizerSim>("/organizers/update/simulate", token, payload);

export const setOrganizerStatus = (token: string, payload: OrganizerTx) =>
  postTx<OrganizerAction>("/organizers/status", token, payload);
export const simulateSetOrganizerStatus = (token: string, payload: OrganizerTx) =>
  postSim<OrganizerSim>("/organizers/status/simulate", token, payload);

export const setOrganizerComplianceFlags = (token: string, payload: OrganizerTx) =>
  postTx<OrganizerAction>("/organizers/compliance-flags", token, payload);
export const simulateSetOrganizerComplianceFlags = (
  token: string,
  payload: OrganizerTx
) => postSim<OrganizerSim>("/organizers/compliance-flags/simulate", token, payload);

export const setOrganizerOperator = (token: string, payload: OrganizerTx) =>
  postTx<OrganizerAction>("/organizers/operators", token, payload);
export const simulateSetOrganizerOperator = (token: string, payload: OrganizerTx) =>
  postSim<OrganizerSim>("/organizers/operators/simulate", token, payload);

export const getOrganizer = (token: string, organizerId: string) =>
  request<{ organizer: unknown }>(`/organizers/${organizerId}`, { token });
