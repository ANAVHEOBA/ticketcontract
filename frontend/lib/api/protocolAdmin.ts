import { postSim, postTx } from "./onchain";
import type { SimResponseBase, TxRequestBase } from "./types";

type AdminAction = {
  action: string;
  signature: string;
  confirmation_status?: string;
};

const tx = (path: string, token: string, payload: TxRequestBase) =>
  postTx<AdminAction>(path, token, payload);

const sim = (path: string, token: string, payload: TxRequestBase) =>
  postSim<SimResponseBase>(path, token, payload);

export const initializeProtocol = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/initialize-protocol", t, p);
export const simulateInitializeProtocol = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/initialize-protocol/simulate", t, p);

export const pauseProtocol = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/pause-protocol", t, p);
export const simulatePauseProtocol = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/pause-protocol/simulate", t, p);

export const setProtocolConfig = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/set-protocol-config", t, p);
export const simulateSetProtocolConfig = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/set-protocol-config/simulate", t, p);

export const registerProtocolVaults = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/register-protocol-vaults", t, p);
export const simulateRegisterProtocolVaults = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/register-protocol-vaults/simulate", t, p);

export const setProtocolAuthorities = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/set-protocol-authorities", t, p);
export const simulateSetProtocolAuthorities = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/set-protocol-authorities/simulate", t, p);

export const setMultisigConfig = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/set-multisig-config", t, p);
export const simulateSetMultisigConfig = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/set-multisig-config/simulate", t, p);

export const setTimelockDelay = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/set-timelock-delay", t, p);
export const simulateSetTimelockDelay = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/set-timelock-delay/simulate", t, p);

export const queueProtocolConfigChange = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/queue-protocol-config-change", t, p);
export const simulateQueueProtocolConfigChange = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/queue-protocol-config-change/simulate", t, p);

export const executeProtocolConfigChange = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/execute-protocol-config-change", t, p);
export const simulateExecuteProtocolConfigChange = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/execute-protocol-config-change/simulate", t, p);

export const beginUpgradeAuthorityHandoff = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/begin-upgrade-authority-handoff", t, p);
export const simulateBeginUpgradeAuthorityHandoff = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/begin-upgrade-authority-handoff/simulate", t, p);

export const acceptUpgradeAuthorityHandoff = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/accept-upgrade-authority-handoff", t, p);
export const simulateAcceptUpgradeAuthorityHandoff = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/accept-upgrade-authority-handoff/simulate", t, p);

export const emergencyRotateAdmin = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/emergency-rotate-admin", t, p);
export const simulateEmergencyRotateAdmin = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/emergency-rotate-admin/simulate", t, p);

export const setGlobalLoyaltyMultiplier = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/set-global-loyalty-multiplier", t, p);
export const simulateSetGlobalLoyaltyMultiplier = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/set-global-loyalty-multiplier/simulate", t, p);

export const grantRole = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/grant-role", t, p);
export const simulateGrantRole = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/grant-role/simulate", t, p);

export const revokeRole = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/revoke-role", t, p);
export const simulateRevokeRole = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/revoke-role/simulate", t, p);

export const rotateAuthority = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/rotate-authority", t, p);
export const simulateRotateAuthority = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/rotate-authority/simulate", t, p);

export const initializeVault = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/initialize-vault", t, p);
export const simulateInitializeVault = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/initialize-vault/simulate", t, p);

export const snapshotVault = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/snapshot-vault", t, p);
export const simulateSnapshotVault = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/snapshot-vault/simulate", t, p);

export const withdrawVault = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/withdraw-vault", t, p);
export const simulateWithdrawVault = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/withdraw-vault/simulate", t, p);

export const upsertRegistryEntry = (t: string, p: TxRequestBase) =>
  tx("/protocol-admin/upsert-registry-entry", t, p);
export const simulateUpsertRegistryEntry = (t: string, p: TxRequestBase) =>
  sim("/protocol-admin/upsert-registry-entry/simulate", t, p);
