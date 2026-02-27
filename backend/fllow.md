  Recommended model:

  1. User signs in wallet for intent/consent

  - buy_ticket, list_ticket, buy_resale_ticket, transfers, accepting financing terms.
  - Anything that changes user ownership/money should have the user signature.

  2. Backend pays gas as fee payer

  - Run a relayer wallet (your backend signer) as feePayer.
  - Backend builds tx, user signs in their wallet, backend co-signs as fee payer and submits.

  3. Backend-only signing for operational flows

  Where user should sign:

  - In frontend wallet (Phantom/Solflare/etc) for user-facing economic actions.
  - Do not sign those actions server-side unless you are doing fully custodial wallets (higher risk/compliance burden).

  Critical safeguards for sponsored gas:

  - Allowlist instructions/program IDs.
  - Per-wallet/day spend caps and rate limits.
  - Simulate before send + max compute/max lamports checks.
  - Nonce/idempotency + replay protection.
  - Separate hot relayer key with limited funds.

  If you want, I can design the exact relayer flow for your current endpoints (/primary-sale/buy, /resale/list, /resale/buy) and the .env keys needed.