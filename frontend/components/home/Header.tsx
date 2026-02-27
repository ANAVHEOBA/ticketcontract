import styles from "./Header.module.css";

export function Header() {
  return (
    <header className={styles.header}>
      <div className={styles.brandRow}>
        <div className={styles.brandMark} />
        <span className={styles.brandText}>TicketControl</span>
      </div>
      <nav className={styles.nav}>
        <a href="#flow">Flow</a>
        <a href="#modules">Modules</a>
        <a href="#ops">Ops</a>
      </nav>
      <div className={styles.actions}>
        <button className={styles.ghost}>Fan App</button>
        <button className={styles.primary}>Organizer Console</button>
      </div>
    </header>
  );
}
