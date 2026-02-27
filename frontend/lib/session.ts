export type AuthSession = {
  token: string | null;
  wallet: string | null;
  role: string | null;
  organizerScopes: string[];
};

const TOKEN_KEY = "ticketing_access_token";
const WALLET_KEY = "ticketing_wallet";
const ROLE_KEY = "ticketing_role";
const SCOPES_KEY = "ticketing_organizer_scopes";

function safeParseScopes(raw: string | null): string[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((value): value is string => typeof value === "string");
  } catch {
    return [];
  }
}

export function readAuthSession(): AuthSession {
  if (typeof window === "undefined") {
    return { token: null, wallet: null, role: null, organizerScopes: [] };
  }

  return {
    token: window.localStorage.getItem(TOKEN_KEY),
    wallet: window.localStorage.getItem(WALLET_KEY),
    role: window.localStorage.getItem(ROLE_KEY),
    organizerScopes: safeParseScopes(window.localStorage.getItem(SCOPES_KEY)),
  };
}

export function hasOrganizerScope(scopes: string[], organizerId: string): boolean {
  return scopes.includes("*") || scopes.includes(organizerId);
}
