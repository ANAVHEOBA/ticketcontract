import Link from "next/link";
import { LumaWordmark } from "./LumaLogo";
import styles from "./Footer.module.css";

const social = ["Email", "iOS", "X", "Instagram"];

export function Footer() {
  return (
    <footer className={styles.footer}>
      <div className={styles.topRow}>
        <div className={styles.left}>
          <Link aria-label="Luma Home" href="/" className={styles.wordmarkLink}>
            <LumaWordmark className={styles.wordmark} />
          </Link>
          <div className={styles.links}>
            <a href="#">Discover</a>
            <a href="#">Pricing</a>
            <a href="#">Help</a>
          </div>
        </div>
        <div className={styles.social}>
          {social.map((label) => (
            <a key={label} href="#" aria-label={label}>
              {label}
            </a>
          ))}
        </div>
      </div>
      <div className={styles.bottomRow}>
        <a href="#">Terms</a>
        <a href="#">Privacy</a>
        <a href="#">Security</a>
      </div>
    </footer>
  );
}
