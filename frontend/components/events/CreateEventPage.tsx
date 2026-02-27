"use client";

import Link from "next/link";
import { useState } from "react";
import { eventApi, organizerApi, relayApi, ticketClassApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { upsertCachedClasses, upsertCachedEvent } from "@/lib/eventsCache";
import { hasOrganizerScope, readAuthSession } from "@/lib/session";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import styles from "./CreateEventPage.module.css";

type TicketClassForm = {
  id: string;
  name: string;
  priceSol: string;
  supply: string;
};

const CREATE_EVENT_DISCRIMINATOR = Uint8Array.from([49, 219, 29, 203, 22, 98, 100, 87]);
const CREATE_TICKET_CLASS_DISCRIMINATOR = Uint8Array.from([18, 184, 8, 230, 232, 103, 137, 150]);
const CREATE_ORGANIZER_DISCRIMINATOR = Uint8Array.from([200, 214, 58, 143, 4, 114, 99, 4]);
const FALLBACK_RELAYER_PUBKEY = "9yN2JURFCKjrh6PMBbA5sspgrXn12xTbpNrKXi1gtePZ";

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

function encodeI64(value: number) {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigInt64(0, BigInt(value), true);
  return out;
}

function encodeU64(value: bigint) {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, value, true);
  return out;
}

function encodeString(value: string) {
  const bytes = new TextEncoder().encode(value);
  return concatBytes(encodeU32(bytes.length), bytes);
}

