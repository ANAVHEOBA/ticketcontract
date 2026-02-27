import { request } from "./http";
import type { AuthTokenResponse } from "./types";

export type NonceResponse = { nonce: string; message: string; expires_at_epoch: number };

export function issueNonce(wallet: string) {
  return request<NonceResponse>("/auth/nonce", {
    method: "POST",
    body: { wallet },
  });
}

export function verifySignature(payload: {
  wallet: string;
  nonce: string;
  signature: string;
}) {
  return request<AuthTokenResponse>("/auth/verify", {
    method: "POST",
    body: payload,
  });
}

export function verifyProvider(payload: {
  provider: "google";
  id_token: string;
  wallet: string;
  nonce: string;
  signature: string;
}) {
  return request<AuthTokenResponse>("/auth/provider/verify", {
    method: "POST",
    body: payload,
  });
}

export function me(token: string) {
  return request<{ wallet: string; role: string; organizer_scopes: string[] }>("/auth/me", {
    token,
  });
}
