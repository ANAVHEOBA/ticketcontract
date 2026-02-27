"use client";

import styles from "./TxStatusCard.module.css";

export type TxStatus = "idle" | "pending" | "confirmed" | "failed";

type Props = {
  status: TxStatus;
  label: string;
  signature?: string | null;
  error?: string | null;
};

function signatureUrl(signature: string): string {
  const cluster = process.env.NEXT_PUBLIC_SOLANA_CLUSTER ?? "devnet";
  const suffix = cluster === "mainnet-beta" ? "" : `?cluster=${cluster}`;
  return `https://explorer.solana.com/tx/${signature}${suffix}`;
}

export function TxStatusCard({ status, label, signature, error }: Props) {
  if (status === "idle") return null;

  return (
    <section className={`${styles.card} ${styles[status]}`}>
      <p className={styles.label}>{label}</p>
      {signature ? (
        <a href={signatureUrl(signature)} target="_blank" rel="noreferrer" className={styles.link}>
          View signature
        </a>
      ) : null}
      {error ? <p className={styles.error}>{error}</p> : null}
    </section>
  );
}
