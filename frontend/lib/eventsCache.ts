export type CachedTicketClass = {
  classId: number;
  classPda: string;
  eventId: string;
  eventPda: string;
  organizerId: string;
  name: string;
  supply: number;
  priceLamports: number;
  stakeholderWallet: string;
  stakeholderBps: number;
};

export type CachedEvent = {
  eventId: string;
  eventPda: string;
  organizerId: string;
  name: string;
  venue: string;
  startsAtEpoch: number;
  endsAtEpoch: number;
  updatedAtEpoch: number;
};

const EVENTS_KEY = "ticketing_cached_events";
const CLASSES_KEY = "ticketing_cached_classes";

function parseArray<T>(raw: string | null): T[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as unknown;
    return Array.isArray(parsed) ? (parsed as T[]) : [];
  } catch {
    return [];
  }
}

function persist<T>(key: string, value: T[]) {
  window.localStorage.setItem(key, JSON.stringify(value));
}

export function getCachedEvents(): CachedEvent[] {
  if (typeof window === "undefined") return [];
  return parseArray<CachedEvent>(window.localStorage.getItem(EVENTS_KEY));
}

export function upsertCachedEvent(entry: CachedEvent) {
  const current = getCachedEvents();
  const index = current.findIndex((event) => event.eventId === entry.eventId);
  if (index >= 0) {
    current[index] = entry;
  } else {
    current.unshift(entry);
  }
  persist(EVENTS_KEY, current.slice(0, 200));
}

export function getCachedClasses(): CachedTicketClass[] {
  if (typeof window === "undefined") return [];
  return parseArray<CachedTicketClass>(window.localStorage.getItem(CLASSES_KEY));
}

export function upsertCachedClasses(entries: CachedTicketClass[]) {
  const existing = getCachedClasses();
  const map = new Map<string, CachedTicketClass>();
  for (const item of existing) {
    map.set(item.classPda, item);
  }
  for (const item of entries) {
    map.set(item.classPda, item);
  }
  persist(CLASSES_KEY, Array.from(map.values()).slice(0, 500));
}

export function getCachedClassesByEvent(eventId: string): CachedTicketClass[] {
  return getCachedClasses().filter((entry) => entry.eventId === eventId || entry.eventPda === eventId);
}
