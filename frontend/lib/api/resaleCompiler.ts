import { request } from "./http";

export function simulateResaleCompiler(token: string, payload: Record<string, unknown>) {
  return request<{
    organizer_id: string;
    event_id: string;
    class_id?: string;
    goals: unknown;
    inputs: unknown;
    simulations: unknown[];
    recommendation?: unknown;
  }>("/resale-compiler/simulate", {
    method: "POST",
    token,
    body: payload,
  });
}
