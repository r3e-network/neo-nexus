import { request as httpRequest, type IncomingMessage } from "node:http";
import { request as httpsRequest } from "node:https";
import { isIP } from "node:net";
import { Errors } from "../../api/errors";
import { assertLiteralPublicTarget, assertResolvedPublicTarget, defaultHostnameResolver } from "../../utils/outboundTargets";

export interface PinnedRequestInit {
  method?: string;
  headers?: Record<string, string>;
  body?: string | Buffer;
  signal: AbortSignal;
}

/**
 * Issue an HTTPS/HTTP request to a user-supplied URL, pinned to the resolved
 * public IP and returning the underlying IncomingMessage stream so the caller
 * can consume server-sent events as they arrive. The pinning logic mirrors
 * publicFetch but the body stays unbuffered.
 */
export async function pinnedStreamingRequest(url: string, init: PinnedRequestInit): Promise<IncomingMessage> {
  const parsed = new URL(url);
  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error("Streaming URL must use http or https");
  }

  const allowPrivate = process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS === "true";
  assertLiteralPublicTarget(parsed.hostname, Errors.integrationUrlPrivateTarget, allowPrivate);
  const addresses = await assertResolvedPublicTarget(
    parsed.hostname,
    defaultHostnameResolver,
    Errors.integrationUrlPrivateTarget,
    allowPrivate,
  );
  const targetAddress = addresses[0]?.address;
  if (!targetAddress) {
    throw new Error(`Unable to resolve outbound target: ${parsed.hostname}`);
  }

  const request = parsed.protocol === "https:" ? httpsRequest : httpRequest;
  const headers: Record<string, string> = { ...(init.headers ?? {}) };
  headers.Host = parsed.host;

  return new Promise<IncomingMessage>((resolve, reject) => {
    const req = request(
      {
        protocol: parsed.protocol,
        hostname: targetAddress,
        port: parsed.port || (parsed.protocol === "https:" ? "443" : "80"),
        path: `${parsed.pathname}${parsed.search}`,
        method: init.method ?? "GET",
        headers,
        servername: isIP(stripBrackets(parsed.hostname)) === 0 ? stripBrackets(parsed.hostname) : undefined,
      },
      (res) => {
        cleanupAbort();
        resolve(res);
      },
    );

    const onAbort = () => req.destroy(new Error("Request aborted"));
    const cleanupAbort = () => init.signal.removeEventListener("abort", onAbort);

    req.on("error", (error) => {
      cleanupAbort();
      reject(error);
    });

    if (init.signal.aborted) {
      onAbort();
      return;
    }
    init.signal.addEventListener("abort", onAbort, { once: true });

    if (init.body !== undefined && init.body !== null) {
      req.write(init.body);
    }
    req.end();
  });
}

function stripBrackets(hostname: string): string {
  return hostname.replace(/^\[/, "").replace(/\]$/, "");
}

/**
 * Consume an SSE response stream and yield each event's data string. SSE event
 * boundaries are marked by a blank line; lines beginning with "data:" are
 * collected and returned without the prefix. Comments (":" lines) and other
 * field types (event:, id:, retry:) are ignored — the providers we target
 * only care about the data field.
 */
export async function* sseEvents(stream: IncomingMessage, signal: AbortSignal): AsyncGenerator<string, void, void> {
  let buffer = "";
  for await (const chunk of stream) {
    if (signal.aborted) return;
    buffer += Buffer.isBuffer(chunk) ? chunk.toString("utf8") : String(chunk);
    let idx: number;
    while ((idx = buffer.indexOf("\n\n")) !== -1) {
      const event = buffer.slice(0, idx);
      buffer = buffer.slice(idx + 2);
      const data = collectDataField(event);
      if (data !== null) {
        yield data;
      }
    }
  }
  // Flush a trailing event without final newline.
  if (buffer.length > 0) {
    const data = collectDataField(buffer);
    if (data !== null) yield data;
  }
}

function collectDataField(event: string): string | null {
  const lines = event.split(/\r?\n/);
  const dataLines: string[] = [];
  for (const line of lines) {
    if (line.startsWith("data:")) {
      dataLines.push(line.slice(5).replace(/^\s/, ""));
    }
  }
  return dataLines.length > 0 ? dataLines.join("\n") : null;
}
