"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { resalePolicyApi } from "@/lib/api";
import { ApiError } from "@/lib/api/http";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import { TxStatusCard, type TxStatus } from "@/components/tx/TxStatusCard";
import { getProgramId } from "@/lib/solana";
import { readAuthSession } from "@/lib/session";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import styles from "./ResalePolicyPage.module.css";

const SET_RESALE_POLICY_DISCRIMINATOR = Uint8Array.from([69, 96, 104, 248, 153, 249, 250, 42]);

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

function encodePubkeyVec(pubkeys: PublicKey[]) {
  const body = new Uint8Array(pubkeys.length * 32);
  pubkeys.forEach((pk, idx) => body.set(pk.toBytes(), idx * 32));
  return concatBytes(encodeU32(pubkeys.length), body);
}

function parsePubkeysCsv(input: string): PublicKey[] {
  const raw = input
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  return raw.map((v) => new PublicKey(v));
}

export function ResalePolicyPage() {
  const { connection } = useConnection();
  const { publicKey, signTransaction } = useWallet();

  const [organizerId, setOrganizerId] = useState("");
  const [eventId, setEventId] = useState("");
  const [classId, setClassId] = useState("1");
  const [policyIdInput, setPolicyIdInput] = useState("");

  const [maxMarkupBps, setMaxMarkupBps] = useState("2000");
  const [royaltyBps, setRoyaltyBps] = useState("500");
  const [royaltyVault, setRoyaltyVault] = useState("");
  const [transferCooldownSecs, setTransferCooldownSecs] = useState("0");
  const [maxTransferCount, setMaxTransferCount] = useState("0");
  const [transferLockBeforeEventSecs, setTransferLockBeforeEventSecs] = useState("0");
  const [whitelistCsv, setWhitelistCsv] = useState("");
  const [blacklistCsv, setBlacklistCsv] = useState("");

  const [recId, setRecId] = useState(`rec_${Date.now()}`);
  const [confidence, setConfidence] = useState("0.8");
  const [rationale, setRationale] = useState("Market-driven recommendation");

  const [status, setStatus] = useState<TxStatus>("idle");
  const [label, setLabel] = useState("");
  const [signature, setSignature] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [policyRead, setPolicyRead] = useState<unknown | null>(null);
  const [validationRead, setValidationRead] = useState<unknown | null>(null);
  const [recommendationRead, setRecommendationRead] = useState<unknown | null>(null);

  const parsed = useMemo(() => {
    return {
      classNum: Number(classId),
      maxMarkup: Number(maxMarkupBps),
      royalty: Number(royaltyBps),
      cooldown: Number(transferCooldownSecs),
      maxTransfers: Number(maxTransferCount),
      lockBefore: Number(transferLockBeforeEventSecs),
      confidenceNum: Number(confidence),
    };
  }, [classId, confidence, maxMarkupBps, maxTransferCount, royaltyBps, transferCooldownSecs, transferLockBeforeEventSecs]);

  useEffect(() => {
    if (!publicKey || organizerId) return;
    const programId = getProgramId();
    const [derived] = PublicKey.findProgramAddressSync([Buffer.from("organizer"), publicKey.toBuffer()], programId);
    setOrganizerId(derived.toBase58());
    if (!royaltyVault) setRoyaltyVault(publicKey.toBase58());
  }, [organizerId, publicKey, royaltyVault]);

  const derivePolicyPda = () => {
    const programId = getProgramId();
    const eventKey = new PublicKey(eventId);
    const [policyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("resale-policy"), eventKey.toBuffer(), Buffer.from(encodeU16(parsed.classNum))],
      programId,
    );
    return policyPda;
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

  const submitPolicy = async (simulate: boolean) => {
    setStatus("pending");
    setError(null);
    setSignature(null);
    setLabel(simulate ? "Simulating set policy..." : "Submitting set policy...");

    try {
      if (!publicKey) throw new Error("Connect wallet first.");
      if (!organizerId || !eventId) throw new Error("Organizer ID and Event ID are required.");
      if (!Number.isFinite(parsed.classNum) || parsed.classNum <= 0) throw new Error("Invalid class id.");

      const whitelist = parsePubkeysCsv(whitelistCsv);
      const blacklist = parsePubkeysCsv(blacklistCsv);
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");

      const programId = getProgramId();
      const organizerKey = new PublicKey(organizerId);
      const eventKey = new PublicKey(eventId);
      const payoutKey = new PublicKey(royaltyVault);
      const [protocolConfig] = PublicKey.findProgramAddressSync([Buffer.from("protocol-config")], programId);
      const [ticketClass] = PublicKey.findProgramAddressSync(
        [Buffer.from("ticket-class"), eventKey.toBuffer(), Buffer.from(encodeU16(parsed.classNum))],
        programId,
      );
      const policyPda = derivePolicyPda();
      setPolicyIdInput(policyPda.toBase58());

      const inputBytes = concatBytes(
        encodeU16(parsed.maxMarkup),
        encodeU16(parsed.royalty),
        payoutKey.toBytes(),
        encodeI64(parsed.cooldown),
        encodeU16(parsed.maxTransfers),
        encodeI64(parsed.lockBefore),
        encodePubkeyVec(whitelist),
        encodePubkeyVec(blacklist),
      );

      const ix = new TransactionInstruction({
        programId,
        keys: [
          { pubkey: publicKey, isSigner: true, isWritable: true },
          { pubkey: publicKey, isSigner: true, isWritable: false },
          { pubkey: protocolConfig, isSigner: false, isWritable: false },
          { pubkey: organizerKey, isSigner: false, isWritable: false },
          { pubkey: eventKey, isSigner: false, isWritable: false },
          { pubkey: ticketClass, isSigner: false, isWritable: false },
          { pubkey: policyPda, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(
          concatBytes(SET_RESALE_POLICY_DISCRIMINATOR, encodeU16(parsed.classNum), inputBytes),
        ),
      });

      const txBase64 = await signInstruction(ix);
      const payload = {
        organizer_id: organizerKey.toBase58(),
        event_id: eventKey.toBase58(),
        class_id: ticketClass.toBase58(),
        transaction_base64: txBase64,
      };

      if (simulate) {
        const sim = await resalePolicyApi.simulateSetResalePolicy(session.token, payload);
        if (sim.err) throw new Error(JSON.stringify(sim.err));
        setStatus("confirmed");
        setLabel("Simulation ok: set_resale_policy");
        return;
      }

      const result = await resalePolicyApi.setResalePolicy(session.token, payload);
      setStatus("confirmed");
      setLabel("Confirmed: set_resale_policy");
      setSignature(result.signature);
    } catch (submitError) {
      const message =
        submitError instanceof ApiError
          ? submitError.message
          : submitError instanceof Error
            ? submitError.message
            : "Resale policy action failed.";
      setStatus("failed");
      setLabel("Resale policy action failed.");
      setError(message);
    }
  };

  const runValidation = async () => {
    setStatus("pending");
    setLabel("Validating policy request...");
    setError(null);
    try {
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      const response = await resalePolicyApi.validatePolicy(session.token, {
        max_markup_bps: parsed.maxMarkup,
        royalty_bps: parsed.royalty,
        whitelist_enabled: whitelistCsv.trim().length > 0,
        blacklist_enabled: blacklistCsv.trim().length > 0,
      });
      setValidationRead(response);
      setStatus("confirmed");
      setLabel("Validation complete.");
    } catch (validationError) {
      const message =
        validationError instanceof ApiError
          ? validationError.message
          : validationError instanceof Error
            ? validationError.message
            : "Validation failed.";
      setStatus("failed");
      setLabel("Validation failed.");
      setError(message);
    }
  };

  const writeRecommendation = async () => {
    setStatus("pending");
    setLabel("Writing recommendation...");
    setError(null);
    try {
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");
      if (!organizerId || !eventId) throw new Error("Organizer ID and Event ID are required.");
      const response = await resalePolicyApi.writePolicyRecommendation(session.token, {
        recommendation_id: recId,
        organizer_id: organizerId,
        event_id: eventId,
        class_id: derivePolicyPda().toBase58(),
        max_markup_bps: parsed.maxMarkup,
        royalty_bps: parsed.royalty,
        confidence: parsed.confidenceNum,
        rationale,
      });
      setRecommendationRead(response);
      setStatus("confirmed");
      setLabel("Recommendation saved.");
    } catch (recommendError) {
      const message =
        recommendError instanceof ApiError
          ? recommendError.message
          : recommendError instanceof Error
            ? recommendError.message
            : "Recommendation write failed.";
      setStatus("failed");
      setLabel("Recommendation write failed.");
      setError(message);
    }
  };

  const readPolicy = async () => {
    setStatus("pending");
    setLabel("Loading policy...");
    setError(null);
    try {
      const session = readAuthSession();
      if (!session.token) throw new Error("Sign in required.");

      const policyId = policyIdInput.trim();
      if (policyId) {
        try {
          const byId = await resalePolicyApi.getResalePolicyById(session.token, policyId);
          setPolicyRead(byId.policy ?? null);
          setStatus("confirmed");
          setLabel("Policy loaded by ID.");
          return;
        } catch (idError) {
          const endpointMissing = idError instanceof ApiError && (idError.status === 404 || idError.status === 405);
          if (!endpointMissing) throw idError;
        }
      }

      if (!eventId) throw new Error("Event ID is required for query fallback read.");
      const queryRead = await resalePolicyApi.getResalePolicy(session.token, {
        event_id: eventId,
        class_id: derivePolicyPda().toBase58(),
      });
      setPolicyRead(queryRead.policy ?? null);
      setStatus("confirmed");
      setLabel("Policy loaded by query fallback.");
    } catch (readError) {
      const message =
        readError instanceof ApiError
          ? readError.message
          : readError instanceof Error
            ? readError.message
            : "Policy read failed.";
      setStatus("failed");
      setLabel("Policy read failed.");
      setError(message);
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />

      <main className={styles.content}>
        <section className={styles.header}>
          <h1>Resale Policy</h1>
          <p>Dedicated flow for policy set/simulate/recommend/validate/read.</p>
          <div className={styles.links}>
            <Link href="/dashboard">Back to Dashboard</Link>
            {policyIdInput ? <Link href={`/resale-policy/${encodeURIComponent(policyIdInput)}`}>Open Policy Detail</Link> : null}
          </div>
        </section>

        <section className={styles.card}>
          <h2>Context</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Organizer ID</span>
              <input value={organizerId} onChange={(e) => setOrganizerId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Event ID</span>
              <input value={eventId} onChange={(e) => setEventId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Class ID (u16)</span>
              <input value={classId} onChange={(e) => setClassId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Policy ID</span>
              <input value={policyIdInput} onChange={(e) => setPolicyIdInput(e.target.value)} />
            </label>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Set Policy</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Max Markup BPS</span>
              <input value={maxMarkupBps} onChange={(e) => setMaxMarkupBps(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Royalty BPS</span>
              <input value={royaltyBps} onChange={(e) => setRoyaltyBps(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Royalty Vault</span>
              <input value={royaltyVault} onChange={(e) => setRoyaltyVault(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Transfer Cooldown Secs</span>
              <input value={transferCooldownSecs} onChange={(e) => setTransferCooldownSecs(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Max Transfer Count</span>
              <input value={maxTransferCount} onChange={(e) => setMaxTransferCount(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Transfer Lock Before Event Secs</span>
              <input value={transferLockBeforeEventSecs} onChange={(e) => setTransferLockBeforeEventSecs(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Whitelist (comma-separated wallets)</span>
              <input value={whitelistCsv} onChange={(e) => setWhitelistCsv(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Blacklist (comma-separated wallets)</span>
              <input value={blacklistCsv} onChange={(e) => setBlacklistCsv(e.target.value)} />
            </label>
          </div>

          <div className={styles.actions}>
            <button onClick={() => void submitPolicy(true)}>Sim Set Policy</button>
            <button onClick={() => void submitPolicy(false)}>Submit Set Policy</button>
          </div>
        </section>

        <section className={styles.card}>
          <h2>Validate + Recommendation</h2>
          <div className={styles.grid}>
            <label className={styles.field}>
              <span>Recommendation ID</span>
              <input value={recId} onChange={(e) => setRecId(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Confidence</span>
              <input value={confidence} onChange={(e) => setConfidence(e.target.value)} />
            </label>
            <label className={styles.field}>
              <span>Rationale</span>
              <input value={rationale} onChange={(e) => setRationale(e.target.value)} />
            </label>
          </div>
          <div className={styles.actions}>
            <button onClick={() => void runValidation()}>Validate Policy</button>
            <button onClick={() => void writeRecommendation()}>Write Recommendation</button>
          </div>
          {validationRead ? <pre className={styles.pre}>{JSON.stringify(validationRead, null, 2)}</pre> : null}
          {recommendationRead ? <pre className={styles.pre}>{JSON.stringify(recommendationRead, null, 2)}</pre> : null}
        </section>

        <section className={styles.card}>
          <h2>Policy Read</h2>
          <div className={styles.actions}>
            <button onClick={() => void readPolicy()}>Read Policy</button>
          </div>
          {policyRead ? <pre className={styles.pre}>{JSON.stringify(policyRead, null, 2)}</pre> : null}
          <p className={styles.caption}>
            `GET /resale-policy/{{policy_id}}` is attempted first, then query fallback is used for local backend.
          </p>
        </section>

        <TxStatusCard status={status} label={label} signature={signature} error={error} />
      </main>
      <Footer />
    </div>
  );
}
