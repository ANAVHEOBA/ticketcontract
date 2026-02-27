"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { eventApi } from "@/lib/api";
import { getCachedEvents } from "@/lib/eventsCache";
import { getProgramId } from "@/lib/solana";
import { hasOrganizerScope, readAuthSession } from "@/lib/session";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { PublicKey } from "@solana/web3.js";
import styles from "./EventsHome.module.css";

type EventRecord = {
  event_id: string;
  organizer_id: string;
  name?: string | null;
  status?: string | null;
  metadata_uri?: string | null;
  starts_at_epoch?: number | null;
  ends_at_epoch?: number | null;
};

function formatEpoch(epoch?: number | null): string {
  if (!epoch) return "TBD";
  return new Date(epoch * 1000).toLocaleString();
}

export function EventsHome() {
  const [events, setEvents] = useState<EventRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"upcoming" | "past">("upcoming");
  const [refreshAt, setRefreshAt] = useState(0);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);

      try {
        const token = localStorage.getItem("ticketing_access_token");
        if (!token) {
          setEvents([]);
          setError("Sign in required.");
          return;
        }

        const session = readAuthSession();
        const firstScoped = session.organizerScopes.find((scope) => scope && scope !== "*");
        const response = await eventApi.listEvents(
          token,
          firstScoped ? { organizer_id: firstScoped } : undefined,
        );

        const remote = (response.events as EventRecord[]) ?? [];
        const local = getCachedEvents().map((cached) => ({
          event_id: cached.eventPda,
          organizer_id: cached.organizerId,
          name: cached.name,
          status: "Draft",
          metadata_uri: null,
          starts_at_epoch: cached.startsAtEpoch,
          ends_at_epoch: cached.endsAtEpoch,
        }));

        const merged = new Map<string, EventRecord>();
        for (const item of [...local, ...remote]) {
          merged.set(item.event_id, item);
        }
        setEvents(Array.from(merged.values()));
      } catch (loadError) {
        setEvents([]);
        setError(loadError instanceof Error ? loadError.message : "Could not load events.");
      } finally {
        setLoading(false);
      }
    };

    void load();
  }, [refreshAt]);

  const nowSec = Math.floor(Date.now() / 1000);
  const session = readAuthSession();
  const walletOrganizerPda = useMemo(() => {
    if (!session.wallet) return null;
    try {
      const wallet = new PublicKey(session.wallet);
      const [organizerPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), wallet.toBuffer()],
        getProgramId(),
      );
      return organizerPda.toBase58();
    } catch {
      return null;
    }
  }, [session.wallet]);

  const canWriteEvent = (organizerId: string): boolean => {
    if (hasOrganizerScope(session.organizerScopes, organizerId)) return true;
    return walletOrganizerPda === organizerId;
  };

  const filteredEvents = useMemo(
    () =>
      events.filter((event) => {
        if (!event.starts_at_epoch) return activeTab === "upcoming";
        return activeTab === "upcoming" ? event.starts_at_epoch >= nowSec : event.starts_at_epoch < nowSec;
      }),
    [activeTab, events, nowSec],
  );

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.headerBar}>
          <h1>Events</h1>
          <div className={styles.headerActions}>
            <Link href="/tickets" className={styles.inlineLinkBtn}>
              My Tickets
            </Link>
            <Link href="/secondary-sale" className={styles.inlineLinkBtn}>
              Secondary Sale
            </Link>
            <Link href="/resale-policy" className={styles.inlineLinkBtn}>
              Resale Policy
            </Link>
            <Link href="/ops" className={styles.inlineLinkBtn}>
              Backoffice
            </Link>
            <button className={styles.refreshBtn} onClick={() => setRefreshAt(Date.now())}>
              Refresh
            </button>
            <div className={styles.tabs}>
              <button
                className={`${styles.tab} ${activeTab === "upcoming" ? styles.active : ""}`}
                onClick={() => setActiveTab("upcoming")}
              >
                Upcoming
              </button>
              <button
                className={`${styles.tab} ${activeTab === "past" ? styles.active : ""}`}
                onClick={() => setActiveTab("past")}
              >
                Past
              </button>
            </div>
          </div>
        </section>

        {loading ? <section className={styles.statusCard}>Loading events...</section> : null}
        {!loading && error ? <section className={styles.statusCard}>{error}</section> : null}

        {!loading && !error && filteredEvents.length > 0 ? (
          <section className={styles.listSection}>
            {filteredEvents.map((event) => (
              <article key={event.event_id} className={styles.eventCard}>
                <div className={styles.eventMeta}>
                  <h3>{event.name || event.event_id}</h3>
                  <p>Organizer: {event.organizer_id}</p>
                  <p>Start: {formatEpoch(event.starts_at_epoch)}</p>
                  <p>Status: {event.status || "unknown"}</p>
                  <p>
                    Access: {canWriteEvent(event.organizer_id) ? "Organizer owner" : "Read-only"}
                  </p>
                </div>
                <div className={styles.eventActions}>
                  <Link href={`/events/${encodeURIComponent(event.event_id)}`} className={styles.actionBtn}>
                    View / Buy
                  </Link>
                  <Link href="/secondary-sale" className={styles.actionBtn}>
                    Resale
                  </Link>
                </div>
              </article>
            ))}
          </section>
        ) : null}

        {!loading && !error && filteredEvents.length === 0 ? (
          <section className={styles.emptyState}>
            <div className={styles.illustration} aria-hidden>
              <div className={styles.cardLarge} />
              <div className={styles.cardSmall}>0</div>
            </div>
            <h3>No {activeTab === "upcoming" ? "Upcoming" : "Past"} Events</h3>
            <p>You have no {activeTab} events. Why not host one?</p>
            <Link href="/create" className={styles.createBtn}>
              Create Event
            </Link>
          </section>
        ) : null}
      </main>
      <Footer />
    </div>
  );
}
