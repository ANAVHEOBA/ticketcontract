export type TxRequestBase = {
  transaction_base64: string;
  skip_preflight?: boolean;
  max_retries?: number;
  timeout_ms?: number;
  poll_ms?: number;
};

export type SimRequestBase = {
  transaction_base64: string;
  sig_verify?: boolean;
  replace_recent_blockhash?: boolean;
};

export type ActionResponseBase = {
  action: string;
  signature: string;
  confirmation_status?: string;
};

export type SimResponseBase = {
  action: string;
  err?: unknown;
  logs: string[];
  units_consumed?: number;
};

export type AuthTokenResponse = {
  access_token: string;
  token_type: string;
  expires_in: number;
  role: string;
  organizer_scopes: string[];
};
