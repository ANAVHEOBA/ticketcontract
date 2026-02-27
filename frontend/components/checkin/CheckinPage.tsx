"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { checkinApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

type CheckinPayload = {
  organizer_id: string;
  event_id: string;
  class_id: string;
  ticket_id: string;
  gate_id: string;
  scanner_id: string;
  transaction_base64: string;
};

export function CheckinPage() {
  const [payload, setPayload] = useState<CheckinPayload>({
    organizer_id: "",
    event_id: "",
    class_id: "",
    ticket_id: "",
    gate_id: "",
    scanner_id: "",
    transaction_base64: "",
  });
  const [accepted, setAccepted] = useState(true);
  const [reason, setReason] = useState("");
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

  const update = (key: keyof CheckinPayload, value: string) => {
    setPayload((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <BackofficeLayout title="Check-in" subtitle="Gate policy, ticket check-in, and gate response endpoint.">
      <section className={styles.section}>
        <h3>Payload</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={payload.organizer_id} onChange={(e) => update("organizer_id", e.target.value)} /></div>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={payload.event_id} onChange={(e) => update("event_id", e.target.value)} /></div>
          <div className={styles.field}><label>Class ID</label><input className={styles.input} value={payload.class_id} onChange={(e) => update("class_id", e.target.value)} /></div>
          <div className={styles.field}><label>Ticket ID</label><input className={styles.input} value={payload.ticket_id} onChange={(e) => update("ticket_id", e.target.value)} /></div>
          <div className={styles.field}><label>Gate ID</label><input className={styles.input} value={payload.gate_id} onChange={(e) => update("gate_id", e.target.value)} /></div>
          <div className={styles.field}><label>Scanner ID</label><input className={styles.input} value={payload.scanner_id} onChange={(e) => update("scanner_id", e.target.value)} /></div>
        </div>
        <div className={styles.field}><label>Signed Transaction (base64)</label><textarea className={styles.textarea} value={payload.transaction_base64} onChange={(e) => update("transaction_base64", e.target.value)} /></div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Set Policy", () => checkinApi.setCheckinPolicy(token(), payload))}>Submit Policy</button>
          <button className={styles.btn} onClick={() => run("Check-in Ticket", () => checkinApi.checkinTicket(token(), payload))}>Submit Check-in</button>
          <button className={styles.btn} onClick={() => run("Sim Check-in", () => checkinApi.simulateCheckinTicket(token(), payload))}>Sim Check-in</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Gate Response</h3>
        <p className={styles.note}>Some backend builds do not expose this path yet.</p>
        <div className={styles.grid}>
          <div className={styles.field}>
            <label>Accepted</label>
            <select className={styles.input} value={accepted ? "yes" : "no"} onChange={(e) => setAccepted(e.target.value === "yes")}>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </div>
          <div className={styles.field}><label>Reason</label><input className={styles.input} value={reason} onChange={(e) => setReason(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button
            className={styles.btn}
            onClick={() =>
              run("Send Gate Response", () =>
                checkinApi.sendGateResponse(token(), {
                  gate_id: payload.gate_id,
                  ticket_id: payload.ticket_id,
                  scanner_id: payload.scanner_id || undefined,
                  accepted,
                  reason: reason || undefined,
                  checked_in_at_epoch: Math.floor(Date.now() / 1000),
                })
              )
            }
          >
            Submit Gate Response
          </button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} signature={signature} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
