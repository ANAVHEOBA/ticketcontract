"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import { eventApi, organizerApi, primarySaleApi, relayApi, ticketClassApi, ticketStateApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { getCachedClassesByEvent, getCachedEvents } from "@/lib/eventsCache";
import {
  decodeOrganizerPayoutWallet,
  decodeProtocolFeeVault,
  decodeTicketClass,
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
import styles from "./EventDetailPage.module.css";

type EventRecord = {
  event_id: string;
  organizer_id: string;
  name?: string | null;
  status?: string | null;
  starts_at_epoch?: number | null;
  ends_at_epoch?: number | null;
};

type DisplayClass = {
  classPda: string;
  classId: number;
  name: string;
  soldSupply: number;
  totalSupply?: number;
  priceLamports: bigint;
  stakeholderWallet: string;
};

type PurchaseReceipt = {
  signature: string;
  ticketPda: string;
  classId: number;
  priceLamports: bigint;
};

type ClassReadRecord = {
  class_id: string;
  event_id: string;
  organizer_id: string;
  name?: string | null;
  status?: string | null;
  supply_total?: number | null;
  supply_reserved?: number | null;
  supply_sold?: number | null;
};

type ClassAnalyticsRecord = {
  class_id: string;
  event_id: string;
  organizer_id: string;
  supply_total: number;
  supply_reserved: number;
  supply_sold: number;
  supply_remaining: number;
  pacing_ratio: number;
};

type OrganizerReadRecord = {
  organizer_id: string;
  authority?: string | null;
  payout_wallet?: string | null;
  metadata_uri?: string | null;
  status?: string | null;
  compliance_flags?: number | null;
};

const BUY_TICKET_DISCRIMINATOR = Uint8Array.from([11, 24, 17, 193, 168, 116, 164, 169]);
const CREATE_ORGANIZER_DISCRIMINATOR = Uint8Array.from([200, 214, 58, 143, 4, 114, 99, 4]);
const UPDATE_ORGANIZER_DISCRIMINATOR = Uint8Array.from([243, 26, 10, 51, 155, 79, 248, 89]);
const SET_ORGANIZER_STATUS_DISCRIMINATOR = Uint8Array.from([177, 65, 254, 184, 39, 149, 89, 81]);
const SET_ORGANIZER_COMPLIANCE_FLAGS_DISCRIMINATOR = Uint8Array.from([214, 141, 170, 120, 19, 169, 70, 37]);
const SET_ORGANIZER_OPERATOR_DISCRIMINATOR = Uint8Array.from([124, 152, 63, 203, 122, 229, 73, 61]);
const ISSUE_COMP_TICKET_DISCRIMINATOR = Uint8Array.from([132, 193, 24, 49, 213, 167, 151, 116]);
const SET_TICKET_METADATA_DISCRIMINATOR = Uint8Array.from([155, 242, 41, 38, 172, 10, 140, 201]);
const TRANSITION_TICKET_STATUS_DISCRIMINATOR = Uint8Array.from([142, 95, 121, 165, 63, 221, 81, 15]);
const CREATE_TICKET_CLASS_DISCRIMINATOR = Uint8Array.from([18, 184, 8, 230, 232, 103, 137, 150]);
const UPDATE_TICKET_CLASS_DISCRIMINATOR = Uint8Array.from([64, 166, 143, 145, 6, 71, 204, 199]);
const RESERVE_INVENTORY_DISCRIMINATOR = Uint8Array.from([176, 77, 133, 114, 118, 212, 11, 21]);
const UPDATE_EVENT_DISCRIMINATOR = Uint8Array.from([70, 108, 211, 125, 171, 176, 25, 217]);
const FREEZE_EVENT_DISCRIMINATOR = Uint8Array.from([176, 154, 203, 151, 40, 111, 34, 128]);
const CANCEL_EVENT_DISCRIMINATOR = Uint8Array.from([55, 143, 36, 45, 59, 241, 89, 119]);
const PAUSE_EVENT_DISCRIMINATOR = Uint8Array.from([66, 24, 187, 127, 190, 23, 1, 190]);
const CLOSE_EVENT_DISCRIMINATOR = Uint8Array.from([117, 114, 193, 54, 49, 25, 75, 194]);
const SET_EVENT_RESTRICTIONS_DISCRIMINATOR = Uint8Array.from([171, 92, 161, 142, 247, 193, 235, 223]);
const SET_EVENT_LOYALTY_MULTIPLIER_DISCRIMINATOR = Uint8Array.from([35, 176, 141, 149, 25, 242, 164, 220]);
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

function encodeString(value: string) {
  const bytes = new TextEncoder().encode(value);
  return concatBytes(encodeU32(bytes.length), bytes);
}

function toUnixSeconds(dateTimeLocal: string) {
  return Math.floor(new Date(dateTimeLocal).getTime() / 1000);
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

function formatEpoch(epoch?: number | null): string {
  if (!epoch) return "TBD";
  return new Date(epoch * 1000).toLocaleString();
}

function formatSol(lamports: bigint): string {
  return (Number(lamports) / 1_000_000_000).toFixed(4);
}

function ticketStatusToCode(status: string): number {
  const value = status.trim().toLowerCase();
  if (value === "active") return 1;
  if (value === "checked_in") return 2;
  if (value === "refunded") return 3;
  if (value === "invalidated") return 4;
  return 0;
}

export function EventDetailPage() {
  const params = useParams<{ eventId: string }>();
  const eventKey = decodeURIComponent(params.eventId);
  const { connection } = useConnection();
  const { publicKey, signTransaction } = useWallet();

  const [event, setEvent] = useState<EventRecord | null>(null);
  const [classes, setClasses] = useState<DisplayClass[]>([]);
  const [selectedClassPda, setSelectedClassPda] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [txStatus, setTxStatus] = useState<TxStatus>("idle");
  const [txLabel, setTxLabel] = useState("");
  const [txSignature, setTxSignature] = useState<string | null>(null);
  const [txError, setTxError] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<PurchaseReceipt | null>(null);
  const [adminTxStatus, setAdminTxStatus] = useState<TxStatus>("idle");
  const [adminTxLabel, setAdminTxLabel] = useState("");
  const [adminTxSignature, setAdminTxSignature] = useState<string | null>(null);
  const [adminTxError, setAdminTxError] = useState<string | null>(null);

  const [updateTitle, setUpdateTitle] = useState("");
  const [updateVenue, setUpdateVenue] = useState("");
  const [updateStart, setUpdateStart] = useState("");
  const [updateEnd, setUpdateEnd] = useState("");
  const [updateSalesStart, setUpdateSalesStart] = useState("");
  const [updateLock, setUpdateLock] = useState("");
  const [updateCapacity, setUpdateCapacity] = useState("100");
  const [pauseFlag, setPauseFlag] = useState(false);
  const [restrictionFlags, setRestrictionFlags] = useState("0");
  const [decisionCode, setDecisionCode] = useState("0");
  const [loyaltyMultiplier, setLoyaltyMultiplier] = useState("10000");
  const [isOwner, setIsOwner] = useState(false);
  const [classRead, setClassRead] = useState<ClassReadRecord | null>(null);
  const [classAnalytics, setClassAnalytics] = useState<ClassAnalyticsRecord | null>(null);
  const [classAdminStatus, setClassAdminStatus] = useState<TxStatus>("idle");
  const [classAdminLabel, setClassAdminLabel] = useState("");
  const [classAdminSignature, setClassAdminSignature] = useState<string | null>(null);
  const [classAdminError, setClassAdminError] = useState<string | null>(null);
  const [classIdInput, setClassIdInput] = useState("1");
  const [classNameInput, setClassNameInput] = useState("");
  const [classSupplyInput, setClassSupplyInput] = useState("100");
  const [classReservedInput, setClassReservedInput] = useState("0");
  const [classPriceInput, setClassPriceInput] = useState("0.25");
  const [classSaleStartInput, setClassSaleStartInput] = useState("");
  const [classSaleEndInput, setClassSaleEndInput] = useState("");
  const [classWalletLimitInput, setClassWalletLimitInput] = useState("10");
  const [classTransferable, setClassTransferable] = useState(true);
  const [classResaleEnabled, setClassResaleEnabled] = useState(true);
  const [reserveAmountInput, setReserveAmountInput] = useState("1");
  const [compRecipientWallet, setCompRecipientWallet] = useState("");
  const [compTicketIdInput, setCompTicketIdInput] = useState("1");

  const [ticketStateStatus, setTicketStateStatus] = useState<TxStatus>("idle");
  const [ticketStateLabel, setTicketStateLabel] = useState("");
  const [ticketStateSignature, setTicketStateSignature] = useState<string | null>(null);
  const [ticketStateError, setTicketStateError] = useState<string | null>(null);
  const [ticketLookupPda, setTicketLookupPda] = useState("");
  const [ticketClassIdInput, setTicketClassIdInput] = useState("1");
  const [ticketIdInput, setTicketIdInput] = useState("1");
  const [ticketMetadataUriInput, setTicketMetadataUriInput] = useState("");
  const [ticketMetadataVersionInput, setTicketMetadataVersionInput] = useState("1");
  const [ticketNextStatus, setTicketNextStatus] = useState("checked_in");
  const [ticketReadData, setTicketReadData] = useState<unknown | null>(null);
  const [organizerTxStatus, setOrganizerTxStatus] = useState<TxStatus>("idle");
  const [organizerTxLabel, setOrganizerTxLabel] = useState("");
  const [organizerTxSignature, setOrganizerTxSignature] = useState<string | null>(null);
  const [organizerTxError, setOrganizerTxError] = useState<string | null>(null);
  const [organizerRead, setOrganizerRead] = useState<OrganizerReadRecord | null>(null);
  const [organizerMetadataUriInput, setOrganizerMetadataUriInput] = useState("");
  const [organizerPayoutWalletInput, setOrganizerPayoutWalletInput] = useState("");
  const [organizerStatusInput, setOrganizerStatusInput] = useState("1");
  const [organizerComplianceFlagsInput, setOrganizerComplianceFlagsInput] = useState("0");
  const [operatorWalletInput, setOperatorWalletInput] = useState("");
  const [operatorPermissionsInput, setOperatorPermissionsInput] = useState("0");
  const [operatorActiveInput, setOperatorActiveInput] = useState(true);

  const selectedClass = useMemo(
    () => classes.find((cls) => cls.classPda === selectedClassPda) ?? null,
    [classes, selectedClassPda],
  );

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      setError(null);

      try {
        const session = readAuthSession();
        if (!session.token) {
          setError("Sign in required.");
          return;
        }

        let eventRecord: EventRecord | null = null;
        try {
          const response = await eventApi.getEvent(session.token, eventKey);
          eventRecord = (response.event as EventRecord) ?? null;
        } catch {
          const cached = getCachedEvents().find(
            (item) => item.eventPda === eventKey || item.eventId === eventKey,
          );
          if (cached) {
            eventRecord = {
              event_id: cached.eventPda,
              organizer_id: cached.organizerId,
              name: cached.name,
              status: "Draft",
              starts_at_epoch: cached.startsAtEpoch,
              ends_at_epoch: cached.endsAtEpoch,
            };
          }
        }

        if (!eventRecord) {
          setError("Event not found.");
          return;
        }
        setEvent(eventRecord);
        setUpdateTitle(eventRecord.name ?? "");
        setUpdateVenue("");
        if (eventRecord.starts_at_epoch) {
          const startDate = new Date(eventRecord.starts_at_epoch * 1000).toISOString().slice(0, 16);
          setUpdateStart(startDate);
          setUpdateSalesStart(startDate);
          setUpdateLock(startDate);
        }
        if (eventRecord.ends_at_epoch) {
          setUpdateEnd(new Date(eventRecord.ends_at_epoch * 1000).toISOString().slice(0, 16));
        }

        const remoteClasses = await ticketClassApi.listTicketClasses(session.token, {
          event_id: eventRecord.event_id,
        });
        const cachedClasses = getCachedClassesByEvent(eventRecord.event_id);

        const byPda = new Map<string, DisplayClass>();
        for (const cached of cachedClasses) {
          byPda.set(cached.classPda, {
            classPda: cached.classPda,
            classId: cached.classId,
            name: cached.name,
            soldSupply: 0,
            totalSupply: cached.supply,
            priceLamports: BigInt(cached.priceLamports),
            stakeholderWallet: cached.stakeholderWallet,
          });
        }

        for (const raw of (remoteClasses.classes as Record<string, unknown>[]) ?? []) {
          const classPda = String(raw.class_id ?? "");
          if (!classPda) continue;
          const existing = byPda.get(classPda);
          byPda.set(classPda, {
            classPda,
            classId: existing?.classId ?? 0,
            name: String(raw.name ?? existing?.name ?? "Ticket Class"),
            soldSupply: Number(raw.supply_sold ?? existing?.soldSupply ?? 0),
            totalSupply: Number(raw.supply_total ?? existing?.totalSupply ?? 0),
            priceLamports: existing?.priceLamports ?? 0n,
            stakeholderWallet: existing?.stakeholderWallet ?? PublicKey.default.toBase58(),
          });
        }

        const enriched: DisplayClass[] = [];
        for (const value of byPda.values()) {
          try {
            const classKey = new PublicKey(value.classPda);
            const info = await connection.getAccountInfo(classKey, "confirmed");
            if (info) {
              const decoded = decodeTicketClass(Buffer.from(info.data));
              enriched.push({
                classPda: value.classPda,
                classId: decoded.classId,
                name: value.name,
                soldSupply: decoded.soldSupply,
                totalSupply: value.totalSupply,
                priceLamports: decoded.facePriceLamports,
                stakeholderWallet: decoded.stakeholderWallet.toBase58(),
              });
              continue;
            }
          } catch {
            // fallback to indexed/cache value
          }
          if (value.classId > 0) {
            enriched.push(value);
          }
        }

        setClasses(enriched);
        if (enriched[0]) {
          setSelectedClassPda(enriched[0].classPda);
        }
      } catch (loadError) {
        setError(loadError instanceof Error ? loadError.message : "Could not load event.");
      } finally {
        setLoading(false);
      }
    };

    void load();
  }, [connection, eventKey]);

  useEffect(() => {
    if (!publicKey || !event) {
      setIsOwner(false);
      return;
    }
    const programId = getProgramId();
    const [walletOrganizerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("organizer"), publicKey.toBuffer()],
      programId,
    );
    setIsOwner(walletOrganizerPda.toBase58() === event.organizer_id);
  }, [event, publicKey]);

  useEffect(() => {
    if (!publicKey) return;
    if (!organizerPayoutWalletInput) {
      setOrganizerPayoutWalletInput(publicKey.toBase58());
    }
    if (!operatorWalletInput) {
      setOperatorWalletInput(publicKey.toBase58());
    }
  }, [publicKey, operatorWalletInput, organizerPayoutWalletInput]);

  useEffect(() => {
    if (!selectedClass) return;
    setClassIdInput(String(selectedClass.classId));
    setTicketClassIdInput(String(selectedClass.classId));
    setClassNameInput(selectedClass.name);
    setClassSupplyInput(String(selectedClass.totalSupply ?? 100));
    setClassReservedInput("0");
    setClassPriceInput(formatSol(selectedClass.priceLamports));
    setClassWalletLimitInput("10");
    const now = new Date().toISOString().slice(0, 16);
    setClassSaleStartInput(now);
    setClassSaleEndInput(new Date(Date.now() + 7 * 86_400_000).toISOString().slice(0, 16));
    setCompTicketIdInput(String(selectedClass.soldSupply + 1));
    setTicketIdInput(String(selectedClass.soldSupply || 1));
  }, [selectedClass]);

  useEffect(() => {
    const loadClassRead = async () => {
      if (!selectedClassPda) {
        setClassRead(null);
        setClassAnalytics(null);
        return;
      }
      const session = readAuthSession();
      if (!session.token) return;

      try {
        const read = await ticketClassApi.getTicketClass(session.token, selectedClassPda);
        setClassRead((read.class as ClassReadRecord) ?? null);
      } catch {
        setClassRead(null);
      }

      try {
        const analytics = await ticketClassApi.getTicketClassAnalytics(session.token, selectedClassPda);
        setClassAnalytics((analytics.analytics as ClassAnalyticsRecord) ?? null);
      } catch {
        setClassAnalytics(null);
      }
    };
    void loadClassRead();
  }, [selectedClassPda]);

  useEffect(() => {
    const loadOrganizerRead = async () => {
      if (!event) return;
      const session = readAuthSession();
      if (!session.token) return;
      try {
        const read = await organizerApi.getOrganizer(session.token, event.organizer_id);
        const organizer = (read.organizer as OrganizerReadRecord) ?? null;
        setOrganizerRead(organizer);
        if (organizer?.metadata_uri) setOrganizerMetadataUriInput(String(organizer.metadata_uri));
        if (organizer?.payout_wallet) setOrganizerPayoutWalletInput(String(organizer.payout_wallet));
        if (organizer?.compliance_flags != null) {
          setOrganizerComplianceFlagsInput(String(organizer.compliance_flags));
        }
      } catch {
        setOrganizerRead(null);
      }
    };
    void loadOrganizerRead();
  }, [event]);

  const signInstructionTx = async (instruction: TransactionInstruction): Promise<string> => {
    if (!publicKey || !signTransaction) {
      throw new Error("Connect wallet first.");
    }
    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    const msg = new TransactionMessage({
      payerKey: publicKey,
      recentBlockhash: blockhash,
      instructions: [instruction],
    }).compileToV0Message();
    const tx = new VersionedTransaction(msg);
    const signed = await signTransaction(tx);
    return Buffer.from(signed.serialize()).toString("base64");
  };

  const deriveTicketPda = (eventPda: PublicKey, classId: number, ticketId: number) => {
    const programId = getProgramId();
    const [ticketPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("ticket"),
        eventPda.toBuffer(),
        Buffer.from(encodeU16(classId)),
        Buffer.from(encodeU32(ticketId)),
      ],
      programId,
    );
    return ticketPda;
  };

  const runEventEndpointAction = async (
    action: "update" | "freeze" | "cancel" | "pause" | "close" | "restrictions" | "loyalty",
    simulate: boolean,
  ) => {
    setAdminTxStatus("pending");
    setAdminTxError(null);
    setAdminTxSignature(null);
    setAdminTxLabel(simulate ? `Simulating ${action}...` : `Submitting ${action}...`);

    try {
      if (!event || !publicKey) {
        throw new Error("Event not loaded.");
      }
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");

      const programId = getProgramId();
      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      const eventAccount = new PublicKey(event.event_id);

      const mkBasePayload = (transaction_base64: string) => ({
        organizer_id: organizerProfile.toBase58(),
        event_id: eventAccount.toBase58(),
        transaction_base64,
      });

      let ix: TransactionInstruction;
      if (action === "update") {
        const startTs = toUnixSeconds(updateStart);
        const endTs = toUnixSeconds(updateEnd);
        const salesStartTs = toUnixSeconds(updateSalesStart);
        const lockTs = toUnixSeconds(updateLock);
        const capacity = Number(updateCapacity);
        const data = concatBytes(
          UPDATE_EVENT_DISCRIMINATOR,
          encodeString(updateTitle),
          encodeString(updateVenue),
          encodeI64(startTs),
          encodeI64(endTs),
          encodeI64(salesStartTs),
          encodeI64(lockTs),
          encodeU32(capacity),
        );
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(data),
        });
      } else if (action === "freeze") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(FREEZE_EVENT_DISCRIMINATOR),
        });
      } else if (action === "cancel") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(CANCEL_EVENT_DISCRIMINATOR),
        });
      } else if (action === "pause") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(concatBytes(PAUSE_EVENT_DISCRIMINATOR, Uint8Array.from([pauseFlag ? 1 : 0]))),
        });
      } else if (action === "close") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(CLOSE_EVENT_DISCRIMINATOR),
        });
      } else if (action === "restrictions") {
        const [protocolConfig] = PublicKey.findProgramAddressSync(
          [Buffer.from("protocol-config")],
          programId,
        );
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              SET_EVENT_RESTRICTIONS_DISCRIMINATOR,
              encodeU32(Number(restrictionFlags)),
              encodeU16(Number(decisionCode)),
            ),
          ),
        });
      } else {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(SET_EVENT_LOYALTY_MULTIPLIER_DISCRIMINATOR, encodeU16(Number(loyaltyMultiplier))),
          ),
        });
      }

      const txBase64 = await signInstructionTx(ix);

      if (simulate) {
        const response =
          action === "update"
            ? await eventApi.simulateUpdateEvent(session.token, mkBasePayload(txBase64))
            : action === "freeze"
              ? await eventApi.simulateFreezeEvent(session.token, mkBasePayload(txBase64))
              : action === "cancel"
                ? await eventApi.simulateCancelEvent(session.token, mkBasePayload(txBase64))
                : action === "pause"
                  ? await eventApi.simulatePauseEvent(session.token, mkBasePayload(txBase64))
                  : action === "close"
                    ? await eventApi.simulateCloseEvent(session.token, mkBasePayload(txBase64))
                    : action === "restrictions"
                      ? await eventApi.simulateSetEventRestrictions(session.token, mkBasePayload(txBase64))
                      : await eventApi.simulateSetEventLoyaltyMultiplier(session.token, mkBasePayload(txBase64));

        if (response.err) {
          throw new Error(JSON.stringify(response.err));
        }
        setAdminTxStatus("confirmed");
        setAdminTxLabel(`Simulation ok: ${action}`);
        return;
      }

      const response =
        action === "update"
          ? await eventApi.updateEvent(session.token, mkBasePayload(txBase64))
          : action === "freeze"
            ? await eventApi.freezeEvent(session.token, mkBasePayload(txBase64))
            : action === "cancel"
              ? await eventApi.cancelEvent(session.token, mkBasePayload(txBase64))
              : action === "pause"
                ? await eventApi.pauseEvent(session.token, mkBasePayload(txBase64))
                : action === "close"
                  ? await eventApi.closeEvent(session.token, mkBasePayload(txBase64))
                  : action === "restrictions"
                    ? await eventApi.setEventRestrictions(session.token, mkBasePayload(txBase64))
                    : await eventApi.setEventLoyaltyMultiplier(session.token, mkBasePayload(txBase64));

      setAdminTxStatus("confirmed");
      setAdminTxLabel(`Confirmed: ${action}`);
      setAdminTxSignature(response.signature);
    } catch (adminError) {
      const msg =
        adminError instanceof ApiError
          ? adminError.message
          : adminError instanceof Error
            ? adminError.message
            : "Action failed";
      setAdminTxStatus("failed");
      setAdminTxLabel("Event admin action failed.");
      setAdminTxError(msg);
    }
  };

  const runTicketClassAction = async (
    action: "create" | "update" | "reserve",
    simulate: boolean,
  ) => {
    setClassAdminStatus("pending");
    setClassAdminError(null);
    setClassAdminSignature(null);
    setClassAdminLabel(simulate ? `Simulating ${action} class...` : `Submitting ${action} class...`);

    try {
      if (!event || !publicKey || !signTransaction) throw new Error("Missing event context.");
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");

      const programId = getProgramId();
      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      const [protocolConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from("protocol-config")],
        programId,
      );
      const eventAccount = new PublicKey(event.event_id);
      const parsedClassId = Number(classIdInput);
      const classIdCandidate = action === "create" ? parsedClassId : selectedClass?.classId;
      if (!Number.isFinite(classIdCandidate) || classIdCandidate <= 0) {
        throw new Error("Class ID must be a positive number.");
      }
      if ((action === "update" || action === "reserve") && !selectedClass) {
        throw new Error("Select an existing class for update/reserve.");
      }
      const classId = Number(classIdCandidate);
      const [classPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("ticket-class"), eventAccount.toBuffer(), Buffer.from(encodeU16(classId))],
        programId,
      );

      const mkInput = () => {
        const lamports = BigInt(Math.round(Number(classPriceInput) * 1_000_000_000));
        return concatBytes(
          encodeString(classNameInput),
          encodeU32(Number(classSupplyInput)),
          encodeU32(Number(classReservedInput)),
          encodeU64(lamports),
          encodeI64(toUnixSeconds(classSaleStartInput)),
          encodeI64(toUnixSeconds(classSaleEndInput)),
          encodeU16(Number(classWalletLimitInput)),
          Uint8Array.from([classTransferable ? 1 : 0]),
          Uint8Array.from([classResaleEnabled ? 1 : 0]),
          publicKey.toBytes(),
          encodeU16(0),
        );
      };

      let ix: TransactionInstruction;
      if (action === "create") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: classPda, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(
            concatBytes(CREATE_TICKET_CLASS_DISCRIMINATOR, encodeU16(classId), mkInput()),
          ),
        });
      } else if (action === "update") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: classPda, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(UPDATE_TICKET_CLASS_DISCRIMINATOR, encodeU16(classId), mkInput()),
          ),
        });
      } else {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: classPda, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              RESERVE_INVENTORY_DISCRIMINATOR,
              encodeU16(classId),
              encodeU32(Number(reserveAmountInput)),
            ),
          ),
        });
      }

      const txBase64 = await signInstructionTx(ix);
      const payload = {
        organizer_id: organizerProfile.toBase58(),
        event_id: eventAccount.toBase58(),
        class_id: classPda.toBase58(),
        transaction_base64: txBase64,
      };

      if (simulate) {
        const sim =
          action === "create"
            ? await ticketClassApi.simulateCreateTicketClass(session.token, payload)
            : action === "update"
              ? await ticketClassApi.simulateUpdateTicketClass(session.token, payload)
              : await ticketClassApi.simulateReserveInventory(session.token, payload);
        if (sim.err) throw new Error(JSON.stringify(sim.err));
        setClassAdminStatus("confirmed");
        setClassAdminLabel(`Simulation ok: ${action}`);
        return;
      }

      const res =
        action === "create"
          ? await ticketClassApi.createTicketClass(session.token, payload)
          : action === "update"
            ? await ticketClassApi.updateTicketClass(session.token, payload)
            : await ticketClassApi.reserveInventory(session.token, payload);
      setClassAdminStatus("confirmed");
      setClassAdminLabel(`Confirmed: ${action}`);
      setClassAdminSignature(res.signature);
      if (action === "create") {
        setSelectedClassPda(classPda.toBase58());
      }
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : err instanceof Error ? err.message : "Class action failed";
      setClassAdminStatus("failed");
      setClassAdminLabel("Ticket class admin action failed.");
      setClassAdminError(msg);
    }
  };

  const runOrganizerAction = async (
    action: "create" | "update" | "status" | "compliance" | "operator",
    simulate: boolean,
  ) => {
    setOrganizerTxStatus("pending");
    setOrganizerTxError(null);
    setOrganizerTxSignature(null);
    setOrganizerTxLabel(simulate ? `Simulating organizer ${action}...` : `Submitting organizer ${action}...`);

    try {
      if (!publicKey || !signTransaction) throw new Error("Connect wallet first.");
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      const programId = getProgramId();
      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      const [protocolConfig] = PublicKey.findProgramAddressSync([Buffer.from("protocol-config")], programId);

      let ix: TransactionInstruction;
      if (action === "create") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(
            concatBytes(
              CREATE_ORGANIZER_DISCRIMINATOR,
              encodeString(organizerMetadataUriInput),
              new PublicKey(organizerPayoutWalletInput || publicKey.toBase58()).toBytes(),
            ),
          ),
        });
      } else if (action === "update") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              UPDATE_ORGANIZER_DISCRIMINATOR,
              encodeString(organizerMetadataUriInput),
              new PublicKey(organizerPayoutWalletInput || publicKey.toBase58()).toBytes(),
            ),
          ),
        });
      } else if (action === "status") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              SET_ORGANIZER_STATUS_DISCRIMINATOR,
              Uint8Array.from([Number(organizerStatusInput)]),
            ),
          ),
        });
      } else if (action === "compliance") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              SET_ORGANIZER_COMPLIANCE_FLAGS_DISCRIMINATOR,
              encodeU32(Number(organizerComplianceFlagsInput)),
            ),
          ),
        });
      } else {
        const operatorWallet = new PublicKey(operatorWalletInput || publicKey.toBase58());
        const [organizerOperator] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("organizer-operator"),
            organizerProfile.toBuffer(),
            operatorWallet.toBuffer(),
          ],
          programId,
        );
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: true },
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: operatorWallet, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: organizerOperator, isSigner: false, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          ],
          data: Buffer.from(
            concatBytes(
              SET_ORGANIZER_OPERATOR_DISCRIMINATOR,
              encodeU32(Number(operatorPermissionsInput)),
              Uint8Array.from([operatorActiveInput ? 1 : 0]),
            ),
          ),
        });
      }

      const txBase64 = await signInstructionTx(ix);
      const payload = {
        organizer_id: organizerProfile.toBase58(),
        transaction_base64: txBase64,
      };

      if (simulate) {
        const sim =
          action === "create"
            ? await organizerApi.simulateCreateOrganizer(session.token, payload)
            : action === "update"
              ? await organizerApi.simulateUpdateOrganizer(session.token, payload)
              : action === "status"
                ? await organizerApi.simulateSetOrganizerStatus(session.token, payload)
                : action === "compliance"
                  ? await organizerApi.simulateSetOrganizerComplianceFlags(session.token, payload)
                  : await organizerApi.simulateSetOrganizerOperator(session.token, payload);
        if (sim.err) throw new Error(JSON.stringify(sim.err));
        setOrganizerTxStatus("confirmed");
        setOrganizerTxLabel(`Simulation ok: organizer ${action}`);
        return;
      }

      const result =
        action === "create"
          ? await organizerApi.createOrganizer(session.token, payload)
          : action === "update"
            ? await organizerApi.updateOrganizer(session.token, payload)
            : action === "status"
              ? await organizerApi.setOrganizerStatus(session.token, payload)
              : action === "compliance"
                ? await organizerApi.setOrganizerComplianceFlags(session.token, payload)
                : await organizerApi.setOrganizerOperator(session.token, payload);
      setOrganizerTxStatus("confirmed");
      setOrganizerTxLabel(`Confirmed: organizer ${action}`);
      setOrganizerTxSignature(result.signature);

      try {
        const read = await organizerApi.getOrganizer(session.token, organizerProfile.toBase58());
        setOrganizerRead((read.organizer as OrganizerReadRecord) ?? null);
      } catch {
        // no-op
      }
    } catch (orgError) {
      const message =
        orgError instanceof ApiError
          ? orgError.message
          : orgError instanceof Error
            ? orgError.message
            : "Organizer action failed.";
      setOrganizerTxStatus("failed");
      setOrganizerTxLabel("Organizer action failed.");
      setOrganizerTxError(message);
    }
  };

  const buyTicket = async (simulateOnly = false) => {
    setTxStatus("idle");
    setTxError(null);
    setTxSignature(null);
    setReceipt(null);

    try {
      if (!publicKey || !signTransaction) {
        throw new Error("Connect wallet first.");
      }
      if (!event || !selectedClass) {
        throw new Error("Select a ticket class first.");
      }
      if (selectedClass.classId <= 0) {
        throw new Error("Ticket class metadata is not indexed yet. Retry in a few seconds.");
      }

      const session = readAuthSession();
      if (!session.token) {
        throw new Error("Sign in required.");
      }

      const programId = getProgramId();
      const relayerPubkey = new PublicKey(
        process.env.NEXT_PUBLIC_RELAYER_PUBKEY ?? FALLBACK_RELAYER_PUBKEY,
      );
      const eventAccount = new PublicKey(event.event_id);
      const organizerProfile = new PublicKey(event.organizer_id);
      const ticketClass = new PublicKey(selectedClass.classPda);

      const [protocolConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from("protocol-config")],
        programId,
      );
      const [walletPurchaseCounter] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("wallet-purchase-counter"),
          eventAccount.toBuffer(),
          ticketClass.toBuffer(),
          publicKey.toBuffer(),
        ],
        programId,
      );
      const [complianceRegistry] = PublicKey.findProgramAddressSync(
        [Buffer.from("compliance-registry"), eventAccount.toBuffer()],
        programId,
      );

      const latestClassAccount = await connection.getAccountInfo(ticketClass, "confirmed");
      if (!latestClassAccount) throw new Error("Ticket class account not found.");
      const latestClassState = decodeTicketClass(Buffer.from(latestClassAccount.data));

      let ticketId = latestClassState.soldSupply + 1;
      let ticketPda: PublicKey | null = null;
      for (let i = 0; i < 8; i += 1) {
        const [candidate] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("ticket"),
            eventAccount.toBuffer(),
            Buffer.from(encodeU16(selectedClass.classId)),
            Buffer.from(encodeU32(ticketId)),
          ],
          programId,
        );
        const exists = await connection.getAccountInfo(candidate, "confirmed");
        if (!exists) {
          ticketPda = candidate;
          break;
        }
        ticketId += 1;
      }
      if (!ticketPda) {
        throw new Error("Could not allocate free ticket id. Retry.");
      }

      const protocolInfo = await connection.getAccountInfo(protocolConfig, "confirmed");
      if (!protocolInfo) throw new Error("Protocol config account not found.");
      const feeVault = decodeProtocolFeeVault(Buffer.from(protocolInfo.data));

      const organizerInfo = await connection.getAccountInfo(organizerProfile, "confirmed");
      if (!organizerInfo) throw new Error("Organizer profile account not found.");
      const organizerPayoutWallet = decodeOrganizerPayoutWallet(Buffer.from(organizerInfo.data));

      const data = concatBytes(
        BUY_TICKET_DISCRIMINATOR,
        encodeU16(selectedClass.classId),
        encodeU32(ticketId),
        encodeU64(selectedClass.priceLamports),
      );

      const ix = new TransactionInstruction({
        programId,
        keys: [
          { pubkey: publicKey, isSigner: true, isWritable: true },
          { pubkey: protocolConfig, isSigner: false, isWritable: false },
          { pubkey: organizerProfile, isSigner: false, isWritable: false },
          { pubkey: eventAccount, isSigner: false, isWritable: false },
          { pubkey: ticketClass, isSigner: false, isWritable: true },
          { pubkey: ticketPda, isSigner: false, isWritable: true },
          { pubkey: walletPurchaseCounter, isSigner: false, isWritable: true },
          { pubkey: feeVault, isSigner: false, isWritable: true },
          { pubkey: organizerPayoutWallet, isSigner: false, isWritable: true },
          { pubkey: new PublicKey(selectedClass.stakeholderWallet), isSigner: false, isWritable: true },
          { pubkey: complianceRegistry, isSigner: false, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(data),
      });

      setTxStatus("pending");
      setTxLabel(simulateOnly ? "Simulating buy ticket transaction..." : "Submitting buy ticket transaction...");

      let result: { signature: string; confirmation_status?: string } | null = null;
      for (let attempt = 0; attempt < 2; attempt += 1) {
        const { blockhash } = await connection.getLatestBlockhash("confirmed");
        const msg = new TransactionMessage({
          payerKey: relayerPubkey,
          recentBlockhash: blockhash,
          instructions: [ix],
        }).compileToV0Message();

        const tx = new VersionedTransaction(msg);
        const signed = await signTransaction(tx);
        const txBase64 = Buffer.from(signed.serialize()).toString("base64");
        const basePayload = {
          organizer_id: organizerProfile.toBase58(),
          event_id: eventAccount.toBase58(),
          class_id: ticketClass.toBase58(),
          transaction_base64: txBase64,
          buyer_wallet: publicKey.toBase58(),
          ticket_pda: ticketPda.toBase58(),
          gross_amount: Number(selectedClass.priceLamports),
          protocol_fee_amount: 0,
          net_amount: Number(selectedClass.priceLamports),
        };

        try {
          if (simulateOnly) {
            const sim = await primarySaleApi.simulateBuyTicket(session.token, basePayload);
            if (sim.err) throw new Error(JSON.stringify(sim.err));
            setTxStatus("confirmed");
            setTxLabel("Simulation ok: buy_ticket");
            return;
          }

          const response = await primarySaleApi.buyTicket(session.token, basePayload);
          result = {
            signature: response.receipt.signature,
            confirmation_status: response.confirmation_status,
          };
          break;
        } catch (submitError) {
          const text =
            submitError instanceof Error ? submitError.message : String(submitError);
          const isBlockhashIssue =
            text.includes("Blockhash not found") || text.includes("blockhash not found");
          const endpointMissing =
            submitError instanceof ApiError &&
            (submitError.status === 404 || submitError.status === 405);

          if (endpointMissing && !simulateOnly) {
            result = await relayApi.submitViaRelayer(session.token, {
              transaction_base64: txBase64,
              expected_instructions: ["buy_ticket"],
              max_retries: 20,
              timeout_ms: 120_000,
              poll_ms: 2_000,
            });
            break;
          }

          if (!isBlockhashIssue || attempt === 1) {
            throw submitError;
          }
        }
      }

      if (simulateOnly) return;
      if (!result) {
        throw new Error("Could not submit buy ticket transaction.");
      }

      setTxStatus("confirmed");
      setTxLabel("Ticket purchase confirmed.");
      setTxSignature(result.signature);
      setReceipt({
        signature: result.signature,
        ticketPda: ticketPda.toBase58(),
        classId: selectedClass.classId,
        priceLamports: selectedClass.priceLamports,
      });
      setClasses((prev) =>
        prev.map((item) =>
          item.classPda === selectedClass.classPda
            ? { ...item, soldSupply: item.soldSupply + 1 }
            : item,
        ),
      );
    } catch (buyError) {
      const message =
        buyError instanceof ApiError
          ? buyError.message
          : buyError instanceof Error
            ? buyError.message
            : "Could not buy ticket.";
      setTxStatus("failed");
      setTxLabel("Ticket purchase failed.");
      setTxError(message);
    }
  };

  const issueCompTicket = async (simulateOnly = false) => {
    setTxStatus("pending");
    setTxError(null);
    setTxSignature(null);
    setTxLabel(simulateOnly ? "Simulating comp ticket..." : "Submitting comp ticket...");
    try {
      if (!event || !publicKey) throw new Error("Event not loaded.");
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      const classId = Number(ticketClassIdInput);
      const ticketId = Number(compTicketIdInput);
      if (!Number.isFinite(classId) || classId <= 0 || !Number.isFinite(ticketId) || ticketId <= 0) {
        throw new Error("Class ID and Ticket ID must be positive numbers.");
      }
      const recipient = new PublicKey(compRecipientWallet || publicKey.toBase58());
      const programId = getProgramId();
      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      const [protocolConfig] = PublicKey.findProgramAddressSync([Buffer.from("protocol-config")], programId);
      const eventAccount = new PublicKey(event.event_id);
      const [ticketClass] = PublicKey.findProgramAddressSync(
        [Buffer.from("ticket-class"), eventAccount.toBuffer(), Buffer.from(encodeU16(classId))],
        programId,
      );
      const ticketPda = deriveTicketPda(eventAccount, classId, ticketId);
      const ix = new TransactionInstruction({
        programId,
        keys: [
          { pubkey: publicKey, isSigner: true, isWritable: true },
          { pubkey: publicKey, isSigner: true, isWritable: false },
          { pubkey: recipient, isSigner: false, isWritable: false },
          { pubkey: protocolConfig, isSigner: false, isWritable: false },
          { pubkey: organizerProfile, isSigner: false, isWritable: false },
          { pubkey: eventAccount, isSigner: false, isWritable: false },
          { pubkey: ticketClass, isSigner: false, isWritable: true },
          { pubkey: ticketPda, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(
          concatBytes(ISSUE_COMP_TICKET_DISCRIMINATOR, encodeU16(classId), encodeU32(ticketId)),
        ),
      });
      const txBase64 = await signInstructionTx(ix);
      const payload = {
        organizer_id: organizerProfile.toBase58(),
        event_id: eventAccount.toBase58(),
        class_id: ticketClass.toBase58(),
        transaction_base64: txBase64,
        buyer_wallet: recipient.toBase58(),
        ticket_pda: ticketPda.toBase58(),
        gross_amount: 0,
        protocol_fee_amount: 0,
        net_amount: 0,
      };
      if (simulateOnly) {
        const sim = await primarySaleApi.simulateIssueCompTicket(session.token, payload);
        if (sim.err) throw new Error(JSON.stringify(sim.err));
        setTxStatus("confirmed");
        setTxLabel("Simulation ok: issue_comp_ticket");
        return;
      }
      const response = await primarySaleApi.issueCompTicket(session.token, payload);
      setTxStatus("confirmed");
      setTxLabel("Comp ticket confirmed.");
      setTxSignature(response.receipt.signature);
      setCompTicketIdInput(String(ticketId + 1));
    } catch (compError) {
      const message =
        compError instanceof ApiError
          ? compError.message
          : compError instanceof Error
            ? compError.message
            : "Could not issue comp ticket.";
      setTxStatus("failed");
      setTxLabel("Comp ticket failed.");
      setTxError(message);
    }
  };

  const runTicketStateAction = async (
    action: "metadata" | "status",
    simulateOnly: boolean,
  ) => {
    setTicketStateStatus("pending");
    setTicketStateError(null);
    setTicketStateSignature(null);
    setTicketStateLabel(
      simulateOnly ? `Simulating ticket ${action}...` : `Submitting ticket ${action}...`,
    );
    try {
      if (!event || !publicKey) throw new Error("Event not loaded.");
      const classId = Number(ticketClassIdInput);
      const ticketId = Number(ticketIdInput);
      if (!Number.isFinite(classId) || classId <= 0 || !Number.isFinite(ticketId) || ticketId <= 0) {
        throw new Error("Class ID and Ticket ID must be positive numbers.");
      }
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      const programId = getProgramId();
      const eventAccount = new PublicKey(event.event_id);
      const [organizerProfile] = PublicKey.findProgramAddressSync(
        [Buffer.from("organizer"), publicKey.toBuffer()],
        programId,
      );
      const [protocolConfig] = PublicKey.findProgramAddressSync([Buffer.from("protocol-config")], programId);
      const [ticketClass] = PublicKey.findProgramAddressSync(
        [Buffer.from("ticket-class"), eventAccount.toBuffer(), Buffer.from(encodeU16(classId))],
        programId,
      );
      const ticketPda = deriveTicketPda(eventAccount, classId, ticketId);
      setTicketLookupPda(ticketPda.toBase58());

      let ix: TransactionInstruction;
      if (action === "metadata") {
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: ticketPda, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              SET_TICKET_METADATA_DISCRIMINATOR,
              encodeU16(classId),
              encodeU32(ticketId),
              encodeString(ticketMetadataUriInput),
              encodeU16(Number(ticketMetadataVersionInput)),
            ),
          ),
        });
      } else {
        const statusCode = ticketStatusToCode(ticketNextStatus);
        if (statusCode === 0) throw new Error("Invalid target status.");
        ix = new TransactionInstruction({
          programId,
          keys: [
            { pubkey: publicKey, isSigner: true, isWritable: false },
            { pubkey: protocolConfig, isSigner: false, isWritable: false },
            { pubkey: organizerProfile, isSigner: false, isWritable: false },
            { pubkey: eventAccount, isSigner: false, isWritable: false },
            { pubkey: ticketClass, isSigner: false, isWritable: false },
            { pubkey: ticketPda, isSigner: false, isWritable: true },
          ],
          data: Buffer.from(
            concatBytes(
              TRANSITION_TICKET_STATUS_DISCRIMINATOR,
              encodeU16(classId),
              encodeU32(ticketId),
              Uint8Array.from([statusCode]),
            ),
          ),
        });
      }

      const txBase64 = await signInstructionTx(ix);
      const payload = {
        organizer_id: organizerProfile.toBase58(),
        ticket_id: ticketPda.toBase58(),
        transaction_base64: txBase64,
      };

      if (simulateOnly) {
        const sim =
          action === "metadata"
            ? await ticketStateApi.simulateUpdateTicketMetadata(session.token, payload)
            : await ticketStateApi.simulateTransitionTicketStatus(session.token, {
                ...payload,
                target_status: ticketNextStatus,
              });
        if (sim.err) throw new Error(JSON.stringify(sim.err));
        setTicketStateStatus("confirmed");
        setTicketStateLabel(`Simulation ok: ticket ${action}`);
        return;
      }

      const response =
        action === "metadata"
          ? await ticketStateApi.updateTicketMetadata(session.token, payload)
          : await ticketStateApi.transitionTicketStatus(session.token, {
              ...payload,
              target_status: ticketNextStatus,
            });
      setTicketStateStatus("confirmed");
      setTicketStateLabel(`Confirmed: ticket ${action}`);
      setTicketStateSignature(response.signature);
    } catch (stateError) {
      const message =
        stateError instanceof ApiError
          ? stateError.message
          : stateError instanceof Error
            ? stateError.message
            : "Ticket state action failed.";
      setTicketStateStatus("failed");
      setTicketStateLabel("Ticket state action failed.");
      setTicketStateError(message);
    }
  };

  const fetchTicketState = async () => {
    setTicketStateStatus("pending");
    setTicketStateLabel("Loading ticket state...");
    setTicketStateError(null);
    try {
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      const ticketId = ticketLookupPda.trim();
      if (!ticketId) throw new Error("Ticket PDA is required.");
      const response = await ticketStateApi.getTicket(session.token, ticketId);
      setTicketReadData(response.ticket ?? null);
      setTicketStateStatus("confirmed");
      setTicketStateLabel("Ticket state loaded.");
    } catch (loadError) {
      const message =
        loadError instanceof ApiError
          ? loadError.message
          : loadError instanceof Error
            ? loadError.message
            : "Could not load ticket state.";
      setTicketStateStatus("failed");
      setTicketStateLabel("Ticket read failed.");
      setTicketStateError(message);
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <Link href="/dashboard" className={styles.backBtn}>
            Back to Dashboard
          </Link>
          <h1>{event?.name ?? "Event"}</h1>
          <p>Start: {formatEpoch(event?.starts_at_epoch)}</p>
          <p>Status: {event?.status ?? "unknown"}</p>
        </section>

        {loading ? <section className={styles.statusCard}>Loading event...</section> : null}
        {!loading && error ? <section className={styles.statusCard}>{error}</section> : null}

        {!loading && !error ? (
          <section className={styles.buyCard}>
            <h2>Primary Sale</h2>
            <p className={styles.caption}>
              Integrates `/primary-sale/buy`, `/primary-sale/buy/simulate`, `/primary-sale/comp`,
              `/primary-sale/comp/simulate`.
            </p>

            <label className={styles.field}>
              <span>Ticket class</span>
              <select
                value={selectedClassPda}
                onChange={(e) => setSelectedClassPda(e.target.value)}
                disabled={classes.length === 0}
              >
                {classes.map((item) => (
                  <option key={item.classPda} value={item.classPda}>
                    #{item.classId} {item.name} - {formatSol(item.priceLamports)} SOL
                  </option>
                ))}
              </select>
            </label>

            {selectedClass ? (
              <div className={styles.summary}>
                <p>Class ID: {selectedClass.classId}</p>
                <p>Price: {formatSol(selectedClass.priceLamports)} SOL</p>
                <p>Sold: {selectedClass.soldSupply}</p>
              </div>
            ) : null}

            <div className={styles.actionsWrap}>
              <button
                type="button"
                className={styles.buyBtn}
                onClick={() => void buyTicket(true)}
                disabled={!selectedClass || txStatus === "pending"}
              >
                Sim Buy Ticket
              </button>
              <button
                type="button"
                className={styles.buyBtn}
                onClick={() => void buyTicket(false)}
                disabled={!selectedClass || txStatus === "pending"}
              >
                {txStatus === "pending" ? "Submitting..." : "Buy Ticket"}
              </button>
            </div>

            <div className={styles.adminGrid}>
              <label className={styles.field}>
                <span>Comp Recipient Wallet</span>
                <input
                  placeholder={publicKey?.toBase58() ?? "Recipient wallet"}
                  value={compRecipientWallet}
                  onChange={(e) => setCompRecipientWallet(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Comp Ticket ID</span>
                <input value={compTicketIdInput} onChange={(e) => setCompTicketIdInput(e.target.value)} />
              </label>
            </div>

            <div className={styles.actionsWrap}>
              <button
                type="button"
                className={styles.actionBtn}
                onClick={() => void issueCompTicket(true)}
                disabled={!isOwner || txStatus === "pending"}
              >
                Sim Comp Ticket
              </button>
              <button
                type="button"
                className={styles.actionBtn}
                onClick={() => void issueCompTicket(false)}
                disabled={!isOwner || txStatus === "pending"}
              >
                Issue Comp Ticket
              </button>
            </div>

            <TxStatusCard status={txStatus} label={txLabel} signature={txSignature} error={txError} />

            {receipt ? (
              <section className={styles.receipt}>
                <h3>Purchase Receipt</h3>
                <p>Class: #{receipt.classId}</p>
                <p>Price: {formatSol(receipt.priceLamports)} SOL</p>
                <p>Ticket PDA: {receipt.ticketPda}</p>
                <a
                  href={`https://explorer.solana.com/tx/${receipt.signature}?cluster=${process.env.NEXT_PUBLIC_SOLANA_CLUSTER ?? "devnet"}`}
                  target="_blank"
                  rel="noreferrer"
                >
                  View Signature
                </a>
              </section>
            ) : null}
          </section>
        ) : null}

        {!loading && !error ? (
          <section className={styles.buyCard}>
            <h2>Organizer Admin</h2>
            <p className={styles.caption}>
              Integrates `/organizers*` create/read/update/status/compliance/operators with simulate + submit.
            </p>

            <div className={styles.summary}>
              <p>Organizer ID: {event?.organizer_id ?? "N/A"}</p>
              <p>Status: {organizerRead?.status ?? "unknown"}</p>
              <p>Compliance Flags: {organizerRead?.compliance_flags ?? 0}</p>
              <p>Payout Wallet: {organizerRead?.payout_wallet ?? organizerPayoutWalletInput ?? "N/A"}</p>
            </div>

            <div className={styles.adminGrid}>
              <label className={styles.field}>
                <span>Metadata URI</span>
                <input
                  placeholder="https://..."
                  value={organizerMetadataUriInput}
                  onChange={(e) => setOrganizerMetadataUriInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Payout Wallet</span>
                <input
                  value={organizerPayoutWalletInput}
                  onChange={(e) => setOrganizerPayoutWalletInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Status (1=active, 2=suspended)</span>
                <input value={organizerStatusInput} onChange={(e) => setOrganizerStatusInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Compliance Flags (u32)</span>
                <input
                  value={organizerComplianceFlagsInput}
                  onChange={(e) => setOrganizerComplianceFlagsInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Operator Wallet</span>
                <input value={operatorWalletInput} onChange={(e) => setOperatorWalletInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Operator Permissions (u32)</span>
                <input
                  value={operatorPermissionsInput}
                  onChange={(e) => setOperatorPermissionsInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Operator Active</span>
                <select
                  value={operatorActiveInput ? "1" : "0"}
                  onChange={(e) => setOperatorActiveInput(e.target.value === "1")}
                >
                  <option value="1">true</option>
                  <option value="0">false</option>
                </select>
              </label>
            </div>

            <div className={styles.actionsWrap}>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("create", true)}>Sim Create Organizer</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("create", false)}>Submit Create Organizer</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("update", true)}>Sim Update Organizer</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("update", false)}>Submit Update Organizer</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("status", true)}>Sim Status</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("status", false)}>Submit Status</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("compliance", true)}>Sim Compliance</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("compliance", false)}>Submit Compliance</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("operator", true)}>Sim Operator</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runOrganizerAction("operator", false)}>Submit Operator</button>
            </div>

            <TxStatusCard
              status={organizerTxStatus}
              label={organizerTxLabel}
              signature={organizerTxSignature}
              error={organizerTxError}
            />
          </section>
        ) : null}

        {!loading && !error ? (
          <section className={styles.buyCard}>
            <h2>Event Admin</h2>
            <p className={styles.caption}>
              Integrates `/events*` write + simulate endpoints. {isOwner ? "Organizer owner mode." : "Read-only mode."}
            </p>

            <div className={styles.adminGrid}>
              <label className={styles.field}>
                <span>Title</span>
                <input value={updateTitle} onChange={(e) => setUpdateTitle(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Venue</span>
                <input value={updateVenue} onChange={(e) => setUpdateVenue(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Start</span>
                <input type="datetime-local" value={updateStart} onChange={(e) => setUpdateStart(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>End</span>
                <input type="datetime-local" value={updateEnd} onChange={(e) => setUpdateEnd(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Sales Start</span>
                <input
                  type="datetime-local"
                  value={updateSalesStart}
                  onChange={(e) => setUpdateSalesStart(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Lock</span>
                <input type="datetime-local" value={updateLock} onChange={(e) => setUpdateLock(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Capacity</span>
                <input value={updateCapacity} onChange={(e) => setUpdateCapacity(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Pause Flag</span>
                <select value={pauseFlag ? "1" : "0"} onChange={(e) => setPauseFlag(e.target.value === "1")}>
                  <option value="0">false</option>
                  <option value="1">true</option>
                </select>
              </label>
              <label className={styles.field}>
                <span>Restriction Flags</span>
                <input value={restrictionFlags} onChange={(e) => setRestrictionFlags(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Decision Code</span>
                <input value={decisionCode} onChange={(e) => setDecisionCode(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Loyalty Multiplier BPS</span>
                <input value={loyaltyMultiplier} onChange={(e) => setLoyaltyMultiplier(e.target.value)} />
              </label>
            </div>

            <div className={styles.actionsWrap}>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("update", true)}>Sim Update</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("update", false)}>Submit Update</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("freeze", true)}>Sim Freeze</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("freeze", false)}>Submit Freeze</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("cancel", true)}>Sim Cancel</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("cancel", false)}>Submit Cancel</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("pause", true)}>Sim Pause</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("pause", false)}>Submit Pause</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("close", true)}>Sim Close</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("close", false)}>Submit Close</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("restrictions", true)}>Sim Restrictions</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("restrictions", false)}>Submit Restrictions</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("loyalty", true)}>Sim Loyalty</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runEventEndpointAction("loyalty", false)}>Submit Loyalty</button>
            </div>

            <TxStatusCard
              status={adminTxStatus}
              label={adminTxLabel}
              signature={adminTxSignature}
              error={adminTxError}
            />
          </section>
        ) : null}

        {!loading && !error ? (
          <section className={styles.buyCard}>
            <h2>Ticket Class Admin</h2>
            <p className={styles.caption}>
              Integrates `/ticket-classes*` write + simulate + read endpoints.
            </p>

            <div className={styles.summary}>
              <p>Selected Class PDA: {selectedClassPda || "N/A"}</p>
              <p>Read Status: {classRead?.status ?? "unknown"}</p>
              <p>
                Supply (T/R/S): {classAnalytics?.supply_total ?? classRead?.supply_total ?? 0}/
                {classAnalytics?.supply_reserved ?? classRead?.supply_reserved ?? 0}/
                {classAnalytics?.supply_sold ?? classRead?.supply_sold ?? 0}
              </p>
              <p>Remaining: {classAnalytics?.supply_remaining ?? "n/a"}</p>
            </div>

            <div className={styles.adminGrid}>
              <label className={styles.field}>
                <span>Class ID</span>
                <input value={classIdInput} onChange={(e) => setClassIdInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Name</span>
                <input value={classNameInput} onChange={(e) => setClassNameInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Supply</span>
                <input value={classSupplyInput} onChange={(e) => setClassSupplyInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Reserved</span>
                <input value={classReservedInput} onChange={(e) => setClassReservedInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Price (SOL)</span>
                <input value={classPriceInput} onChange={(e) => setClassPriceInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Sale Start</span>
                <input
                  type="datetime-local"
                  value={classSaleStartInput}
                  onChange={(e) => setClassSaleStartInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Sale End</span>
                <input
                  type="datetime-local"
                  value={classSaleEndInput}
                  onChange={(e) => setClassSaleEndInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Wallet Limit</span>
                <input
                  value={classWalletLimitInput}
                  onChange={(e) => setClassWalletLimitInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Transferable</span>
                <select
                  value={classTransferable ? "1" : "0"}
                  onChange={(e) => setClassTransferable(e.target.value === "1")}
                >
                  <option value="1">true</option>
                  <option value="0">false</option>
                </select>
              </label>
              <label className={styles.field}>
                <span>Resale Enabled</span>
                <select
                  value={classResaleEnabled ? "1" : "0"}
                  onChange={(e) => setClassResaleEnabled(e.target.value === "1")}
                >
                  <option value="1">true</option>
                  <option value="0">false</option>
                </select>
              </label>
              <label className={styles.field}>
                <span>Reserve Amount</span>
                <input
                  value={reserveAmountInput}
                  onChange={(e) => setReserveAmountInput(e.target.value)}
                />
              </label>
            </div>

            <div className={styles.actionsWrap}>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketClassAction("create", true)}>Sim Create Class</button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketClassAction("create", false)}>Submit Create Class</button>
              <button className={styles.actionBtn} disabled={!isOwner || !selectedClass} onClick={() => void runTicketClassAction("update", true)}>Sim Update Class</button>
              <button className={styles.actionBtn} disabled={!isOwner || !selectedClass} onClick={() => void runTicketClassAction("update", false)}>Submit Update Class</button>
              <button className={styles.actionBtn} disabled={!isOwner || !selectedClass} onClick={() => void runTicketClassAction("reserve", true)}>Sim Reserve</button>
              <button className={styles.actionBtn} disabled={!isOwner || !selectedClass} onClick={() => void runTicketClassAction("reserve", false)}>Submit Reserve</button>
            </div>

            <TxStatusCard
              status={classAdminStatus}
              label={classAdminLabel}
              signature={classAdminSignature}
              error={classAdminError}
            />
          </section>
        ) : null}

        {!loading && !error ? (
          <section className={styles.buyCard}>
            <h2>Ticket State Admin</h2>
            <p className={styles.caption}>
              Integrates GET /tickets/{'{ticket_id}'} plus ticket metadata/status write and simulate endpoints.
            </p>

            <div className={styles.adminGrid}>
              <label className={styles.field}>
                <span>Ticket Class ID</span>
                <input value={ticketClassIdInput} onChange={(e) => setTicketClassIdInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Ticket ID</span>
                <input value={ticketIdInput} onChange={(e) => setTicketIdInput(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Ticket PDA</span>
                <input value={ticketLookupPda} onChange={(e) => setTicketLookupPda(e.target.value)} />
              </label>
              <label className={styles.field}>
                <span>Metadata URI</span>
                <input
                  placeholder="ipfs://..."
                  value={ticketMetadataUriInput}
                  onChange={(e) => setTicketMetadataUriInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Metadata Version</span>
                <input
                  value={ticketMetadataVersionInput}
                  onChange={(e) => setTicketMetadataVersionInput(e.target.value)}
                />
              </label>
              <label className={styles.field}>
                <span>Next Status</span>
                <select value={ticketNextStatus} onChange={(e) => setTicketNextStatus(e.target.value)}>
                  <option value="active">active</option>
                  <option value="checked_in">checked_in</option>
                  <option value="refunded">refunded</option>
                  <option value="invalidated">invalidated</option>
                </select>
              </label>
            </div>

            <div className={styles.actionsWrap}>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void fetchTicketState()}>
                Read Ticket
              </button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketStateAction("metadata", true)}>
                Sim Metadata
              </button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketStateAction("metadata", false)}>
                Submit Metadata
              </button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketStateAction("status", true)}>
                Sim Status
              </button>
              <button className={styles.actionBtn} disabled={!isOwner} onClick={() => void runTicketStateAction("status", false)}>
                Submit Status
              </button>
            </div>

            {ticketReadData ? (
              <pre className={styles.statusCard}>{JSON.stringify(ticketReadData, null, 2)}</pre>
            ) : null}

            <TxStatusCard
              status={ticketStateStatus}
              label={ticketStateLabel}
              signature={ticketStateSignature}
              error={ticketStateError}
            />
          </section>
        ) : null}
      </main>
      <Footer />
    </div>
  );
}
