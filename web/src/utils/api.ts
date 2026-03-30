export class ApiRequestError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly suggestion?: string,
    public readonly status?: number,
  ) {
    super(message);
  }
}

type JsonValue = Record<string, unknown> | Array<unknown> | string | number | boolean | null;

function getToken(): string | null {
  if (typeof localStorage === "undefined") {
    return null;
  }

  return localStorage.getItem("token");
}

function resolveUrl(url: string): string {
  if (url.startsWith("/api/") || url.startsWith("http://") || url.startsWith("https://")) {
    return url;
  }

  if (url.startsWith("/")) {
    return `/api${url}`;
  }

  return `/api/${url}`;
}

async function request<T>(url: string, init: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {};
  const token = getToken();

  if (init.body !== undefined) {
    headers["Content-Type"] = "application/json";
  }

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  if (init.headers && typeof init.headers === "object" && !Array.isArray(init.headers)) {
    Object.assign(headers, init.headers);
  }

  const response = await fetch(resolveUrl(url), {
    ...init,
    headers,
  });

  if (!response.ok) {
    let message = "API Error";
    let code: string | undefined;
    let suggestion: string | undefined;

    try {
      const body = (await response.json()) as {
        error?: string;
        code?: string;
        suggestion?: string;
      };
      message = body.error || message;
      code = body.code;
      suggestion = body.suggestion;
    } catch {
      // Response wasn't JSON
    }

    throw new ApiRequestError(message, code, suggestion, response.status);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export const api = {
  get: <T>(url: string) => request<T>(url),
  post: <T>(url: string, data?: JsonValue) =>
    request<T>(url, {
      method: "POST",
      body: data === undefined ? undefined : JSON.stringify(data),
    }),
  put: <T>(url: string, data?: JsonValue) =>
    request<T>(url, {
      method: "PUT",
      body: data === undefined ? undefined : JSON.stringify(data),
    }),
  delete: <T>(url: string) =>
    request<T>(url, {
      method: "DELETE",
    }),
};
