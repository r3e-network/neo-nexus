import { request as httpRequest } from "node:http";
import { request as httpsRequest } from "node:https";
import { isIP } from "node:net";
import type { ApiError } from "../api/errors";
import {
  assertLiteralPublicTarget,
  assertResolvedPublicTarget,
  type HostnameResolver,
} from "./outboundTargets";

export interface PublicFetchOptions {
  resolveHostname: HostnameResolver;
  makeError: (hostname: string) => ApiError;
  allowPrivateTarget: boolean;
}

export async function publicFetch(input: string | URL, init: RequestInit | undefined, options: PublicFetchOptions): Promise<Response> {
  const url = new URL(input.toString());
  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("Outbound URL must use http or https");
  }

  assertLiteralPublicTarget(url.hostname, options.makeError, options.allowPrivateTarget);
  const addresses = await assertResolvedPublicTarget(
    url.hostname,
    options.resolveHostname,
    options.makeError,
    options.allowPrivateTarget,
  );
  const targetAddress = addresses[0]?.address;
  if (!targetAddress) {
    throw new Error(`Unable to resolve outbound target: ${url.hostname}`);
  }

  return requestPinnedTarget(url, targetAddress, init);
}

function requestPinnedTarget(url: URL, targetAddress: string, init: RequestInit | undefined): Promise<Response> {
  return new Promise((resolve, reject) => {
    const request = url.protocol === "https:" ? httpsRequest : httpRequest;
    const headers = headersToRecord(init?.headers);
    setPinnedHostHeader(headers, url.host);

    const req = request({
      protocol: url.protocol,
      hostname: targetAddress,
      port: url.port || (url.protocol === "https:" ? "443" : "80"),
      path: `${url.pathname}${url.search}`,
      method: init?.method ?? "GET",
      headers,
      servername: isIP(stripBrackets(url.hostname)) === 0 ? stripBrackets(url.hostname) : undefined,
    }, (res) => {
      const chunks: Buffer[] = [];
      res.on("error", (error) => {
        cleanupAbortListener();
        reject(error);
      });
      res.on("data", (chunk: Buffer | string) => {
        chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
      });
      res.on("end", () => {
        cleanupAbortListener();
        try {
          const status = res.statusCode ?? 599;
          const body = status === 204 || status === 304 ? null : Buffer.concat(chunks);
          resolve(new Response(body, {
            status,
            statusText: res.statusMessage,
            headers: responseHeadersToHeaders(res.headers),
          }));
        } catch (error) {
          reject(error);
        }
      });
    });

    const onAbort = () => {
      req.destroy(new Error("Request aborted"));
    };
    const cleanupAbortListener = () => {
      init?.signal?.removeEventListener("abort", onAbort);
    };

    req.on("error", (error) => {
      cleanupAbortListener();
      reject(error);
    });

    if (init?.signal?.aborted) {
      onAbort();
      return;
    }
    init?.signal?.addEventListener("abort", onAbort, { once: true });

    const body = init?.body;
    if (body !== undefined && body !== null) {
      if (typeof body === "string" || Buffer.isBuffer(body) || body instanceof Uint8Array) {
        req.write(body);
      } else {
        req.destroy(new Error("Unsupported outbound request body type"));
        return;
      }
    }
    req.end();
  });
}

function headersToRecord(headers: RequestInit["headers"] | undefined): Record<string, string> {
  const result: Record<string, string> = {};
  if (!headers) {
    return result;
  }

  new Headers(headers).forEach((value, key) => {
    result[key] = value;
  });
  return result;
}

function setPinnedHostHeader(headers: Record<string, string>, host: string): void {
  for (const key of Object.keys(headers)) {
    if (key.toLowerCase() === "host") {
      delete headers[key];
    }
  }
  headers.Host = host;
}

function responseHeadersToHeaders(headers: Record<string, string | string[] | number | undefined>): Headers {
  const result = new Headers();
  for (const [key, value] of Object.entries(headers)) {
    if (Array.isArray(value)) {
      for (const entry of value) {
        result.append(key, entry);
      }
    } else if (value !== undefined) {
      result.set(key, String(value));
    }
  }
  return result;
}

function stripBrackets(hostname: string): string {
  return hostname.replace(/^\[/, "").replace(/\]$/, "");
}
