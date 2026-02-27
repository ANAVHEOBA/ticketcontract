"use client";

import { useState } from "react";
import { BackofficeLayout, backofficeStyles as styles } from "@/components/backoffice/BackofficeLayout";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { deliveryApi, opsApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { readAuthSession } from "@/lib/session";

export function OpsPage() {
  const [limit, setLimit] = useState("100");
  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState("");

  const token = () => {
    const session = readAuthSession();
    if (!session.token) throw new Error("Sign in required.");
    return session.token;
  };

  const run = async (name: string, fn: () => Promise<unknown>, asText = false) => {
    setStatus("pending");
    setLabel(`${name}...`);
    setError(null);
    try {
      const out = await fn();
      setResult(asText ? String(out) : JSON.stringify(out, null, 2));
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
    <BackofficeLayout title="Ops / Admin / Delivery" subtitle="Operational metrics, alerts, logs, and API docs retrieval.">
      <section className={styles.section}>
        <h3>Ops</h3>
        <div className={styles.grid}>
          <div className={styles.field}><label>Audit Log Limit</label><input className={styles.input} value={limit} onChange={(e) => setLimit(e.target.value)} /></div>
        </div>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("Metrics", () => opsApi.getOpsMetrics(token()))}>Get Metrics</button>
          <button className={styles.btn} onClick={() => run("Alerts", () => opsApi.getOpsAlerts(token()))}>Get Alerts</button>
          <button className={styles.btn} onClick={() => run("Audit Logs", () => opsApi.getAuditLogs(token(), Number(limit) || 100))}>Get Audit Logs</button>
        </div>
      </section>

      <section className={styles.section}>
        <h3>Delivery Docs</h3>
        <div className={styles.actions}>
          <button className={styles.btn} onClick={() => run("OpenAPI YAML", () => deliveryApi.getOpenApiYaml(), true)}>OpenAPI</button>
          <button className={styles.btn} onClick={() => run("Postman", () => deliveryApi.getPostmanCollection(), true)}>Postman</button>
          <button className={styles.btn} onClick={() => run("Bruno", () => deliveryApi.getBrunoCollection(), true)}>Bruno</button>
        </div>
      </section>

      <TxStatusCard status={status} label={label} error={error} />
      {result ? <pre className={styles.result}>{result}</pre> : null}
    </BackofficeLayout>
  );
}
