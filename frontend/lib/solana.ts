import { PublicKey } from "@solana/web3.js";

export const DEFAULT_PROGRAM_ID = "Evb4oQnwYDZhabE5E5tuJPg2yvCMGgwMRgkxzzaUw5Rv";

export function getProgramId(): PublicKey {
  return new PublicKey(process.env.NEXT_PUBLIC_PROGRAM_ID ?? DEFAULT_PROGRAM_ID);
}

export function deriveOrganizerPda(authority: PublicKey, programId: PublicKey): PublicKey {
  const [organizerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("organizer"), authority.toBuffer()],
    programId,
  );
  return organizerPda;
}

export function readPubkey(data: Buffer, offset: number): PublicKey {
  return new PublicKey(data.subarray(offset, offset + 32));
}

function dataViewFor(data: Buffer): DataView {
  return new DataView(data.buffer, data.byteOffset, data.byteLength);
}

function readU16LE(data: Buffer, offset: number): number {
  return dataViewFor(data).getUint16(offset, true);
}

function readU32LE(data: Buffer, offset: number): number {
  return dataViewFor(data).getUint32(offset, true);
}

function readU64LE(data: Buffer, offset: number): bigint {
  return dataViewFor(data).getBigUint64(offset, true);
}

function readI64LE(data: Buffer, offset: number): bigint {
  return dataViewFor(data).getBigInt64(offset, true);
}

export function decodeProtocolFeeVault(data: Buffer): PublicKey {
  if (data.length < 8 + 335) {
    throw new Error("protocol config account data too short");
  }
  // Skip anchor discriminator (8). Layout offset from Rust struct:
  // bump(1) + admin(32) + upgrade_authority(32) + pending_upgrade_authority(32)
  // + handoff_started(8) + handoff_eta(8) + timelock(8)
  // + pending_fee_bps(2) + pending_max_tickets(2) + config_eta(8)
  // + multisig_enabled(1) + threshold(1)
  // + signer1(32) + signer2(32) + signer3(32) + emergency_admin(32)
  // + emergency_nonce(8) + treasury_vault(32) = 303 bytes from start of struct.
  const feeVaultOffset = 8 + 303;
  return readPubkey(data, feeVaultOffset);
}

export function decodeOrganizerPayoutWallet(data: Buffer): PublicKey {
  if (data.length < 8 + 65) {
    throw new Error("organizer profile account data too short");
  }
  // discriminator + bump + authority
  const payoutOffset = 8 + 1 + 32;
  return readPubkey(data, payoutOffset);
}

export function decodeResalePolicyRoyaltyVault(data: Buffer): PublicKey {
  if (data.length < 8 + 155) {
    throw new Error("resale policy account data too short");
  }
  // discriminator + bump + schema_version + deprecated_layout_version + replacement_account
  // + deprecated_at + event + ticket_class + class_id + max_markup_bps + royalty_bps
  const royaltyVaultOffset = 8 + 1 + 2 + 2 + 32 + 8 + 32 + 32 + 2 + 2 + 2;
  return readPubkey(data, royaltyVaultOffset);
}

export type ParsedListing = {
  event: PublicKey;
  ticketClass: PublicKey;
  ticket: PublicKey;
  seller: PublicKey;
  priceLamports: bigint;
  expiresAt: bigint;
  isActive: boolean;
};

export function decodeListing(data: Buffer): ParsedListing {
  if (data.length < 8 + 147) {
    throw new Error("listing account data too short");
  }
  let offset = 8; // discriminator
  offset += 1; // bump
  const event = readPubkey(data, offset);
  offset += 32;
  const ticketClass = readPubkey(data, offset);
  offset += 32;
  const ticket = readPubkey(data, offset);
  offset += 32;
  const seller = readPubkey(data, offset);
  offset += 32;
  const priceLamports = readU64LE(data, offset);
  offset += 8;
  const expiresAt = readI64LE(data, offset);
  offset += 8;
  const isActive = data[offset] === 1;

  return {
    event,
    ticketClass,
    ticket,
    seller,
    priceLamports,
    expiresAt,
    isActive,
  };
}

export type ParsedTicketClass = {
  classId: number;
  soldSupply: number;
  facePriceLamports: bigint;
  stakeholderWallet: PublicKey;
  stakeholderBps: number;
};

export function decodeTicketClass(data: Buffer): ParsedTicketClass {
  if (data.length < 8 + 1 + 32 + 2 + 4) {
    throw new Error("ticket class account data too short");
  }

  let offset = 8; // discriminator
  offset += 1; // bump
  offset += 32; // event
  const classId = readU16LE(data, offset);
  offset += 2;

  const nameLen = readU32LE(data, offset);
  offset += 4 + nameLen;

  offset += 4; // total_supply
  offset += 4; // reserved_supply
  const soldSupply = readU32LE(data, offset);
  offset += 4;
  offset += 4; // refunded_supply
  offset += 4; // remaining_supply

  const facePriceLamports = readU64LE(data, offset);
  offset += 8;
  offset += 8; // sale_start_ts
  offset += 8; // sale_end_ts
  offset += 2; // per_wallet_limit
  offset += 1; // is_transferable
  offset += 1; // is_resale_enabled
  offset += 1; // allow_reentry
  offset += 1; // max_reentries

  const stakeholderWallet = readPubkey(data, offset);
  offset += 32;
  const stakeholderBps = readU16LE(data, offset);

  return { classId, soldSupply, facePriceLamports, stakeholderWallet, stakeholderBps };
}

export type ParsedTicket = {
  event: PublicKey;
  ticketClass: PublicKey;
  owner: PublicKey;
  buyer: PublicKey;
  ticketId: number;
  status: number;
  paidAmountLamports: bigint;
  isComp: boolean;
  createdAt: bigint;
};

export function decodeTicket(data: Buffer): ParsedTicket {
  if (data.length < 8 + 203) {
    throw new Error("ticket account data too short");
  }

  let offset = 8; // discriminator
  offset += 1; // bump
  offset += 2; // schema_version
  offset += 2; // deprecated_layout_version
  offset += 32; // replacement_account
  offset += 8; // deprecated_at

  const event = readPubkey(data, offset);
  offset += 32;
  const ticketClass = readPubkey(data, offset);
  offset += 32;
  const owner = readPubkey(data, offset);
  offset += 32;
  const buyer = readPubkey(data, offset);
  offset += 32;

  const ticketId = readU32LE(data, offset);
  offset += 4;
  const status = data[offset];
  offset += 1;
  const paidAmountLamports = readU64LE(data, offset);
  offset += 8;
  const isComp = data[offset] === 1;
  offset += 1;
  const createdAt = readI64LE(data, offset);

  return {
    event,
    ticketClass,
    owner,
    buyer,
    ticketId,
    status,
    paidAmountLamports,
    isComp,
    createdAt,
  };
}

export const TICKET_OWNER_OFFSET = 117;
