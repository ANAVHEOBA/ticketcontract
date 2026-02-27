"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { authApi } from "@/lib/api";
import { LumaWordmark } from "@/components/luma/LumaLogo";
import { ApiError } from "@/lib/api/http";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import styles from "./SignInPage.module.css";

const attemptedAutoAuthWallets = new Set<string>();
const inFlightAutoAuthWallets = new Set<string>();

type GooglePromptNotification = {
  isNotDisplayed?: () => boolean;
  isSkippedMoment?: () => boolean;
  getNotDisplayedReason?: () => string;
  getSkippedReason?: () => string;
};

type GoogleAccountsId = {
  initialize: (params: {
    client_id: string;
    callback: (response: { credential?: string }) => void;
    auto_select?: boolean;
    cancel_on_tap_outside?: boolean;
  }) => void;
  prompt: (callback?: (notification: GooglePromptNotification) => void) => void;
};

function formatLocalTime() {
  return new Intl.DateTimeFormat("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
    timeZoneName: "short",
  }).format(new Date());
}

function toBase64(bytes: Uint8Array): string {
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

function loadGoogleScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    const id = "google-identity-services";
    if (document.getElementById(id)) {
      resolve();
      return;
    }

    const script = document.createElement("script");
    script.id = id;
    script.src = "https://accounts.google.com/gsi/client";
    script.async = true;
    script.defer = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error("Failed to load Google Identity Services"));
    document.head.appendChild(script);
  });
}

function getGoogleIdToken(clientId: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const google = (window as unknown as { google?: { accounts?: { id?: GoogleAccountsId } } })
      .google;
    if (!google?.accounts?.id) {
      reject(new Error("Google Identity Services not available"));
      return;
    }

    google.accounts.id.initialize({
      client_id: clientId,
      callback: (response: { credential?: string }) => {
        if (!response?.credential) {
          reject(new Error("Google sign-in was cancelled or blocked"));
          return;
        }
        resolve(response.credential);
      },
      auto_select: false,
      cancel_on_tap_outside: true,
    });

    google.accounts.id.prompt((notification: GooglePromptNotification) => {
      if (notification?.isNotDisplayed?.()) {
        reject(new Error(notification.getNotDisplayedReason?.() || "Google prompt not displayed"));
      } else if (notification?.isSkippedMoment?.()) {
        reject(new Error(notification.getSkippedReason?.() || "Google prompt skipped"));
      }
    });
  });
}