function encodeBool(value: boolean) {
  return Uint8Array.from([value ? 1 : 0]);
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

function toUnixSeconds(dateTimeLocal: string) {
  return Math.floor(new Date(dateTimeLocal).getTime() / 1000);
}

function safeEventIdFromNow(): bigint {
  const base = BigInt(Date.now());
  const rand = BigInt(Math.floor(Math.random() * 999));
  return base * 1000n + rand;
}

async function waitForAccount(
  connection: ReturnType<typeof useConnection>["connection"],
  pubkey: PublicKey,
  attempts = 6,
  delayMs = 1200,
) {
  for (let i = 0; i < attempts; i += 1) {
    const info = await connection.getAccountInfo(pubkey, "confirmed");
    if (info) return true;
    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }
  return false;
}

export function CreateEventPage() {
  const { connection } = useConnection();
  const { publicKey, signTransaction } = useWallet();

  const [eventName, setEventName] = useState("");
  const [startDateTime, setStartDateTime] = useState("");
  const [endDateTime, setEndDateTime] = useState("");
  const [venue, setVenue] = useState("");
  const [description, setDescription] = useState("");
  const [ticketClasses, setTicketClasses] = useState<TicketClassForm[]>([
    { id: crypto.randomUUID(), name: "General", priceSol: "0.25", supply: "100" },
  ]);

  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [txStatus, setTxStatus] = useState<TxStatus>("idle");
  const [txLabel, setTxLabel] = useState("");
  const [txSignature, setTxSignature] = useState<string | null>(null);
  const [txError, setTxError] = useState<string | null>(null);

  const updateClass = (id: string, patch: Partial<TicketClassForm>) => {
    setTicketClasses((prev) => prev.map((item) => (item.id === id ? { ...item, ...patch } : item)));
  };

  const addClass = () => {
    setTicketClasses((prev) => [
      ...prev,
      { id: crypto.randomUUID(), name: "", priceSol: "", supply: "" },
    ]);
  };

  const removeClass = (id: string) => {
    setTicketClasses((prev) => (prev.length > 1 ? prev.filter((item) => item.id !== id) : prev));
  };

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setMessage(null);
    setError(null);
    setTxStatus("idle");
    setTxLabel("");
    setTxSignature(null);
    setTxError(null);

    try {
      if (!publicKey || !signTransaction) {
        throw new Error("Connect a wallet that supports transaction signing.");
      }

      const token = localStorage.getItem("ticketing_access_token");
      if (!token) {
        throw new Error("Not authenticated. Sign in first.");
      }
      const session = readAuthSession();
      if (!session.wallet || session.wallet !== publicKey.toBase58()) {
        throw new Error("Authenticated wallet mismatch. Reconnect and sign in again.");
      }

      if (!eventName.trim() || !venue.trim() || !startDateTime || !endDateTime) {
        throw new Error("Fill all required event fields.");
      }

      const startTs = toUnixSeconds(startDateTime);
      const endTs = toUnixSeconds(endDateTime);
      const nowTs = Math.floor(Date.now() / 1000);
      const salesStartTs = Math.min(nowTs, startTs);
      const lockTs = startTs;

      if (startTs >= endTs) {
        throw new Error("End Date/Time must be after Start Date/Time.");
      }

      const normalizedClasses = ticketClasses.map((item, index) => {
        const priceSol = Number(item.priceSol);
        const supply = Number(item.supply);
        if (!item.name.trim()) {
          throw new Error(`Ticket class ${index + 1}: name is required.`);
        }
        if (!Number.isFinite(priceSol) || priceSol < 0) {
          throw new Error(`Ticket class ${index + 1}: invalid price.`);
        }
        if (!Number.isInteger(supply) || supply <= 0) {
          throw new Error(`Ticket class ${index + 1}: supply must be positive integer.`);
        }
        const lamports = Math.round(priceSol * LAMPORTS_PER_SOL);
        return {
          classId: index + 1,
          name: item.name.trim(),
          supply,
          lamports,
        };
      });

      const capacity = normalizedClasses.reduce((sum, item) => sum + item.supply, 0);
      if (capacity <= 0) {
        throw new Error("Total capacity must be greater than 0.");
      }

      const programId = new PublicKey(
        process.env.NEXT_PUBLIC_PROGRAM_ID ?? "Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv",
      );
      const relayerPubkeyRaw =
        process.env.NEXT_PUBLIC_RELAYER_PUBKEY ?? FALLBACK_RELAYER_PUBKEY;
      const relayerPubkey = new PublicKey(relayerPubkeyRaw);

      const [protocolConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from("protocol-config")],
        programId,
      );

      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      if (!hasOrganizerScope(session.organizerScopes, organizerProfile.toBase58())) {
        throw new Error("No organizer scope for this wallet. Request organizer access first.");
      }

      // Auto-onboard organizer profile if missing, so first-time creators are not blocked.
      const organizerAccount = await connection.getAccountInfo(organizerProfile, "confirmed");
      if (!organizerAccount) {
        const createOrganizerData = concatBytes(
          CREATE_ORGANIZER_DISCRIMINATOR,
          encodeString(""),
          publicKey.toBytes(),
        );
        const createOrganizerIxDirect = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(createOrganizerData),
        });
        const createOrganizerIxSponsored = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: relayerPubkey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(createOrganizerData),
        });

        const organizerBlockhash = await connection.getLatestBlockhash("confirmed");
        const organizerDirectMsg = new TransactionMessage({
          payerKey: publicKey,
          recentBlockhash: organizerBlockhash.blockhash,
          instructions: [createOrganizerIxDirect],
        }).compileToV0Message();
        const organizerDirectTx = new VersionedTransaction(organizerDirectMsg);
        const signedOrganizerDirectTx = await signTransaction(organizerDirectTx);
        const organizerDirectBase64 = Buffer.from(signedOrganizerDirectTx.serialize()).toString("base64");

        const organizerSponsoredMsg = new TransactionMessage({
          payerKey: relayerPubkey,
          recentBlockhash: organizerBlockhash.blockhash,
          instructions: [createOrganizerIxSponsored],
        }).compileToV0Message();
        const organizerSponsoredTx = new VersionedTransaction(organizerSponsoredMsg);
        const signedOrganizerSponsoredTx = await signTransaction(organizerSponsoredTx);
        const organizerSponsoredBase64 = Buffer.from(signedOrganizerSponsoredTx.serialize()).toString("base64");

        try {
          setTxStatus("pending");
          setTxLabel("Simulating organizer profile...");
          setTxSignature(null);
          setTxError(null);
          const sim = await organizerApi.simulateCreateOrganizer(token, {
            organizer_id: organizerProfile.toBase58(),
            transaction_base64: organizerDirectBase64,
          });
          if (sim.err) {
            throw new Error(`Create organizer simulation failed: ${JSON.stringify(sim.err)}`);
          }

          setTxLabel("Submitting organizer profile via /organizers...");
          await organizerApi.createOrganizer(token, {
            organizer_id: organizerProfile.toBase58(),
            transaction_base64: organizerDirectBase64,
            max_retries: 20,
            timeout_ms: 120_000,
            poll_ms: 2_000,
          });
          setTxStatus("confirmed");
          setTxLabel("Organizer profile confirmed.");
        } catch (organizerSubmitErr) {
          const endpointMissing =
            organizerSubmitErr instanceof ApiError &&
            (organizerSubmitErr.status === 404 || organizerSubmitErr.status === 405);
          if (!endpointMissing) {
            throw organizerSubmitErr;
          }
          await relayApi.submitViaRelayer(token, {
            transaction_base64: organizerSponsoredBase64,
            expected_instructions: ["create_organizer"],
            max_retries: 20,
            timeout_ms: 120_000,
            poll_ms: 2_000,
          });
          const landed = await waitForAccount(connection, organizerProfile);
          if (!landed) {
            throw organizerSubmitErr;
          }
          setTxStatus("confirmed");
          setTxLabel("Organizer profile confirmed via relay fallback.");
        }
      }

      const eventId = safeEventIdFromNow();
      const eventIdLe = encodeU64(eventId);

      const [eventAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from("event"), organizerProfile.toBuffer(), Buffer.from(eventIdLe)],
        programId,
      );

      const createEventData = concatBytes(
        CREATE_EVENT_DISCRIMINATOR,
        eventIdLe,
        encodeString(eventName.trim()),
        encodeString(venue.trim()),
        encodeI64(startTs),
        encodeI64(endTs),
        encodeI64(salesStartTs),
        encodeI64(lockTs),
        encodeU32(capacity),
      );

      const createEventIxDirect = new TransactionInstruction({
        programId,
        keys: [
          { pubkey: publicKey, isSigner: true, isWritable: true },
          { pubkey: publicKey, isSigner: true, isWritable: false },
          { pubkey: protocolConfig, isSigner: false, isWritable: false },
          { pubkey: organizerProfile, isSigner: false, isWritable: false },
          { pubkey: eventAccount, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(createEventData),
      });

      const createEventIxSponsored = new TransactionInstruction({
        programId,
        keys: [
          { pubkey: relayerPubkey, isSigner: true, isWritable: true },
          { pubkey: publicKey, isSigner: true, isWritable: false },
          { pubkey: protocolConfig, isSigner: false, isWritable: false },
          { pubkey: organizerProfile, isSigner: false, isWritable: false },
          { pubkey: eventAccount, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(createEventData),
      });

      const { blockhash: directCreateBlockhash } = await connection.getLatestBlockhash("confirmed");
      const createEventDirectMsg = new TransactionMessage({
        payerKey: publicKey,
        recentBlockhash: directCreateBlockhash,
        instructions: [createEventIxDirect],
      }).compileToV0Message();
      const createEventDirectTx = new VersionedTransaction(createEventDirectMsg);
      const signedCreateEventDirectTx = await signTransaction(createEventDirectTx);
      const createEventDirectBase64 = Buffer.from(signedCreateEventDirectTx.serialize()).toString(
        "base64",
      );

      let createEventSubmitted = false;
      try {
        setTxStatus("pending");
        setTxLabel("Simulating create event...");
        const sim = await eventApi.simulateCreateEvent(token, {
          organizer_id: organizerProfile.toBase58(),
          event_id: eventAccount.toBase58(),
          transaction_base64: createEventDirectBase64,
        });
        if (sim.err) {
          throw new Error(`Create event simulation failed: ${JSON.stringify(sim.err)}`);
        }

        setTxLabel("Submitting create event via /events...");
        const createResp = await eventApi.createEvent(token, {
          organizer_id: organizerProfile.toBase58(),
          event_id: eventAccount.toBase58(),
          transaction_base64: createEventDirectBase64,
          max_retries: 20,
          timeout_ms: 120_000,
          poll_ms: 2_000,
        });
        createEventSubmitted = true;
        setTxStatus("confirmed");
        setTxLabel("Event account confirmed.");
        setTxSignature(createResp.signature);
      } catch (directCreateError) {
        const directErrMsg =
          directCreateError instanceof Error ? directCreateError.message : String(directCreateError);
        setTxStatus("pending");
        setTxLabel(`Falling back to sponsored create path... (${directErrMsg})`);
      }

      if (!createEventSubmitted) {
        const { blockhash } = await connection.getLatestBlockhash("confirmed");
        const createEventMsg = new TransactionMessage({
          payerKey: relayerPubkey,
          recentBlockhash: blockhash,
          instructions: [createEventIxSponsored],
        }).compileToV0Message();
        const createEventTx = new VersionedTransaction(createEventMsg);
        const signedCreateEventTx = await signTransaction(createEventTx);
        const createEventBase64 = Buffer.from(signedCreateEventTx.serialize()).toString("base64");

        try {
          await relayApi.submitViaRelayer(token, {
            transaction_base64: createEventBase64,
            expected_instructions: ["create_event"],
            max_retries: 20,
            timeout_ms: 120_000,
            poll_ms: 2_000,
          });
          setTxStatus("confirmed");
          setTxLabel("Event account confirmed.");
        } catch (eventSubmitErr) {
          const eventErrMsg =
            eventSubmitErr instanceof Error ? eventSubmitErr.message : String(eventSubmitErr);
          if (eventErrMsg.includes("transaction not confirmed")) {
            const landed = await waitForAccount(connection, eventAccount);
            if (!landed) {
              throw eventSubmitErr;
            }
          } else {
            throw eventSubmitErr;
          }
        }
      }

      for (const cls of normalizedClasses) {
        const classSaleStartTs = Math.min(salesStartTs, endTs - 60);
        const classSaleEndTs = Math.max(classSaleStartTs + 60, endTs);

        const [ticketClassPda] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("ticket-class"),
            eventAccount.toBuffer(),
            Buffer.from(encodeU16(cls.classId)),
          ],
          programId,
        );

        const createClassData = concatBytes(
          CREATE_TICKET_CLASS_DISCRIMINATOR,
          encodeU16(cls.classId),
          encodeString(cls.name),
          encodeU32(cls.supply),
          encodeU32(0),
          encodeU64(BigInt(cls.lamports)),
          encodeI64(classSaleStartTs),
          encodeI64(classSaleEndTs),
          encodeU16(10),
          encodeBool(true),
          encodeBool(true),
          publicKey.toBytes(),
          encodeU16(0),
        );

        const createClassIxDirect = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: ticketClassPda, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(createClassData),
        });

        const createClassIxSponsored = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: relayerPubkey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: ticketClassPda, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(createClassData),
        });

        const classDirectBlockhash = await connection.getLatestBlockhash("confirmed");
        const classDirectMsg = new TransactionMessage({
          payerKey: publicKey,
          recentBlockhash: classDirectBlockhash.blockhash,
          instructions: [createClassIxDirect],
        }).compileToV0Message();
        const classDirectTx = new VersionedTransaction(classDirectMsg);
        const signedClassDirectTx = await signTransaction(classDirectTx);
        const classDirectTxBase64 = Buffer.from(signedClassDirectTx.serialize()).toString("base64");

        let createdViaEndpoint = false;
        try {
          setTxStatus("pending");
          setTxLabel(`Simulating ticket class ${cls.classId}...`);
          const sim = await ticketClassApi.simulateCreateTicketClass(token, {
            organizer_id: organizerProfile.toBase58(),
            event_id: eventAccount.toBase58(),
            class_id: ticketClassPda.toBase58(),
            transaction_base64: classDirectTxBase64,
          });
          if (sim.err) {
            throw new Error(`Ticket class simulation failed: ${JSON.stringify(sim.err)}`);
          }

          setTxLabel(`Submitting ticket class ${cls.classId} via /ticket-classes...`);
          const createResp = await ticketClassApi.createTicketClass(token, {
            organizer_id: organizerProfile.toBase58(),
            event_id: eventAccount.toBase58(),
            class_id: ticketClassPda.toBase58(),
            transaction_base64: classDirectTxBase64,
            max_retries: 20,
            timeout_ms: 120_000,
            poll_ms: 2_000,
          });
          createdViaEndpoint = true;
          setTxStatus("confirmed");
          setTxLabel(`Ticket class ${cls.classId} confirmed.`);
          setTxSignature(createResp.signature);
        } catch (classDirectError) {
          setTxStatus("pending");
          setTxLabel(
            `Falling back to sponsored ticket class ${cls.classId}... (${
              classDirectError instanceof Error ? classDirectError.message : String(classDirectError)
            })`,
          );
        }

        if (!createdViaEndpoint) {
          const classBlockhash = await connection.getLatestBlockhash("confirmed");
          const classMsg = new TransactionMessage({
            payerKey: relayerPubkey,
            recentBlockhash: classBlockhash.blockhash,
            instructions: [createClassIxSponsored],
          }).compileToV0Message();
          const classTx = new VersionedTransaction(classMsg);
          const signedClassTx = await signTransaction(classTx);
          const classTxBase64 = Buffer.from(signedClassTx.serialize()).toString("base64");

          try {
            setTxStatus("pending");
            setTxLabel(`Submitting ticket class ${cls.classId}...`);
            setTxError(null);
            const classResult = await relayApi.submitViaRelayer(token, {
              transaction_base64: classTxBase64,
              expected_instructions: ["create_ticket_class"],
              max_retries: 20,
              timeout_ms: 120_000,
              poll_ms: 2_000,
            });
            setTxStatus("confirmed");
            setTxLabel(`Ticket class ${cls.classId} confirmed.`);
            setTxSignature(classResult.signature);
          } catch (classSubmitErr) {
            const classErrMsg =
              classSubmitErr instanceof Error ? classSubmitErr.message : String(classSubmitErr);
            if (classErrMsg.includes("transaction not confirmed")) {
              const landed = await waitForAccount(connection, ticketClassPda);
              if (!landed) {
                throw classSubmitErr;
              }
            } else {
              throw classSubmitErr;
            }
          }
        }

        upsertCachedClasses([
          {
            classId: cls.classId,
            classPda: ticketClassPda.toBase58(),
            eventId: eventId.toString(),
            eventPda: eventAccount.toBase58(),
            organizerId: organizerProfile.toBase58(),
            name: cls.name,
            supply: cls.supply,
            priceLamports: cls.lamports,
            stakeholderWallet: publicKey.toBase58(),
            stakeholderBps: 0,
          },
        ]);
      }

      if (description.trim()) {
        localStorage.setItem(`event:${eventId.toString()}:description`, description.trim());
      }

      upsertCachedEvent({
        eventId: eventId.toString(),
        eventPda: eventAccount.toBase58(),
        organizerId: organizerProfile.toBase58(),
        name: eventName.trim(),
        venue: venue.trim(),
        startsAtEpoch: startTs,
        endsAtEpoch: endTs,
        updatedAtEpoch: Math.floor(Date.now() / 1000),
      });

      setMessage("Event and ticket classes submitted on-chain successfully.");
      setEventName("");
      setStartDateTime("");
      setEndDateTime("");
      setVenue("");
      setDescription("");
      setTicketClasses([{ id: crypto.randomUUID(), name: "General", priceSol: "0.25", supply: "100" }]);
    } catch (submitError) {
      const toMessage = (raw: string) => {
        if (raw.includes("AccountNotInitialized") && raw.includes("protocol_config")) {
          return "Protocol is not initialized on-chain yet. Run bootstrap first: initialize_protocol, then create_organizer for your wallet.";
        }
        if (raw.includes("AccountNotInitialized") && raw.includes("organizer_profile")) {
          return "Organizer profile is missing on-chain for this wallet. Create organizer first, then retry event creation.";
        }
        return raw;
      };

      if (submitError instanceof ApiError) {
        const parsed = toMessage(submitError.message);
        setError(parsed);
        setTxStatus("failed");
        setTxError(parsed);
      } else {
        const fallback =
          submitError instanceof Error ? submitError.message : "Could not create event.";
        const parsed = toMessage(fallback);
        setError(parsed);
        setTxStatus("failed");
        setTxError(parsed);
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>Create Event</h1>
          <p>Business form to on-chain event and ticket class creation.</p>
        </section>

        <form className={styles.form} onSubmit={onSubmit}>
          <TxStatusCard status={txStatus} label={txLabel} signature={txSignature} error={txError} />

          <label className={styles.field}>
            <span>Event Name</span>
            <input required value={eventName} onChange={(e) => setEventName(e.target.value)} />
          </label>

          <div className={styles.grid2}>
            <label className={styles.field}>
              <span>Start Date/Time</span>
              <input
                required
                type="datetime-local"
                value={startDateTime}
                onChange={(e) => setStartDateTime(e.target.value)}
              />
            </label>
            <label className={styles.field}>
              <span>End Date/Time</span>
              <input
                required
                type="datetime-local"
                value={endDateTime}
                onChange={(e) => setEndDateTime(e.target.value)}
              />
            </label>
          </div>

          <label className={styles.field}>
            <span>Venue</span>
            <input required value={venue} onChange={(e) => setVenue(e.target.value)} />
          </label>

          <label className={styles.field}>
            <span>Description</span>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              placeholder="Stored off-chain for now"
            />
          </label>

          <section className={styles.ticketSection}>
            <div className={styles.ticketHeader}>
              <h2>Ticket classes</h2>
              <button type="button" onClick={addClass} className={styles.secondaryBtn}>
                Add class
              </button>
            </div>

            <div className={styles.ticketList}>
              {ticketClasses.map((item, index) => (
                <div key={item.id} className={styles.ticketCard}>
                  <label className={styles.field}>
                    <span>Name</span>
                    <input
                      required
                      value={item.name}
                      onChange={(e) => updateClass(item.id, { name: e.target.value })}
                      placeholder={`Class ${index + 1}`}
                    />
                  </label>

                  <div className={styles.grid2}>
                    <label className={styles.field}>
                      <span>Price (SOL)</span>
                      <input
                        required
                        type="number"
                        min="0"
                        step="0.000001"
                        value={item.priceSol}
                        onChange={(e) => updateClass(item.id, { priceSol: e.target.value })}
                      />
                    </label>

                    <label className={styles.field}>
                      <span>Supply</span>
                      <input
                        required
                        type="number"
                        min="1"
                        step="1"
                        value={item.supply}
                        onChange={(e) => updateClass(item.id, { supply: e.target.value })}
                      />
                    </label>
                  </div>

                  <button type="button" onClick={() => removeClass(item.id)} className={styles.removeBtn}>
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </section>

          <div className={styles.actions}>
            <button type="submit" disabled={busy}>
              {busy ? "Submitting..." : "Create Event On-Chain"}
            </button>
            <Link href="/dashboard">Back to Dashboard</Link>
          </div>

          {message ? <p className={styles.success}>{message}</p> : null}
          {error ? <p className={styles.error}>{error}</p> : null}
        </form>
      </main>

      <Footer />
    </div>
  );
}
