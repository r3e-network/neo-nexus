import { isIP } from "node:net";

interface AuthenticationUser {
  id: string;
  role: "admin" | "viewer";
}

export interface AuthenticationUserManager {
  getAllUsers(): AuthenticationUser[];
  isUsingDefaultPassword(userId: string): Promise<boolean>;
}

export function isLoopbackAddress(address: string | undefined): boolean {
  if (!address) return false;

  const normalized = address.startsWith("::ffff:") ? address.slice(7) : address;
  if (normalized === "::1") return true;
  if (isIP(normalized) !== 4) return false;

  return normalized.startsWith("127.");
}

export async function assertProductionAuthenticationSafe(
  userManager: AuthenticationUserManager,
  environment = process.env.NODE_ENV,
): Promise<void> {
  if (environment !== "production") return;

  const admins = userManager.getAllUsers().filter((user) => user.role === "admin");
  for (const admin of admins) {
    if (await userManager.isUsingDefaultPassword(admin.id)) {
      throw new Error(
        "Refusing to start in production while a legacy default admin password is active. " +
          "Change the password from a local recovery session before restarting NeoNexus.",
      );
    }
  }
}
