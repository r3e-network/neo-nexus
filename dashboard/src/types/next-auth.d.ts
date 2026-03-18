import type { DefaultSession } from 'next-auth';

declare module 'next-auth' {
  interface Session {
    user: DefaultSession['user'] & {
      id: string;
      organizationId: string | null;
      role: string;
      walletAddress: string | null;
    };
  }

  interface User {
    organizationId?: string | null;
    role?: string | null;
    walletAddress?: string | null;
  }
}

declare module 'next-auth/jwt' {
  interface JWT {
    id?: string;
    organizationId?: string | null;
    role?: string | null;
    walletAddress?: string | null;
  }
}