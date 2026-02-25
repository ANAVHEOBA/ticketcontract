#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const idlPath = path.resolve(__dirname, "idl", "ticketing_core.json");
if (!fs.existsSync(idlPath)) {
  console.error(`[sanity] missing IDL: ${idlPath}`);
  process.exit(1);
}

const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
const names = new Set((idl.instructions || []).map((ix) => ix.name));

const backendPath = [
  "initialize_protocol",
  "create_organizer",
  "create_event",
  "create_ticket_class",
  "create_financing_offer",
  "accept_financing_offer",
  "disburse_advance",
  "settle_primary_revenue",
  "finalize_settlement",
];

const frontendPath = [
  "buy_ticket",
  "set_ticket_metadata",
  "set_resale_policy",
  "list_ticket",
  "buy_resale_ticket",
  "check_in_ticket",
  "accrue_points",
  "redeem_points",
];

function assertAllPresent(groupName, required) {
  const missing = required.filter((ix) => !names.has(ix));
  if (missing.length > 0) {
    console.error(`[sanity] ${groupName} missing instructions: ${missing.join(", ")}`);
    process.exit(1);
  }
}

assertAllPresent("backend", backendPath);
assertAllPresent("frontend", frontendPath);

console.log("[sanity] IDL includes required backend + frontend call paths.");
