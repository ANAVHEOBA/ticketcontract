"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { resalePolicyApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { readAuthSession } from "@/lib/session";
import styles from "./ResalePolicyPage.module.css";

export function ResalePolicyDetailPage() {
  const params = useParams<{ policyId: string }>();
  const policyId = decodeURIComponent(params.policyId);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [policy, setPolicy] = useState<unknown | null>(null);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const session = readAuthSession();
        if (!session.token) throw new Error("Sign in required.");
        const response = await resalePolicyApi.getResalePolicyById(session.token, policyId);
        setPolicy(response.policy ?? null);
      } catch (loadError) {
        const message =
          loadError instanceof ApiError
            ? loadError.message
            : loadError instanceof Error
              ? loadError.message
              : "Could not load policy.";
        setError(message);
      } finally {
        setLoading(false);
      }
    };
    void load();
  }, [policyId]);

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>Resale Policy Detail</h1>
          <p>GET /resale-policy/{policyId}</p>
          <div className={styles.links}>
            <Link href="/resale-policy">Back to Resale Policy</Link>
            <Link href="/dashboard">Dashboard</Link>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Policy ID</h2>
          <p className={styles.caption}>{policyId}</p>
          {loading ? <p className={styles.caption}>Loading...</p> : null}
          {error ? <p className={styles.caption}>{error}</p> : null}
          {!loading && !error ? <pre className={styles.pre}>{JSON.stringify(policy, null, 2)}</pre> : null}
        </section>
      </main>
      <Footer />
    </div>
  );
}
