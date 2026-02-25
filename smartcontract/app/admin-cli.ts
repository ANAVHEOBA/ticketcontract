import { readFileSync } from "fs";
import { resolve } from "path";

type AnchorIdl = {
  instructions?: Array<{ name: string }>;
};

const idlPath = resolve(__dirname, "idl", "ticketing_core.json");
const idl: AnchorIdl = JSON.parse(readFileSync(idlPath, "utf8"));
const names = new Set((idl.instructions ?? []).map((ix) => ix.name));

const required = [
  "initializeProtocol",
  "createOrganizer",
  "createEvent",
  "createTicketClass",
  "buyTicket",
  "setResalePolicy",
  "listTicket",
  "buyResaleTicket",
  "createFinancingOffer",
  "acceptFinancingOffer",
  "disburseAdvance",
  "settlePrimaryRevenue",
  "finalizeSettlement",
];

const missing = required.filter((name) => !names.has(name));
if (missing.length > 0) {
  console.error("IDL missing required instructions:", missing.join(", "));
  process.exit(1);
}

console.log("IDL sanity passed for admin flows.");
