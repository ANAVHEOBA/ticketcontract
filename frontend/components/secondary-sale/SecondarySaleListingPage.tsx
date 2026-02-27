"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { secondarySaleApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { readAuthSession } from "@/lib/session";
import styles from "./SecondarySalePage.module.css";

export function SecondarySaleListingPage() {
  const params = useParams<{ listingId: string }>();
  const listingId = decodeURIComponent(params.listingId);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [listing, setListing] = useState<unknown | null>(null);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const session = readAuthSession();
        if (!session.token) throw new Error("Sign in required.");
        const response = await secondarySaleApi.getListing(session.token, listingId);
        setListing(response.listing ?? null);
      } catch (loadError) {
        const message =
          loadError instanceof ApiError
            ? loadError.message
            : loadError instanceof Error
              ? loadError.message
              : "Could not load listing.";
        setError(message);
      } finally {
        setLoading(false);
      }
    };
    void load();
  }, [listingId]);

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>Listing Detail</h1>
          <p>GET /secondary-sale/listings/{listingId}</p>
          <div className={styles.links}>
            <Link href="/secondary-sale">Back to Secondary Sale</Link>
            <Link href="/dashboard">Dashboard</Link>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Listing ID</h2>
          <p className={styles.caption}>{listingId}</p>
          {loading ? <p className={styles.caption}>Loading...</p> : null}
          {error ? <p className={styles.caption}>{error}</p> : null}
          {!loading && !error ? (
            <pre className={styles.pre}>{JSON.stringify(listing, null, 2)}</pre>
          ) : null}
        </section>
      </main>
      <Footer />
    </div>
  );
}
