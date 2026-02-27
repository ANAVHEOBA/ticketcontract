export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export class ApiError extends Error {
  status: number;
  code?: string;

  constructor(status: number, message: string, code?: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080/api/v1";

export type RequestOptions = {
  method?: HttpMethod;
  body?: unknown;
  token?: string;
  headers?: Record<string, string>;
};

export async function request<T>(
  path: string,
  { method = "GET", body, token, headers = {} }: RequestOptions = {}
): Promise<T> {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...headers,
    },
    body: body === undefined ? undefined : JSON.stringify(body),
    cache: "no-store",
  });

  const contentType = res.headers.get("content-type") || "";
  const payload = contentType.includes("application/json")
    ? await res.json()
    : await res.text();

  if (!res.ok) {
    const message =
      typeof payload === "object" && payload && "message" in payload
        ? String(payload.message)
        : `Request failed with status ${res.status}`;
    const code =
      typeof payload === "object" && payload && "code" in payload
        ? String(payload.code)
        : undefined;
    throw new ApiError(res.status, message, code);
  }

  return payload as T;
}

export async function requestText(path: string): Promise<string> {
  const res = await fetch(`${API_BASE_URL}${path}`, { cache: "no-store" });
  if (!res.ok) {
    throw new ApiError(res.status, `Request failed with status ${res.status}`);
  }
  return res.text();
}
