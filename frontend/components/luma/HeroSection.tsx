import { LumaWordmark } from "./LumaLogo";
import styles from "./HeroSection.module.css";

export function HeroSection() {
  return (
    <section className={styles.hero}>
      <div className={styles.copyCol}>
        <LumaWordmark className={styles.wordmark} />
        <h1>
          <span>Delightful</span>
          <br />
          <span>events</span>
          <br />
          <em>start here.</em>
        </h1>
        <p>Set up an event page, invite friends and sell tickets. Host a memorable event today.</p>
        <button className={styles.cta}>Create Your First Event</button>
      </div>
      <div className={styles.mediaCol}>
        <video
          className={styles.phoneVideo}
          width="100%"
          height="100%"
          loop
          muted
          playsInline
          autoPlay
        >
          <source src="https://cdn.lu.ma/landing/phone-dark.webm" type="video/webm" />
          <source src="https://cdn.lu.ma/landing/phone-dark.mp4" type="video/mp4" />
        </video>
      </div>
    </section>
  );
}
