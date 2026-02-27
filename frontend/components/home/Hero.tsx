import styles from "./Hero.module.css";

export function Hero() {
  return (
    <section className={styles.hero}>
      <div className={styles.left}>
        <p className={styles.kicker}>Ticketing + Financial Control Layer</p>
        <h1>
          Run events with <span>on-chain trust</span> and real-time cash control.
        </h1>
        <p className={styles.subcopy}>
          Primary sale, resale rules, underwriting, settlement, and ops telemetry in one flow.
        </p>
        <div className={styles.ctaRow}>
          <button className={styles.primary}>Launch Event</button>
          <button className={styles.secondary}>Simulate Resale Policy</button>
        </div>
      </div>
      <aside className={styles.panel}>
        <div className={styles.panelCard}>
          <p>Cash Position</p>
          <h3>$124,200</h3>
          <span>+11.4% this week</span>
        </div>
        <div className={styles.panelCard}>
          <p>Risk Tier</p>
          <h3>Medium</h3>
          <span>Advance 58% / Fee 4.2%</span>
        </div>
      </aside>
    </section>
  );
}
