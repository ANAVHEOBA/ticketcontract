import type { Metadata } from "next";
import { Sora, Space_Grotesk } from "next/font/google";
import { SolanaWalletProvider } from "@/components/providers/SolanaWalletProvider";
import "./globals.css";

const display = Sora({
  variable: "--font-display",
  subsets: ["latin"],
});

const body = Space_Grotesk({
  variable: "--font-body",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "TicketControl",
  description: "On-chain ticketing with financial control and sponsored gas UX.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${display.variable} ${body.variable} antialiased`}>
        <SolanaWalletProvider>{children}</SolanaWalletProvider>
      </body>
    </html>
  );
}
