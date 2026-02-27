"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { financingApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

type FinancingPayload = {
  organizer_id: string;
  event_id: string;
  offer_id?: string;
  transaction_base64: string;
};

export function FinancingPage() {
  const [payload, setPayload] = useState<FinancingPayload>({
    organizer_id: "",
    event_id: "",
    offer_id: "",
    transaction_base64: "",
  });
  const [offerLookup, setOfferLookup] = useState("");
  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [signature, setSignature] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string>("");

  const update = (key: keyof FinancingPayload, value: string) => {
    setPayload((prev) => ({ ...prev, [key]: value }));
  };

  const token = () => {
    const session = readAuthSession();
    if (!session.token) throw new Error("Sign in required.");
    return session.token;
  };

  const txPayload = {
    ...payload,
    offer_id: payload.offer_id || undefined,
  };

  const run = async (name: string, fn: () => Promise<unknown>) => {
    setStatus("pending");
    setLabel(`${name}...`);
    setError(null);
    setSignature(null);
    try {
      const out = await fn();
      setResult(JSON.stringify(out, null, 2));
      const sig = typeof out === "object" && out && "signature" in out ? String((out as { signature?: string }).signature || "") : "";
      setSignature(sig || null);
      setStatus("confirmed");
      setLabel(`${name} complete.`);
    } catch (e) {
      const message = e instanceof ApiError ? e.message : e instanceof Error ? e.message : "Request failed.";
      setStatus("failed");
      setLabel(`${name} failed.`);
      setError(message);
    }
  };

  return (
    <BackofficeLayout title="Financing" subtitle="Offer lifecycle + disbursement controls.">
      <section className={styles.section}>
        <h3>Payload</h3>
        <div className={styles.grid}>
          <div className={styles.field}>
            <label>Organizer ID</label>
            <input className={styles.input} value={payload.organizer_id} onChange={(e) => update("organizer_id", e.target.value)} />
          </div>
          <div className={styles.field}>
            <label>Event ID</label>
            <input className={styles.input} value={payload.event_id} onChange={(e) => update("event_id", e.target.value)} />
          </div>
          <div className={styles.field}>
            <label>Offer ID (optional)</label>
            <input className={styles.input} value={payload.offer_id} onChange={(e) => update("offer_id", e.target.value)} />
          </div>
        </div>
        <div className={styles.field}>
          <label>Signed Transaction (base64)</label>
          <textarea className={styles.textarea} value={payload.transaction_base64} onChange={(e) => update("transaction_base64", e.target.value)} />
        </div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Create Offer", () => financingApi.createFinancingOffer(token(), txPayload))}>Submit Offer</button>
          <button className={styles.btn} onClick={() => run("Sim Create Offer", () => financingApi.simulateCreateFinancingOffer(token(), txPayload))}>Sim Offer</button>
          <button className={styles.btn} onClick={() => run("Accept Offer", () => financingApi.acceptFinancingOffer(token(), txPayload))}>Accept</button>
          <button className={styles.btn} onClick={() => run("Sim Accept", () => financingApi.simulateAcceptFinancingOffer(token(), txPayload))}>Sim Accept</button>
          <button className={styles.btn} onClick={() => run("Reject Offer", () => financingApi.rejectFinancingOffer(token(), txPayload))}>Reject</button>
          <button className={styles.btn} onClick={() => run("Sim Reject", () => financingApi.simulateRejectFinancingOffer(token(), txPayload))}>Sim Reject</button>
          <button className={styles.btn} onClick={() => run("Disburse", () => financingApi.disburseAdvance(token(), txPayload))}>Disburse</button>
          <button className={styles.btn} onClick={() => run("Sim Disburse", () => financingApi.simulateDisburseAdvance(token(), txPayload))}>Sim Disburse</button>
          <button className={styles.btn} onClick={() => run("Clawback", () => financingApi.clawbackDisbursement(token(), txPayload))}>Clawback</button>
          <button className={styles.btn} onClick={() => run("Sim Clawback", () => financingApi.simulateClawbackDisbursement(token(), txPayload))}>Sim Clawback</button>
          <button className={styles.btn} onClick={() => run("Freeze", () => financingApi.setFinancingFreeze(token(), txPayload))}>Freeze</button>
          <button className={styles.btn} onClick={() => run("Sim Freeze", () => financingApi.simulateSetFinancingFreeze(token(), txPayload))}>Sim Freeze</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Read Offer</h3>
        <div className={styles.grid}>
          <div className={styles.field}>
            <label>Offer ID</label>
            <input className={styles.input} value={offerLookup} onChange={(e) => setOfferLookup(e.target.value)} />
          </div>
        </div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Fetch Offer", () => financingApi.getFinancingOffer(token(), offerLookup))}>Fetch Offer</button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} signature={signature} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
