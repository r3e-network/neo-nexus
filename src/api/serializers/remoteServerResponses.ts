import type { RemoteServerProfile, RemoteServerSummary } from "../../types";
import type { ResponseRole } from "./nodeResponses";

export type ViewerRemoteServerProfile = Omit<RemoteServerProfile, "baseUrl">;

export type ViewerRemoteServerSummary = Omit<RemoteServerSummary, "profile" | "error"> & {
  profile: ViewerRemoteServerProfile;
};

export function remoteServerProfileResponseForRole(
  role: ResponseRole,
  profile: RemoteServerProfile,
): RemoteServerProfile | ViewerRemoteServerProfile {
  if (role !== "viewer") {
    return profile;
  }
  const { baseUrl: _baseUrl, ...safeProfile } = profile;
  return safeProfile;
}

export function remoteServerSummaryResponseForRole(
  role: ResponseRole,
  summary: RemoteServerSummary,
): RemoteServerSummary | ViewerRemoteServerSummary {
  if (role !== "viewer") {
    return summary;
  }
  const { profile, error: _error, ...safeSummary } = summary;
  return {
    ...safeSummary,
    profile: remoteServerProfileResponseForRole(role, profile),
  };
}

export function remoteServerSummariesResponseForRole(
  role: ResponseRole,
  summaries: RemoteServerSummary[],
): RemoteServerSummary[] | ViewerRemoteServerSummary[] {
  return summaries.map((summary) => remoteServerSummaryResponseForRole(role, summary));
}
