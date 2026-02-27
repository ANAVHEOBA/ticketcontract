"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { secondarySaleApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import {
  decodeListing,
  decodeResalePolicyRoyaltyVault,
  getProgramId,
} from "@/lib/solana";
import { readAuthSession } from "@/lib/session";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import styles from "./SecondarySalePage.module.css";

const LIST_TICKET_DISCRIMINATOR = Uint8Array.from([11, 213, 240, 45, 246, 35, 44, 162]);
const BUY_RESALE_TICKET_DISCRIMINATOR = Uint8Array.from([233, 0, 254, 239, 181, 48, 46, 209]);
const CANCEL_LISTING_DISCRIMINATOR = Uint8Array.from([41, 183, 50, 232, 230, 233, 157, 70]);
const EXPIRE_LISTING_DISCRIMINATOR = Uint8Array.from([206, 60, 47, 146, 232, 175, 14, 182]);

function encodeU16(value: number) {
  const out = new Uint8Array(2);
  new DataView(out.buffer).setUint16(0, value, true);
  return out;
}

function encodeU32(value: number) {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, value, true);
  return out;
}

function encodeU64(value: bigint) {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, value, true);
  return out;
}

function encodeI64(value: number) {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigInt64(0, BigInt(value), true);
  return out;
}

