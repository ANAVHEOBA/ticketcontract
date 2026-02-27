"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { eventApi } from "@/lib/api";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { getCachedEvents } from "@/lib/eventsCache";
import { decodeTicket, getProgramId, TICKET_OWNER_OFFSET } from "@/lib/solana";
import { readAuthSession } from "@/lib/session";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import styles from "./MyTicketsPage.module.css";

type TicketRow = {
  ticketPda: string;
  eventPda: string;
  classPda: string;
  ticketId: number;
  status: number;
  paidLamports: bigint;
  createdAt: bigint;
  eventName?: string;
};

function statusLabel(status: number): string {
  switch (status) {
    case 1:
      return "Active";
    case 2:
      return "Checked In";
    case 3:
      return "Refunded";
    case 4:
      return "Invalidated";
    default:
      return "Unknown";
  }
}

function formatSol(lamports: bigint): string {
  return (Number(lamports) / 1_000_000_000).toFixed(4);
}

function formatTs(ts: bigint): string {
  const ms = Number(ts) * 1000;
  return Number.isFinite(ms) ? new Date(ms).toLocaleString() : "TBD";
}

export function MyTicketsPage() {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const [tickets, setTickets] = useState<TicketRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        if (!publicKey) {
          setError("Connect wallet to view tickets.");
          return;
        }

        const ownerBase58 = publicKey.toBase58();
        const programId = getProgramId();
        const session = readAuthSession();

        const accounts = await connection.getProgramAccounts(programId, {
          filters: [{ memcmp: { offset: TICKET_OWNER_OFFSET, bytes: ownerBase58 } }],
          commitment: "confirmed",
        });

        const byEventName = new Map<string, string>();
        for (const cached of getCachedEvents()) {
          byEventName.set(cached.eventPda, cached.name);
        }

        if (session.token) {
          try {
            const response = await eventApi.listEvents(session.token);
            for (const raw of (response.events as Record<string, unknown>[]) ?? []) {
              const eventId = String(raw.event_id ?? "");
              const name = String(raw.name ?? "");
              if (eventId && name) byEventName.set(eventId, name);
            }
          } catch {
            // keep cached event names only
          }
        }

        const rows = accounts
          .map((account) => {
            try {
              const ticket = decodeTicket(Buffer.from(account.account.data));
              return {
                ticketPda: account.pubkey.toBase58(),
                eventPda: ticket.event.toBase58(),
                classPda: ticket.ticketClass.toBase58(),
                ticketId: ticket.ticketId,
                status: ticket.status,
                paidLamports: ticket.paidAmountLamports,
                createdAt: ticket.createdAt,
                eventName: byEventName.get(ticket.event.toBase58()),
              } as TicketRow;
            } catch {
              return null;
            }
          })
          .filter((row): row is TicketRow => row !== null)
          .sort((a, b) => Number(b.createdAt - a.createdAt));

        setTickets(rows);
      } catch (loadError) {
        setError(loadError instanceof Error ? loadError.message : "Could not load tickets.");
      } finally {
        setLoading(false);
      }
    };

    void load();
  }, [connection, publicKey]);

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>My Tickets</h1>
          <Link href="/dashboard" className={styles.backBtn}>
            Back to Dashboard
          </Link>
        </section>

        {loading ? <section className={styles.status}>Loading tickets...</section> : null}
        {!loading && error ? <section className={styles.status}>{error}</section> : null}

        {!loading && !error && tickets.length === 0 ? (
          <section className={styles.status}>No tickets yet. Buy your first ticket from an event.</section>
        ) : null}

        {!loading && !error && tickets.length > 0 ? (
          <section className={styles.list}>
            {tickets.map((ticket) => (
              <article key={ticket.ticketPda} className={styles.card}>
                <h3>{ticket.eventName ?? ticket.eventPda}</h3>
                <p>Ticket #{ticket.ticketId}</p>
                <p>Status: {statusLabel(ticket.status)}</p>
                <p>Paid: {formatSol(ticket.paidLamports)} SOL</p>
                <p>Purchased: {formatTs(ticket.createdAt)}</p>
                <div className={styles.actions}>
                  <Link href={`/events/${encodeURIComponent(ticket.eventPda)}`} className={styles.btn}>
                    View Event
                  </Link>
                  <a
                    href={`https://explorer.solana.com/address/${ticket.ticketPda}?cluster=${process.env.NEXT_PUBLIC_SOLANA_CLUSTER ?? "devnet"}`}
                    target="_blank"
                    rel="noreferrer"
                    className={styles.btn}
                  >
                    View Ticket PDA
                  </a>
                </div>
              </article>
            ))}
          </section>
        ) : null}
      </main>
      <Footer />
    </div>
  );
}
