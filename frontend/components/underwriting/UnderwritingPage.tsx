"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { resaleCompilerApi, underwritingApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

export function UnderwritingPage() {
  const [organizerId, setOrganizerId] = useState("");
  const [eventId, setEventId] = useState("");
  const [requestedAdvance, setRequestedAdvance] = useState("1000000");
  const [projectedGross, setProjectedGross] = useState("5000000");
  const [tenorDays, setTenorDays] = useState("45");

  const [estimatedFaceValue, setEstimatedFaceValue] = useState("250000000");
  const [liquidityWeight, setLiquidityWeight] = useState("0.5");
  const [fairnessWeight, setFairnessWeight] = useState("0.3");
  const [royaltyWeight, setRoyaltyWeight] = useState("0.2");

  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState("");

  const token = () => {
    const session = readAuthSession();
    if (!session.token) throw new Error("Sign in required.");
    return session.token;
  };

  const run = async (name: string, fn: () => Promise<unknown>) => {
    setStatus("pending");
    setLabel(`${name}...`);
    setError(null);
    try {
      const out = await fn();
      setResult(JSON.stringify(out, null, 2));
      setStatus("confirmed");
      setLabel(`${name} complete.`);
    } catch (e) {
      const message = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "Request failed.";
      setStatus("failed");
      setLabel(`${name} failed.`);
      setError(message);
    }
  };

  const underwritingPayload = {
    organizer_id: organizerId,
    event_id: eventId,
    requested_advance_amount: Number(requestedAdvance),
    projected_gross_revenue: Number(projectedGross),
    tenor_days: Number(tenorDays),
  };

  return (
    <BackofficeLayout title="Underwriting / Resale Compiler" subtitle="Risk scoring + resale policy simulation.">
      <section className={styles.section}>
        <h3>Underwriting Score</h3>
        <p className={styles.note}>Uses `/underwriting/score` when available and falls back to `/underwriting/financing/proposal`.</p>
        <div className={styles.grid}>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={organizerId} onChange={(e) => setOrganizerId(e.target.value)} /></div>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={eventId} onChange={(e) => setEventId(e.target.value)} /></div>
          <div className={styles.field}><label>Requested Advance (lamports)</label><input className={styles.input} value={requestedAdvance} onChange={(e) => setRequestedAdvance(e.target.value)} /></div>
          <div className={styles.field}><label>Projected Gross (lamports)</label><input className={styles.input} value={projectedGross} onChange={(e) => setProjectedGross(e.target.value)} /></div>
          <div className={styles.field}><label>Tenor Days</label><input className={styles.input} value={tenorDays} onChange={(e) => setTenorDays(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Underwriting Score", () => underwritingApi.getUnderwritingScore(token(), underwritingPayload))}>Run Score</button>
          <button className={styles.btn} onClick={() => run("Underwriting Proposal", () => underwritingApi.getUnderwritingProposal(token(), underwritingPayload))}>Run Proposal</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Resale Compiler Simulation</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Estimated Face Value (lamports)</label><input className={styles.input} value={estimatedFaceValue} onChange={(e) => setEstimatedFaceValue(e.target.value)} /></div>
          <div className={styles.field}><label>Liquidity Weight</label><input className={styles.input} value={liquidityWeight} onChange={(e) => setLiquidityWeight(e.target.value)} /></div>
          <div className={styles.field}><label>Fairness Weight</label><input className={styles.input} value={fairnessWeight} onChange={(e) => setFairnessWeight(e.target.value)} /></div>
          <div className={styles.field}><label>Royalty Weight</label><input className={styles.input} value={royaltyWeight} onChange={(e) => setRoyaltyWeight(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button
            className={styles.btn}
            onClick={() =>
              run("Resale Compiler Simulate", () =>
                resaleCompilerApi.simulateResaleCompiler(token(), {
                  organizer_id: organizerId,
                  event_id: eventId,
                  estimated_face_value: Number(estimatedFaceValue),
                  goals: {
                    liquidity_weight: Number(liquidityWeight),
                    fairness_weight: Number(fairnessWeight),
                    royalty_weight: Number(royaltyWeight),
                  },
                  candidates: [],
                })
              )
            }
          >
            Run Simulation
          </button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
