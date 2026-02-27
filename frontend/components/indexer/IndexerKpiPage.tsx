"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { indexerApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

export function IndexerKpiPage() {
  const [eventId, setEventId] = useState("");
  const [organizerId, setOrganizerId] = useState("");
  const [wallet, setWallet] = useState("");
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

  return (
    <BackofficeLayout title="Indexer / KPI Reads" subtitle="Indexer status and KPI observability endpoints.">
      <section className={styles.section}>
        <h3>Inputs</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={eventId} onChange={(e) => setEventId(e.target.value)} /></div>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={organizerId} onChange={(e) => setOrganizerId(e.target.value)} /></div>
          <div className={styles.field}><label>Wallet (fan quality query)</label><input className={styles.input} value={wallet} onChange={(e) => setWallet(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Indexer Status", () => indexerApi.getIndexerStatus(token()))}>Indexer Status</button>
          <button className={styles.btn} onClick={() => run("Event Sales KPI", () => indexerApi.getEventSalesKpi(token(), eventId))}>Event Sales KPI</button>
          <button className={styles.btn} onClick={() => run("Event Sales KPI (Query)", () => indexerApi.getEventSalesKpiQuery(token(), { event_id: eventId, organizer_id: organizerId }))}>Event Sales (Query)</button>
          <button className={styles.btn} onClick={() => run("Fan Quality KPI", () => indexerApi.getFanQualityKpi(token(), { event_id: eventId || undefined, organizer_id: organizerId || undefined, wallet: wallet || undefined }))}>Fan Quality KPI</button>
          <button className={styles.btn} onClick={() => run("Financing Cash KPI", () => indexerApi.getFinancingCashKpi(token(), { organizer_id: organizerId, event_id: eventId }))}>Financing Cash KPI</button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
