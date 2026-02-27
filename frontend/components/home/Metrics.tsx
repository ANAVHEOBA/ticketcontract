import styles from "./Metrics.module.css";

const items = [
  { label: "Tx Success", value: "99.1%" },
  { label: "Avg Confirm", value: "1.6s" },
  { label: "Indexer Lag", value: "0.8s" },
  { label: "Sponsored Gas", value: "9 SOL" },
];

export function Metrics() {
  return (
    <section id="ops" className={styles.metrics}>
      {items.map((item) => (
        <article key={item.label} className={styles.metricCard}>
          <p>{item.label}</p>
          <h3>{item.value}</h3>
        </article>
      ))}
    </section>
  );
}
