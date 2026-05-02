import { lookup } from "node:dns/promises";
import { isIP } from "node:net";
import { ApiError } from "../api/errors";

export interface ResolvedHostnameAddress {
  address: string;
  family: 4 | 6;
}

export type HostnameResolver = (hostname: string) => Promise<ResolvedHostnameAddress[]>;

export const defaultHostnameResolver: HostnameResolver = async (hostname) => {
  const results = await lookup(hostname, { all: true });
  return results.map((result) => ({
    address: result.address,
    family: result.family as 4 | 6,
  }));
};

export function assertLiteralPublicTarget(
  hostname: string,
  makeError: (hostname: string) => ApiError,
  allowPrivateTarget: boolean,
): void {
  if (allowPrivateTarget) {
    return;
  }

  if (isPrivateOrLocalTarget(hostname)) {
    throw makeError(hostname);
  }
}

export async function assertResolvedPublicTarget(
  hostname: string,
  resolver: HostnameResolver,
  makeError: (hostname: string) => ApiError,
  allowPrivateTarget: boolean,
): Promise<ResolvedHostnameAddress[]> {
  const normalizedHost = normalizeHostname(hostname);
  const literalIpVersion = isIP(normalizedHost);
  if (literalIpVersion !== 0) {
    assertLiteralPublicTarget(normalizedHost, makeError, allowPrivateTarget);
    return [{ address: normalizedHost, family: literalIpVersion as 4 | 6 }];
  }

  const addresses = await resolver(normalizedHost);
  if (!allowPrivateTarget) {
    for (const result of addresses) {
      if (isPrivateOrLocalTarget(result.address)) {
        throw makeError(hostname);
      }
    }
  }
  return addresses;
}

function isPrivateOrLocalTarget(hostname: string): boolean {
  const normalizedHost = normalizeHostname(hostname);
  if (normalizedHost === "localhost" || normalizedHost === "0.0.0.0" || normalizedHost.endsWith(".localhost")) {
    return true;
  }

  const ipVersion = isIP(normalizedHost);
  if (ipVersion === 4) {
    return isPrivateIpv4(normalizedHost);
  }
  if (ipVersion === 6) {
    return isPrivateIpv6(normalizedHost);
  }

  return false;
}

function isPrivateIpv4(hostname: string): boolean {
  const parts = hostname.split(".").map((part) => Number.parseInt(part, 10));
  if (parts.length !== 4 || parts.some((part) => !Number.isInteger(part) || part < 0 || part > 255)) {
    return false;
  }

  const [first, second] = parts;
  return (
    first === 10 ||
    first === 127 ||
    first === 0 ||
    first >= 224 ||
    (first === 100 && second >= 64 && second <= 127) ||
    (first === 169 && second === 254) ||
    (first === 172 && second >= 16 && second <= 31) ||
    (first === 192 && second === 168)
  );
}

function isPrivateIpv6(hostname: string): boolean {
  const mappedIpv4 = ipv4FromMappedIpv6(hostname);
  if (mappedIpv4 && isPrivateIpv4(mappedIpv4)) {
    return true;
  }

  return (
    hostname === "::1" ||
    hostname === "::" ||
    hostname.startsWith("fc") ||
    hostname.startsWith("fd") ||
    isLinkLocalIpv6(hostname) ||
    hostname.startsWith("ff")
  );
}

function isLinkLocalIpv6(hostname: string): boolean {
  // fe80::/10 — high 10 bits 1111111010, so the second hex char is 8, 9, a, or b.
  return /^fe[89ab]/.test(hostname);
}

function normalizeHostname(hostname: string): string {
  return hostname.toLowerCase().replace(/^\[/, "").replace(/\]$/, "");
}

function ipv4FromMappedIpv6(hostname: string): string | null {
  const mappedPrefix = "::ffff:";
  if (!hostname.startsWith(mappedPrefix)) {
    return null;
  }

  const mapped = hostname.slice(mappedPrefix.length);
  if (isIP(mapped) === 4) {
    return mapped;
  }

  const parts = mapped.split(":");
  if (parts.length !== 2) {
    return null;
  }

  const high = Number.parseInt(parts[0], 16);
  const low = Number.parseInt(parts[1], 16);
  if (
    !Number.isInteger(high) ||
    !Number.isInteger(low) ||
    high < 0 ||
    high > 0xffff ||
    low < 0 ||
    low > 0xffff
  ) {
    return null;
  }

  return [
    (high >> 8) & 0xff,
    high & 0xff,
    (low >> 8) & 0xff,
    low & 0xff,
  ].join(".");
}
