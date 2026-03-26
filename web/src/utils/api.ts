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
    try {
      const error = (await response.json()) as { error?: string };
      throw new Error(error.error || "API Error");
    } catch (error) {
      throw error instanceof Error ? error : new Error("API Error");
    }
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
