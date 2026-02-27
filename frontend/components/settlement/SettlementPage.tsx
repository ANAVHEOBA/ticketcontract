"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { settlementApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

type SettlementPayload = {
  organizer_id: string;
  event_id: string;
  settlement_ref: string;
  transaction_base64: string;
};

export function SettlementPage() {
  const [payload, setPayload] = useState<SettlementPayload>({
    organizer_id: "",
    event_id: "",
    settlement_ref: "",
    transaction_base64: "",
  });
  const [settlementLookup, setSettlementLookup] = useState("");
  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [signature, setSignature] = useState<string | null>(null);
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

  const update = (key: keyof SettlementPayload, value: string) => {
    setPayload((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <BackofficeLayout title="Settlement" subtitle="Primary/resale settlement + finalize flows.">
      <section className={styles.section}>
        <h3>Payload</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={payload.organizer_id} onChange={(e) => update("organizer_id", e.target.value)} /></div>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={payload.event_id} onChange={(e) => update("event_id", e.target.value)} /></div>
          <div className={styles.field}><label>Settlement Ref</label><input className={styles.input} value={payload.settlement_ref} onChange={(e) => update("settlement_ref", e.target.value)} /></div>
        </div>
        <div className={styles.field}><label>Signed Transaction (base64)</label><textarea className={styles.textarea} value={payload.transaction_base64} onChange={(e) => update("transaction_base64", e.target.value)} /></div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Settle Primary", () => settlementApi.settlePrimaryRevenue(token(), payload))}>Submit Primary</button>
          <button className={styles.btn} onClick={() => run("Sim Primary", () => settlementApi.simulateSettlePrimaryRevenue(token(), payload))}>Sim Primary</button>
          <button className={styles.btn} onClick={() => run("Settle Resale", () => settlementApi.settleResaleRevenue(token(), payload))}>Submit Resale</button>
          <button className={styles.btn} onClick={() => run("Sim Resale", () => settlementApi.simulateSettleResaleRevenue(token(), payload))}>Sim Resale</button>
          <button className={styles.btn} onClick={() => run("Finalize", () => settlementApi.finalizeSettlement(token(), payload))}>Submit Finalize</button>
          <button className={styles.btn} onClick={() => run("Sim Finalize", () => settlementApi.simulateFinalizeSettlement(token(), payload))}>Sim Finalize</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Read Settlement</h3>
        <p className={styles.note}>Endpoint availability depends on backend version.</p>
        <div className={styles.field}><label>Settlement Ref</label><input className={styles.input} value={settlementLookup} onChange={(e) => setSettlementLookup(e.target.value)} /></div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Fetch Settlement", () => settlementApi.getSettlement(token(), settlementLookup))}>Fetch Settlement</button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} signature={signature} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
