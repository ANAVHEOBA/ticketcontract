import styles from "./Flow.module.css";

const steps = [
  "Create organizer + event + ticket classes",
  "Run underwriting and accept financing terms",
  "Publish resale policy and enforce on-chain",
  "Settle revenue with idempotent references",
  "Track trust, loyalty, and ops alerts",
];

export function Flow() {
  return (
    <section id="flow" className={styles.flow}>
      <div className={styles.heading}>
        <p>End-to-End Flow</p>
        <h2>From event launch to settlement finalization.</h2>
      </div>
      <ol className={styles.timeline}>
        {steps.map((step, index) => (
          <li key={step}>
            <span>{String(index + 1).padStart(2, "0")}</span>
            <p>{step}</p>
          </li>
        ))}
      </ol>
    </section>
  );
}
