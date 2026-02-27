import styles from "./Modules.module.css";

const modules = [
  "On-chain strict instruction validation",
  "Role-guarded protocol admin controls",
  "Indexer + KPI materialized caches",
  "Underwriting + resale compiler",
  "Ops metrics, alerts, and audit logs",
  "OpenAPI, Postman, and devnet smoke tooling",
];

export function Modules() {
  return (
    <section id="modules" className={styles.modules}>
      {modules.map((mod) => (
        <article key={mod} className={styles.card}>
          <div className={styles.dot} />
          <p>{mod}</p>
        </article>
      ))}
    </section>
  );
}