function concatBytes(...arrays: Uint8Array[]) {
  const total = arrays.reduce((sum, a) => sum + a.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  arrays.forEach((a) => {
    out.set(a, offset);
    offset += a.length;
  });
  return out;
}

function formatSol(lamports?: bigint | null): string {
  if (!lamports) return "0.0000";
  return (Number(lamports) / 1_000_000_000).toFixed(4);
}

export function SecondarySalePage() {
  const { connection } = useConnection();
  const { publicKey, signTransaction } = useWallet();

  const [organizerId, setOrganizerId] = useState("");
  const [eventId, setEventId] = useState("");
  const [classId, setClassId] = useState("1");
  const [ticketId, setTicketId] = useState("1");
  const [listingId, setListingId] = useState("");
  const [listPriceSol, setListPriceSol] = useState("0.25");
  const [listExpiresAt, setListExpiresAt] = useState(new Date(Date.now() + 24 * 3600 * 1000).toISOString().slice(0, 16));
  const [buyMaxPriceSol, setBuyMaxPriceSol] = useState("0.25");

  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [signature, setSignature] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [listingRead, setListingRead] = useState<unknown | null>(null);

  const parsed = useMemo(() => {
    const classNum = Number(classId);
    const ticketNum = Number(ticketId);
    return {
      classNum,
      ticketNum,
      listPriceLamports: BigInt(Math.round(Number(listPriceSol) * 1_000_000_000)),
      buyMaxLamports: BigInt(Math.round(Number(buyMaxPriceSol) * 1_000_000_000)),
      expiresAt: Math.floor(new Date(listExpiresAt).getTime() / 1000),
    };
  }, [buyMaxPriceSol, classId, listExpiresAt, listPriceSol, ticketId]);

  useEffect(() => {
    if (!publicKey || organizerId) return;
    const programId = getProgramId();
    const [derived] = PublicKey.findProgramAddressSync([Buffer.from("organizer"), publicKey.toBuffer()], programId);
    setOrganizerId(derived.toBase58());
  }, [organizerId, publicKey]);

  const deriveAccounts = () => {
    const programId = getProgramId();
    const eventKey = new PublicKey(eventId);
    const organizerKey = new PublicKey(organizerId);
    const [protocolConfig] = PublicKey.findProgramAddressSync([Buffer.from("protocol-config")], programId);
    const [ticketClass] = PublicKey.findProgramAddressSync(
      [Buffer.from("ticket-class"), eventKey.toBuffer(), Buffer.from(encodeU16(parsed.classNum))],
      programId,
    );
    const [ticket] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("ticket"),
        eventKey.toBuffer(),
        Buffer.from(encodeU16(parsed.classNum)),
        Buffer.from(encodeU32(parsed.ticketNum)),
      ],
      programId,
    );
    const [listing] = PublicKey.findProgramAddressSync([Buffer.from("listing"), ticket.toBuffer()], programId);
    const [resalePolicy] = PublicKey.findProgramAddressSync(
      [Buffer.from("resale-policy"), eventKey.toBuffer(), Buffer.from(encodeU16(parsed.classNum))],
      programId,
    );
    const [complianceRegistry] = PublicKey.findProgramAddressSync(
      [Buffer.from("compliance-registry"), eventKey.toBuffer()],
      programId,
    );
    return {
      programId,
      organizerKey,
      eventKey,
      protocolConfig,
      ticketClass,
      ticket,
      listing,
      resalePolicy,
      complianceRegistry,
    };
  };

  const signInstruction = async (ix: TransactionInstruction) => {
    if (!publicKey || !signTransaction) throw new Error("Connect wallet first.");
    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    const msg = new TransactionMessage({
      payerKey: publicKey,
      recentBlockhash: blockhash,
      instructions: [ix],
    }).compileToV0Message();
    const tx = new VersionedTransaction(msg);
    const signed = await signTransaction(tx);
    return Buffer.from(signed.serialize()).toString("base64");
  };

  const submitSecondary = async (
    action: "list" | "buy" | "cancel" | "expire",
    simulate: boolean,
  ) => {
    setStatus("pending");
    setError(null);
    setSignature(null);
    setLabel(simulate ? `Simulating ${action}...` : `Submitting ${action}...`);

    try {
      if (!publicKey) throw new Error("Connect wallet first.");
      if (!eventId || !organizerId) throw new Error("Organizer ID and Event ID are required.");
      if (!Number.isFinite(parsed.classNum) || parsed.classNum <= 0) throw new Error("Invalid class id.");
      if (!Number.isFinite(parsed.ticketNum) || parsed.ticketNum <= 0) throw new Error("Invalid ticket id.");

      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");

      const {
        programId,
        organizerKey,
        eventKey,
        protocolConfig,
        ticketClass,
        ticket,
        listing,
        resalePolicy,
        complianceRegistry,
      } = deriveAccounts();

      setListingId(listing.toBase58());

      let ix: TransactionInstruction;
      if (action === "list") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerKey, isSigner: false, isWritable: false },
            { pubkey: eventKey, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: resalePolicy, isSigner: false, isWritable: false },
            { pubkey: ticket, isSigner: false, isWritable: true },
            { pubkey: listing, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(
            concatBytes(
              LIST_TICKET_DISCRIMINATOR,
              encodeU16(parsed.classNum),
              encodeU32(parsed.ticketNum),
              encodeU64(parsed.listPriceLamports),
              encodeI64(parsed.expiresAt),
            ),
          ),
        });
      } else if (action === "buy") {
        const listingInfo = await connection.getAccountInfo(listing, "confirmed");
        if (!listingInfo) throw new Error("Listing account not found.");
        const listingState = decodeListing(Buffer.from(listingInfo.data));

        const policyInfo = await connection.getAccountInfo(resalePolicy, "confirmed");
        if (!policyInfo) throw new Error("Resale policy account not found.");
        const royaltyVault = decodeResalePolicyRoyaltyVault(Buffer.from(policyInfo.data));

        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerKey, isSigner: false, isWritable: false },
            { pubkey: eventKey, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: resalePolicy, isSigner: false, isWritable: false },
            { pubkey: ticket, isSigner: false, isWritable: true },
            { pubkey: listing, isSigner: false, isWritable: true },
            { pubkey: listingState.seller, isSigner: false, isWritable: true },
            { pubkey: royaltyVault, isSigner: false, isWritable: true },
            { pubkey: complianceRegistry, isSigner: false, isWritable: false },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(
            concatBytes(
              BUY_RESALE_TICKET_DISCRIMINATOR,
              encodeU16(parsed.classNum),
              encodeU32(parsed.ticketNum),
              encodeU64(parsed.buyMaxLamports),
            ),
          ),
        });
      } else if (action === "cancel") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerKey, isSigner: false, isWritable: false },
            { pubkey: eventKey, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: ticket, isSigner: false, isWritable: false },
            { pubkey: listing, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(CANCEL_LISTING_DISCRIMINATOR, encodeU16(parsed.classNum), encodeU32(parsed.ticketNum)),
          ),
        });
      } else {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerKey, isSigner: false, isWritable: false },
            { pubkey: eventKey, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: ticket, isSigner: false, isWritable: false },
            { pubkey: listing, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(EXPIRE_LISTING_DISCRIMINATOR, encodeU16(parsed.classNum), encodeU32(parsed.ticketNum)),
          ),
        });
      }

      const txBase64 = await signInstruction(ix);
      const payload = {
        organizer_id: organizerKey.toBase58(),
        event_id: eventKey.toBase58(),
        class_id: ticketClass.toBase58(),
        ticket_id: ticket.toBase58(),
        listing_id: listing.toBase58(),
        transaction_base64: txBase64,
      };

      if (simulate) {
        const sim =
          action === "list"
            ? await secondarySaleApi.simulateListTicket(session.token, payload)
            : action === "buy"
              ? await secondarySaleApi.simulateBuyResaleTicket(session.token, payload)
              : action === "cancel"
                ? await secondarySaleApi.simulateCancelListing(session.token, payload)
                : await secondarySaleApi.simulateExpireListing(session.token, payload);
        if (sim.err) {
          throw new Error(JSON.stringify(sim.err));
        }
        setStatus("confirmed");
        setLabel(`Simulation ok: ${action}`);
        return;
      }

      const res =
        action === "list"
          ? await secondarySaleApi.listTicket(session.token, payload)
          : action === "buy"
            ? await secondarySaleApi.buyResaleTicket(session.token, payload)
            : action === "cancel"
              ? await secondarySaleApi.cancelListing(session.token, payload)
              : await secondarySaleApi.expireListing(session.token, payload);
      setStatus("confirmed");
      setLabel(`Confirmed: ${action}`);
      setSignature(res.signature);
    } catch (submitError) {
      const message =
        submitError instanceof ApiError
          ? submitError.message
          : submitError instanceof Error
            ? submitError.message
            : "Secondary sale action failed.";
      setStatus("failed");
      setLabel("Secondary sale action failed.");
      setError(message);
    }
  };

  const readListing = async () => {
    setStatus("pending");
    setLabel("Loading listing...");
    setError(null);
    setSignature(null);

    try {
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      if (!listingId.trim()) throw new Error("Listing ID is required.");
      const response = await secondarySaleApi.getListing(session.token, listingId.trim());
      setListingRead(response.listing ?? null);
      setStatus("confirmed");
      setLabel("Listing loaded.");
    } catch (readError) {
      const message =
        readError instanceof ApiError
          ? readError.message
          : readError instanceof Error
            ? readError.message
            : "Could not load listing.";
      setStatus("failed");
      setLabel("Listing read failed.");
      setError(message);
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>Secondary Sale</h1>
          <p>Dedicated flow for resale list/buy/cancel/expire + listing read.</p>
          <div className={styles.links}>
            <Link href="/dashboard">Back to Dashboard</Link>
            {listingId ? <Link href={`/secondary-sale/listings/${encodeURIComponent(listingId)}`}>Open Listing Detail</Link> : null}
          </div>
        </section>

        <section className={styles.card}>
          <h2>Context</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Organizer ID (PDA)</span>
              <input value={organizerId} onChange={(e) => setOrganizerId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Event ID (PDA)</span>
              <input value={eventId} onChange={(e) => setEventId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Class ID (u16)</span>
              <input value={classId} onChange={(e) => setClassId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Ticket ID (u32)</span>
              <input value={ticketId} onChange={(e) => setTicketId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Listing ID (PDA)</span>
              <input value={listingId} onChange={(e) => setListingId(e.target.value)} />
            </label>
          </div>
        </section>

        <section className={styles.card}>
          <h2>List Ticket</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Ask Price (SOL)</span>
              <input value={listPriceSol} onChange={(e) => setListPriceSol(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Expires At</span>
              <input type="datetime-local" value={listExpiresAt} onChange={(e) => setListExpiresAt(e.target.value)} />
            </label>
          </div>
          <div className={styles.actions}>
            <button onClick={() => void submitSecondary("list", true)}>Sim List</button>
            <button onClick={() => void submitSecondary("list", false)}>Submit List</button>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Buy Listing</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Max Price (SOL)</span>
              <input value={buyMaxPriceSol} onChange={(e) => setBuyMaxPriceSol(e.target.value)} />
            </label>
          </div>
          <div className={styles.actions}>
            <button onClick={() => void submitSecondary("buy", true)}>Sim Buy</button>
            <button onClick={() => void submitSecondary("buy", false)}>Submit Buy</button>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Cancel / Expire</h2>
          <div className={styles.actions}>
            <button onClick={() => void submitSecondary("cancel", true)}>Sim Cancel</button>
            <button onClick={() => void submitSecondary("cancel", false)}>Submit Cancel</button>
            <button onClick={() => void submitSecondary("expire", true)}>Sim Expire</button>
            <button onClick={() => void submitSecondary("expire", false)}>Submit Expire</button>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Listing Read</h2>
          <div className={styles.actions}>
            <button onClick={() => void readListing()}>Get Listing</button>
          </div>
          {listingRead ? <pre className={styles.pre}>{JSON.stringify(listingRead, null, 2)}</pre> : null}
          <p className={styles.caption}>
            Price preview: {formatSol(parsed.listPriceLamports)} SOL
          </p>
        </section>

        <TxStatusCard status={status} label={label} signature={signature} error={error} />
      </main>
      <Footer />
    </div>
  );
}
