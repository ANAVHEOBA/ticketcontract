import { request } from "./http";
import type {
  ActionResponseBase,
  SimRequestBase,
  SimResponseBase,
  TxRequestBase,
} from "./types";

type OnchainPayload = Record<string, unknown>;

export function postTx<T extends ActionResponseBase>(
  path: string,
  token: string,
  payload: OnchainPayload & TxRequestBase
) {
  return request<T>(path, { method: "POST", token, body: payload });
}

export function postSim<T extends SimResponseBase>(
  path: string,
  token: string,
  payload: OnchainPayload & SimRequestBase
) {
  return request<T>(path, { method: "POST", token, body: payload });
}
