smartcontract/
    Anchor.toml
    Cargo.toml
    package.json
    tsconfig.json
    .gitignore
    .prettierignore

    programs/
      ticketing_core/
        Cargo.toml
        Xargo.toml
        src/
          lib.rs                     # entrypoints + instruction dispatch
          constants.rs               # seeds, limits, error domains
          error.rs                   # compact custom errors
          events.rs                  # indexed events only

          instructions/
            mod.rs
            protocol/
              mod.rs
              initialize_protocol.rs
              set_protocol_config.rs
              pause_protocol.rs
            organizer/
              mod.rs
              create_organizer.rs
              update_organizer.rs
              set_operator.rs
            event/
              mod.rs
              create_event.rs
              update_event.rs
              freeze_event.rs
              cancel_event.rs
            ticket_class/
              mod.rs
              create_ticket_class.rs
              update_ticket_class.rs
              reserve_inventory.rs
            primary_sale/
              mod.rs
              buy_ticket.rs
              issue_comp_ticket.rs
            resale/
              mod.rs
              set_resale_policy.rs
              list_ticket.rs
              buy_resale_ticket.rs
              cancel_listing.rs
            financing/
              mod.rs
              create_financing_offer.rs
              accept_financing_offer.rs
              disburse_advance.rs
            settlement/
              mod.rs
              settle_primary_revenue.rs
              settle_resale_revenue.rs
              finalize_settlement.rs
            checkin/
              mod.rs
              check_in_ticket.rs
            loyalty/
              mod.rs
              accrue_points.rs
              redeem_points.rs
            disputes/
              mod.rs
              refund_ticket.rs
              flag_dispute.rs
            governance/
              mod.rs
              grant_role.rs
              revoke_role.rs
              rotate_authority.rs

          state/
            mod.rs
            protocol_config.rs
            organizer.rs
            event.rs
            ticket_class.rs
            ticket.rs
            resale_policy.rs
            listing.rs
            financing_offer.rs
            settlement_ledger.rs
            loyalty_ledger.rs
            trust_signal.rs
            role_binding.rs
            vaults.rs

          validation/
            mod.rs
            access.rs                # role gates
            invariants.rs            # state transition checks
            pricing.rs               # price/markup checks
            settlement.rs            # waterfall math checks

          math/
            mod.rs
            safe_math.rs             # checked arithmetic helpers
            watermark.rs             # repayment/waterfall calculations

          utils/
            mod.rs
            pda.rs
            clock.rs
            token.rs                 # SPL/CPI wrappers kept minimal

          migrations/
            mod.rs                   # account version migrations

    tests/
      unit/
        protocol.spec.ts
        event.spec.ts
        primary_sale.spec.ts
        resale.spec.ts
        financing.spec.ts
        settlement.spec.ts
        checkin.spec.ts
        disputes.spec.ts
        loyalty.spec.ts
        governance.spec.ts
      integration/
        end_to_end.spec.ts

    scripts/
      build.sh
      test.sh
      test-validator.sh
      deploy-devnet.sh
      idl-sync.sh


    app/                            # keep minimal for local admin/debug
      admin-cli.ts
