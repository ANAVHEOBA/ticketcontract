"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { disputesApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

type DisputePayload = {
  organizer_id: string;
  event_id: string;
  ticket_id: string;
  dispute_id?: string;
  transaction_base64: string;
};

export function DisputesPage() {
  const [payload, setPayload] = useState<DisputePayload>({
    organizer_id: "",
    event_id: "",
    ticket_id: "",
    dispute_id: "",
    transaction_base64: "",
  });
  const [queueStatus, setQueueStatus] = useState("");
  const [queueLimit, setQueueLimit] = useState("50");
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

  const update = (key: keyof DisputePayload, value: string) => setPayload((prev) => ({ ...prev, [key]: value }));

  const txPayload = { ...payload, dispute_id: payload.dispute_id || undefined };

  return (
    <BackofficeLayout title="Disputes / Refund" subtitle="Refund, flag, chargeback, and queue read.">
      <section className={styles.section}>
        <h3>Dispute Action Payload</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={payload.organizer_id} onChange={(e) => update("organizer_id", e.target.value)} /></div>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={payload.event_id} onChange={(e) => update("event_id", e.target.value)} /></div>
          <div className={styles.field}><label>Ticket ID</label><input className={styles.input} value={payload.ticket_id} onChange={(e) => update("ticket_id", e.target.value)} /></div>
          <div className={styles.field}><label>Dispute ID (optional)</label><input className={styles.input} value={payload.dispute_id} onChange={(e) => update("dispute_id", e.target.value)} /></div>
        </div>
        <div className={styles.field}><label>Signed Transaction (base64)</label><textarea className={styles.textarea} value={payload.transaction_base64} onChange={(e) => update("transaction_base64", e.target.value)} /></div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Refund", () => disputesApi.refundTicket(token(), txPayload))}>Refund</button>
          <button className={styles.btn} onClick={() => run("Flag", () => disputesApi.flagDispute(token(), txPayload))}>Flag</button>
          <button className={styles.btn} onClick={() => run("Chargeback", () => disputesApi.flagChargeback(token(), txPayload))}>Chargeback</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Dispute Queue</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Status</label><input className={styles.input} value={queueStatus} onChange={(e) => setQueueStatus(e.target.value)} placeholder="open|resolved" /></div>
          <div className={styles.field}><label>Limit</label><input className={styles.input} value={queueLimit} onChange={(e) => setQueueLimit(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button
            className={styles.btn}
            onClick={() =>
              run("Fetch Queue", () =>
                disputesApi.getDisputeQueue(token(), {
                  organizer_id: payload.organizer_id || undefined,
                  status: queueStatus || undefined,
                  limit: Number(queueLimit) || undefined,
                })
              )
            }
          >
            Fetch Queue
          </button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} signature={signature} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
