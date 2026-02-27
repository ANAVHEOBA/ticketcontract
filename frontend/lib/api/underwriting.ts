import { request } from "./http";

export function getUnderwritingProposal(
  token: string,
  payload: {
    organizer_id: string;
    event_id: string;
    requested_advance_amount: number;
    projected_gross_revenue: number;
    tenor_days?: number;
  }
) {
  return request<{ decision: unknown }>("/underwriting/financing/proposal", {
    method: "POST",
    token,
    body: payload,
  });
}

export async function getUnderwritingScore(
  token: string,
  payload: {
    organizer_id: string;
    event_id: string;
    requested_advance_amount: number;
    projected_gross_revenue: number;
    tenor_days?: number;
  }
) {
  try {
    return await request<{ decision: unknown }>("/underwriting/score", {
      method: "POST",
      token,
      body: payload,
    });
  } catch {
    return request<{ decision: unknown }>("/underwriting/financing/proposal", {
      method: "POST",
      token,
      body: payload,
    });
  }
}