export function SignInPage() {
  const router = useRouter();
  const { connection } = useConnection();
  const { connected, publicKey, signMessage, disconnect, wallet } = useWallet();
  const { setVisible } = useWalletModal();

  const [timeLabel, setTimeLabel] = useState("");
  const [busy, setBusy] = useState(false);
  const [busyLabel, setBusyLabel] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [balanceSol, setBalanceSol] = useState<string>("-");
  const [copied, setCopied] = useState(false);
  const autoAuthWalletRef = useRef<string | null>(null);

  const googleClientId = useMemo(() => process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID ?? "", []);
  const walletLabel =
    connected && publicKey
      ? `Wallet Connected: ${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
      : "Connect Wallet";

  useEffect(() => {
    const update = () => setTimeLabel(formatLocalTime());
    update();
    const timer = window.setInterval(update, 60_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let cancelled = false;
    const readBalance = async () => {
      if (!connected || !publicKey) {
        setBalanceSol("-");
        return;
      }
      try {
        const lamports = await connection.getBalance(publicKey, "confirmed");
        if (!cancelled) {
          setBalanceSol((lamports / LAMPORTS_PER_SOL).toFixed(4));
        }
      } catch {
        if (!cancelled) {
          setBalanceSol("-");
        }
      }
    };
    void readBalance();
    return () => {
      cancelled = true;
    };
  }, [connected, publicKey, connection]);

  const completeWalletAuth = useCallback(
    async (idToken?: string) => {
      if (!connected || !publicKey) {
        throw new Error("Connect a wallet first.");
      }

      if (!signMessage) {
        throw new Error("This wallet does not support message signing.");
      }

      const wallet = publicKey.toBase58();
      const nonce = await authApi.issueNonce(wallet);
      const signed = await signMessage(new TextEncoder().encode(nonce.message));
      const signature = toBase64(signed);

      const auth = idToken
        ? await authApi.verifyProvider({
            provider: "google",
            id_token: idToken,
            wallet,
            nonce: nonce.nonce,
            signature,
          })
        : await authApi.verifySignature({
            wallet,
            nonce: nonce.nonce,
            signature,
          });

      localStorage.setItem("ticketing_access_token", auth.access_token);
      localStorage.setItem("ticketing_role", auth.role);
      localStorage.setItem("ticketing_wallet", wallet);
      localStorage.setItem("ticketing_organizer_scopes", JSON.stringify(auth.organizer_scopes));

      router.push("/dashboard");
    },
    [connected, publicKey, router, signMessage],
  );

  useEffect(() => {
    const runAutoAuth = async () => {
      if (!connected || !publicKey) return;

      const walletAddress = publicKey.toBase58();
      if (inFlightAutoAuthWallets.has(walletAddress) || attemptedAutoAuthWallets.has(walletAddress)) {
        return;
      }

      autoAuthWalletRef.current = walletAddress;
      inFlightAutoAuthWallets.add(walletAddress);
      attemptedAutoAuthWallets.add(walletAddress);
      setBusy(true);
      setBusyLabel("wallet");
      setMessage(null);

      try {
        await completeWalletAuth();
      } catch (error) {
        if (error instanceof ApiError) {
          setMessage(error.message);
        } else {
          const raw = error instanceof Error ? error.message : "Wallet sign-in failed";
          const rejected = raw.toLowerCase().includes("rejected");
          setMessage(rejected ? "Signature request was rejected. Disconnect and reconnect to retry." : raw);
        }
      } finally {
        inFlightAutoAuthWallets.delete(walletAddress);
        setBusy(false);
        setBusyLabel(null);
      }
    };

    void runAutoAuth();
  }, [completeWalletAuth, connected, publicKey]);

  const handleDisconnectWallet = useCallback(async () => {
    const currentWallet = publicKey?.toBase58() ?? autoAuthWalletRef.current;
    if (currentWallet) {
      inFlightAutoAuthWallets.delete(currentWallet);
      attemptedAutoAuthWallets.delete(currentWallet);
    }
    autoAuthWalletRef.current = null;
    setBusy(false);
    setBusyLabel(null);
    setMessage(null);
    await disconnect();
  }, [disconnect, publicKey]);

  const signInWithGoogleAndWallet = async () => {
    if (busy) return;
    setBusy(true);
    setBusyLabel("google");
    setMessage(null);

    try {
      if (!googleClientId) {
        throw new Error("Missing NEXT_PUBLIC_GOOGLE_CLIENT_ID in frontend .env");
      }
      await loadGoogleScript();
      const idToken = await getGoogleIdToken(googleClientId);
      await completeWalletAuth(idToken);
    } catch (error) {
      if (error instanceof ApiError) {
        setMessage(error.message);
      } else {
        setMessage(error instanceof Error ? error.message : "Sign-in failed");
      }
    } finally {
      setBusy(false);
      setBusyLabel(null);
    }
  };

  const copyWalletAddress = async () => {
    if (!publicKey) return;
    try {
      await navigator.clipboard.writeText(publicKey.toBase58());
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      setMessage("Could not copy wallet address");
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.gradientBg} aria-hidden />
      <div className={styles.navWrap}>
        <nav className={styles.nav}>
          <Link aria-label="Luma Home" href="/" className={styles.logoLink}>
            <LumaWordmark className={styles.logoWordmark} />
          </Link>
          <div className={styles.right}>
            <span className={styles.time}>{timeLabel || "--:--"}</span>
            <a href="#" className={styles.explore}>
              Explore Events ↗
            </a>
            <Link href="/signin" className={styles.signIn}>
              Sign In
            </Link>
          </div>
        </nav>
      </div>

      <main className={styles.main}>
        <section className={styles.card}>
          <div className={styles.iconBubble}>
            <span>↪</span>
          </div>
          <h1>Welcome to Luma</h1>
          <p>Please sign in below.</p>

          <div className={styles.altActions}>
            <button
              type="button"
              className={styles.lightBtn}
              onClick={signInWithGoogleAndWallet}
              disabled={busy}
            >
              <span className={styles.btnIcon}>G</span>
              {busy && busyLabel === "google" ? "Signing in..." : "Sign in with Google"}
            </button>
          </div>

          <div className={styles.walletActions}>
            <button type="button" className={styles.walletMultiBtn} onClick={() => setVisible(true)}>
              {busy && busyLabel === "wallet" ? "Signing in with wallet..." : walletLabel}
            </button>
            {connected && publicKey ? (
              <div className={styles.walletMeta}>
                <div className={styles.walletMetaRow}>
                  <span>Wallet</span>
                  <span className={styles.walletAddressWrap}>
                    {publicKey.toBase58().slice(0, 6)}...{publicKey.toBase58().slice(-6)}
                    <button type="button" className={styles.copyBtn} onClick={() => void copyWalletAddress()}>
                      {copied ? "Copied" : "Copy"}
                    </button>
                  </span>
                </div>
                <div className={styles.walletMetaRow}>
                  <span>Provider</span>
                  <span>{wallet?.adapter?.name ?? "Unknown"}</span>
                </div>
                <div className={styles.walletMetaRow}>
                  <span>Balance</span>
                  <span>{balanceSol} SOL</span>
                </div>
                <button
                  type="button"
                  className={styles.disconnectBtn}
                  onClick={() => void handleDisconnectWallet()}
                >
                  Disconnect Wallet
                </button>
              </div>
            ) : null}
          </div>

          {message ? <div className={styles.errorText}>{message}</div> : null}
        </section>
      </main>
    </div>
  );
}
