import styles from "./FooterCta.module.css";

export function FooterCta() {
  return (
    <section className={styles.cta}>
      <h2>Ready to ship the fan app and organizer console.</h2>
      <p>All backend modules are wired. You can now focus on components and wallet UX.</p>
      <button>Start Building UI</button>
    </section>
  );
}
