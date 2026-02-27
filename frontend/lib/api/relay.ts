import { request } from "./http";

export type RelaySubmitPayload = {
  transaction_base64: string;
  expected_instructions?: string[];
  skip_preflight?: boolean;
  max_retries?: number;
  timeout_ms?: number;
  poll_ms?: number;
};

export function submitViaRelayer(token: string, payload: RelaySubmitPayload) {
  return request<{ signature: string; confirmation_status?: string }>("/relay/submit", {
    method: "POST",
    token,
    body: payload,
  });
}
