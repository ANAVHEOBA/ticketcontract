"use client";

import Link from "next/link";
import type { ReactNode } from "react";
import { Footer } from "@/components/luma/Footer";
import { NavBar } from "@/components/luma/NavBar";
import styles from "./BackofficeLayout.module.css";

type Props = {
  title: string;
  subtitle: string;
  children: ReactNode;
};

const links = [
  ["/dashboard", "Dashboard"],
  ["/financing", "Financing"],
  ["/settlement", "Settlement"],
  ["/checkin", "Check-in"],
  ["/disputes", "Disputes"],
  ["/loyalty-trust", "Loyalty + Trust"],
  ["/underwriting", "Underwriting"],
  ["/indexer-kpis", "Indexer + KPIs"],
  ["/ops", "Ops + Docs"],
] as const;

export function BackofficeLayout({ title, subtitle, children }: Props) {
  return (
    <div className={styles.page}>
      <div className={styles.background} aria-hidden />
      <NavBar />
      <main className={styles.content}>
        <header className={styles.header}>
          <h1>{title}</h1>
          <p className={styles.subtitle}>{subtitle}</p>
          <div className={styles.links}>
            {links.map(([href, label]) => (
              <Link href={href} key={href} className={styles.link}>
                {label}
              </Link>
            ))}
          </div>
        </header>
        {children}
      </main>
      <Footer />
    </div>
  );
}

export { styles as backofficeStyles };
