"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { loyaltyTrustApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

type SignalPayload = {
  organizer_id: string;
  event_id: string;
  wallet: string;
  signal_id?: string;
  transaction_base64: string;
};

export function LoyaltyTrustPage() {
  const [payload, setPayload] = useState<SignalPayload>({
    organizer_id: "",
    event_id: "",
    wallet: "",
    signal_id: "",
    transaction_base64: "",
  });
  const [schemaVersion, setSchemaVersion] = useState("1");
  const [loyaltyWallet, setLoyaltyWallet] = useState("");
  const [trustLimit, setTrustLimit] = useState("50");
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

  const update = (key: keyof SignalPayload, value: string) => setPayload((prev) => ({ ...prev, [key]: value }));
  const txPayload = { ...payload, signal_id: payload.signal_id || undefined };

  return (
    <BackofficeLayout title="Loyalty + Trust" subtitle="Points accrual, trust signals, and read APIs.">
      <section className={styles.section}>
        <h3>Write Payload</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Organizer ID</label><input className={styles.input} value={payload.organizer_id} onChange={(e) => update("organizer_id", e.target.value)} /></div>
          <div className={styles.field}><label>Event ID</label><input className={styles.input} value={payload.event_id} onChange={(e) => update("event_id", e.target.value)} /></div>
          <div className={styles.field}><label>Wallet</label><input className={styles.input} value={payload.wallet} onChange={(e) => update("wallet", e.target.value)} /></div>
          <div className={styles.field}><label>Signal ID (optional)</label><input className={styles.input} value={payload.signal_id} onChange={(e) => update("signal_id", e.target.value)} /></div>
        </div>
        <div className={styles.field}><label>Signed Transaction (base64)</label><textarea className={styles.textarea} value={payload.transaction_base64} onChange={(e) => update("transaction_base64", e.target.value)} /></div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Accrue Points", () => loyaltyTrustApi.accruePoints(token(), txPayload))}>Accrue</button>
          <button className={styles.btn} onClick={() => run("Redeem Points", () => loyaltyTrustApi.redeemPoints(token(), txPayload))}>Redeem</button>
          <button className={styles.btn} onClick={() => run("Trust Purchase", () => loyaltyTrustApi.recordPurchaseSignal(token(), txPayload))}>Trust Purchase</button>
          <button className={styles.btn} onClick={() => run("Trust Attendance", () => loyaltyTrustApi.recordAttendanceSignal(token(), txPayload))}>Trust Attendance</button>
          <button className={styles.btn} onClick={() => run("Trust Abuse", () => loyaltyTrustApi.flagTrustAbuse(token(), txPayload))}>Trust Abuse</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Schema + Reads</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Schema Version</label><input className={styles.input} value={schemaVersion} onChange={(e) => setSchemaVersion(e.target.value)} /></div>
          <div className={styles.field}><label>Loyalty Wallet Query</label><input className={styles.input} value={loyaltyWallet} onChange={(e) => setLoyaltyWallet(e.target.value)} /></div>
          <div className={styles.field}><label>Trust Limit</label><input className={styles.input} value={trustLimit} onChange={(e) => setTrustLimit(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button
            className={styles.btn}
            onClick={() =>
              run("Set Schema", () =>
                loyaltyTrustApi.setTrustSchemaVersion(token(), {
                  organizer_id: payload.organizer_id || undefined,
                  schema_version: Number(schemaVersion),
                  transaction_base64: payload.transaction_base64,
                })
              )
            }
          >
            Set Schema
          </button>
          <button
            className={styles.btn}
            onClick={() =>
              run("Get Loyalty", () =>
                loyaltyTrustApi.getLoyalty(token(), {
                  wallet: loyaltyWallet || payload.wallet,
                  organizer_id: payload.organizer_id || undefined,
                })
              )
            }
          >
            Get Loyalty
          </button>
          <button
            className={styles.btn}
            onClick={() =>
              run("Get Trust Signals", () =>
                loyaltyTrustApi.getTrustSignals(token(), {
                  wallet: payload.wallet || undefined,
                  organizer_id: payload.organizer_id || undefined,
                  event_id: payload.event_id || undefined,
                  limit: Number(trustLimit) || undefined,
                })
              )
            }
          >
            Get Trust Signals
          </button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} signature={signature} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
