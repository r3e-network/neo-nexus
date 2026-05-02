import { Errors } from "../api/errors";
import { assertLiteralPublicTarget, defaultHostnameResolver } from "../utils/outboundTargets";
import { publicFetch } from "../utils/publicFetch";

const allowPrivateIntegrationTargets = () => process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS === "true";

export function validateLiteralIntegrationUrl(rawUrl: string): void {
  const parsed = parseIntegrationUrl(rawUrl);
  assertLiteralPublicTarget(parsed.hostname, Errors.integrationUrlPrivateTarget, allowPrivateIntegrationTargets());
}

export async function safeIntegrationFetch(input: string | URL, init?: RequestInit): Promise<Response> {
  const url = requestUrl(input);
  const parsed = parseIntegrationUrl(url);
  const allowPrivate = allowPrivateIntegrationTargets();

  return publicFetch(parsed, init, {
    resolveHostname: defaultHostnameResolver,
    makeError: Errors.integrationUrlPrivateTarget,
    allowPrivateTarget: allowPrivate,
  });
}

function parseIntegrationUrl(rawUrl: string): URL {
  let parsed: URL;
  try {
    parsed = new URL(rawUrl);
  } catch {
    throw new Error(`Invalid integration URL: ${rawUrl}`);
  }

  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw new Error("Integration URL must use http or https");
  }

  return parsed;
}

function requestUrl(input: string | URL): string {
  if (typeof input === "string") {
    return input;
  }
  return input.toString();
}
