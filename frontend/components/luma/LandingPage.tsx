import { Footer } from "./Footer";
import { HeroSection } from "./HeroSection";
import { NavBar } from "./NavBar";
import styles from "./LandingPage.module.css";

export function LandingPage() {
  return (
    <div className={styles.page}>
      <div className={styles.gradientBg} aria-hidden />

      <NavBar />
      <main className={styles.content}>
        <HeroSection />
      </main>
      <Footer />
    </div>
  );
}
