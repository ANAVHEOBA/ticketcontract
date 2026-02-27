"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { LumaBurstIcon } from "./LumaLogo";
import styles from "./NavBar.module.css";

function formatLocalTime() {
  return new Intl.DateTimeFormat("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
    timeZoneName: "short",
  }).format(new Date());
}

export function NavBar() {
  const router = useRouter();
  const { connection } = useConnection();
  const { disconnect } = useWallet();
  const [timeLabel, setTimeLabel] = useState("");
  const [authedWallet, setAuthedWallet] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [balanceLabel, setBalanceLabel] = useState("-");
  const [disconnecting, setDisconnecting] = useState(false);
  const menuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const syncAuth = () => {
      const token = localStorage.getItem("ticketing_access_token");
      const wallet = localStorage.getItem("ticketing_wallet");
      if (token && wallet) {
        setAuthedWallet(wallet);
        return;
      }
      setAuthedWallet(null);
    };

    syncAuth();
    window.addEventListener("storage", syncAuth);
    return () => window.removeEventListener("storage", syncAuth);
  }, []);

  const authLabel = useMemo(
    () => (authedWallet ? `${authedWallet.slice(0, 4)}...${authedWallet.slice(-4)}` : "Sign In"),
    [authedWallet],
  );

  useEffect(() => {
    const update = () => setTimeLabel(formatLocalTime());
    update();
    const timer = window.setInterval(update, 60_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    let cancelled = false;
    const readBalance = async () => {
      if (!authedWallet) {
        setBalanceLabel("-");
        return;
      }
      try {
        const walletKey = new PublicKey(authedWallet);
        const lamports = await connection.getBalance(walletKey, "confirmed");
        if (!cancelled) {
          setBalanceLabel((lamports / LAMPORTS_PER_SOL).toFixed(4));
        }
      } catch {
        if (!cancelled) {
          setBalanceLabel("-");
        }
      }
    };

    void readBalance();
    return () => {
      cancelled = true;
    };
  }, [authedWallet, connection]);

  useEffect(() => {
    const onPointerDown = (event: MouseEvent) => {
      if (!menuRef.current) return;
      if (!menuRef.current.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    };

    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, []);

  const copyAddress = useCallback(async () => {
    if (!authedWallet) return;
    try {
      await navigator.clipboard.writeText(authedWallet);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      setCopied(false);
    }
  }, [authedWallet]);

  const handleDisconnect = useCallback(async () => {
    setDisconnecting(true);
    try {
      await disconnect();
    } catch {
      // ignore wallet adapter disconnect errors
    } finally {
      localStorage.removeItem("ticketing_access_token");
      localStorage.removeItem("ticketing_role");
      localStorage.removeItem("ticketing_wallet");
      localStorage.removeItem("ticketing_organizer_scopes");
      setAuthedWallet(null);
      setMenuOpen(false);
      setDisconnecting(false);
      router.push("/signin");
    }
  }, [disconnect, router]);

  return (
    <header className={styles.wrapper}>
      <nav className={styles.nav}>
        <Link aria-label="Luma Home" href="/" className={styles.logoLink}>
          <LumaBurstIcon className={styles.logoIcon} />
        </Link>
        <div className={styles.right}>
          <span className={styles.time}>{timeLabel || "--:--"}</span>
          <a href="#" className={styles.explore}>
            Explore Events
          </a>
          {authedWallet ? (
            <div className={styles.walletMenuWrap} ref={menuRef}>
              <button
                type="button"
                className={styles.signIn}
                onClick={() => setMenuOpen((open) => !open)}
                aria-expanded={menuOpen}
                aria-haspopup="menu"
              >
                {authLabel}
              </button>
              {menuOpen ? (
                <div className={styles.walletMenu} role="menu" aria-label="Wallet menu">
                  <div className={styles.walletRow}>
                    <span>Address</span>
                    <span className={styles.walletValue}>{authLabel}</span>
                  </div>
                  <div className={styles.walletRow}>
                    <span>Balance</span>
                    <span className={styles.walletValue}>{balanceLabel} SOL</span>
                  </div>
                  <button type="button" className={styles.menuBtn} onClick={() => void copyAddress()}>
                    {copied ? "Copied" : "Copy address"}
                  </button>
                  <button
                    type="button"
                    className={styles.menuBtn}
                    onClick={() => void handleDisconnect()}
                    disabled={disconnecting}
                  >
                    {disconnecting ? "Disconnecting..." : "Disconnect"}
                  </button>
                </div>
              ) : null}
            </div>
          ) : (
            <Link href="/signin" className={styles.signIn}>
              Sign In
            </Link>
          )}
        </div>
      </nav>
    </header>
  );
}
