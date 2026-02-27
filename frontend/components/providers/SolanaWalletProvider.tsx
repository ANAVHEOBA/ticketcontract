"use client";

import { PropsWithChildren, useMemo } from "react";
import { clusterApiUrl } from "@solana/web3.js";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { SolflareWalletAdapter, TrustWalletAdapter } from "@solana/wallet-adapter-wallets";
import "@solana/wallet-adapter-react-ui/styles.css";

type Cluster = "devnet" | "testnet" | "mainnet-beta";

function resolveEndpoint(): string {
  const cluster = (process.env.NEXT_PUBLIC_SOLANA_CLUSTER ?? "devnet") as Cluster;
  if (cluster === "mainnet-beta" || cluster === "testnet" || cluster === "devnet") {
    return clusterApiUrl(cluster);
  }
  return process.env.NEXT_PUBLIC_SOLANA_RPC_URL ?? clusterApiUrl("devnet");
}

export function SolanaWalletProvider({ children }: PropsWithChildren) {
  const endpoint = useMemo(() => resolveEndpoint(), []);
  const wallets = useMemo(() => [new SolflareWalletAdapter(), new TrustWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
