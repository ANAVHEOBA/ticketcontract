import { request } from "./http";

export function getChainContext(token: string) {
  return request<{
    cluster: string;
    rpc_url: string;
    commitment: string;
    program_id: string;
    anchor_idl_address?: string;
    idl_loaded: boolean;
  }>("/chain/context", { token });
}

export function derivePda(
  token: string,
  payload: { seeds: { encoding: "utf8" | "hex" | "base58"; value: string }[] }
) {
  return request<{ pda: string; bump: number }>("/chain/pda/derive", {
    method: "POST",
    token,
    body: payload,
  });
}

export function simulateTx(token: string, payload: Record<string, unknown>) {
  return request<{ err?: unknown; logs: string[]; units_consumed?: number }>(
    "/chain/tx/simulate",
    {
      method: "POST",
      token,
      body: payload,
    }
  );
}

export function submitTx(token: string, payload: Record<string, unknown>) {
  return request<{ signature: string }>("/chain/tx/submit", {
    method: "POST",
    token,
    body: payload,
  });
}

export function confirmTx(token: string, payload: Record<string, unknown>) {
  return request<{
    confirmed: boolean;
    confirmation_status?: string;
    err?: unknown;
  }>("/chain/tx/confirm", {
    method: "POST",
    token,
    body: payload,
  });
}

export function submitAndConfirmTx(token: string, payload: Record<string, unknown>) {
  return request<{ signature: string; confirmation_status?: string }>(
    "/chain/tx/submit-and-confirm",
    {
      method: "POST",
      token,
      body: payload,
    }
  );
}
