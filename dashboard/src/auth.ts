import { PrismaAdapter } from '@auth/prisma-adapter';
import NextAuth from 'next-auth';
import type { Provider } from 'next-auth/providers';
import CredentialsProvider from 'next-auth/providers/credentials';
import { isDatabaseConfigured } from '@/server/organization';
import { getConfiguredOperatorWallets, resolveUserRole } from '@/server/userRoles';
import { prisma } from '@/utils/prisma';
import { wallet } from '@cityofzion/neon-js';

const providers: Provider[] = [];

providers.push(
  CredentialsProvider({
    name: 'Neo Wallet',
    credentials: {
      address: { label: 'Address', type: 'text' },
      publicKey: { label: 'Public Key', type: 'text' },
      message: { label: 'Message', type: 'text' },
      signature: { label: 'Signature', type: 'text' },
    },
    async authorize(credentials) {
      if (!credentials?.address || !credentials?.publicKey || !credentials?.message || !credentials?.signature) {
        return null;
      }
      
      const { address, publicKey, message, signature } = credentials as Record<string, string>;
      
      try {
        const isValid = wallet.verify(message, signature, publicKey);
        if (!isValid) return null;
        
        const expectedAddress = new wallet.Account(publicKey).address;
        if (expectedAddress !== address) return null;
        
        if (!isDatabaseConfigured()) {
          return {
            id: `dev_${address}`,
            name: address.slice(0, 8),
            walletAddress: address,
            role: 'operator',
          };
        }

        let userRecord = await prisma.user.findUnique({
          where: { walletAddress: address },
        });

        if (!userRecord) {
          userRecord = await prisma.user.create({
            data: {
              walletAddress: address,
              name: `Neo User ${address.slice(0, 4)}`,
            }
          });
        }

        return userRecord;
      } catch (e) {
        console.error('Signature verification failed', e);
        return null;
      }
    },
  })
);

export const { handlers, auth, signIn, signOut } = NextAuth({
  adapter: isDatabaseConfigured() ? PrismaAdapter(prisma) : undefined,
  providers,
  session: {
    strategy: 'jwt',
  },
  callbacks: {
    async jwt({ token, user }) {
      const operatorWallets = getConfiguredOperatorWallets();
      const tokenWallet = typeof token.walletAddress === 'string' ? token.walletAddress : null;
      const tokenRole = typeof token.role === 'string' ? token.role : null;

      if (user?.id) {
        const userRole = typeof user.role === 'string' ? user.role : null;
        token.id = user.id;
        token.walletAddress = user.walletAddress ?? tokenWallet ?? null;
        token.organizationId = user.organizationId ?? token.organizationId ?? null;
        token.role = resolveUserRole({
          role: userRole ?? tokenRole,
          walletAddress: user.walletAddress ?? tokenWallet,
          operatorWallets,
        });

        if (isDatabaseConfigured()) {
          const dbUser = await prisma.user.findUnique({
            where: { id: user.id },
            select: { id: true, name: true, walletAddress: true, organizationId: true, role: true },
          });

          if (dbUser) {
            let userOrgId = dbUser.organizationId;

            // PROFESSIONAL REFACTOR: Auto-provision a Personal Workspace on first login
            if (!userOrgId) {
              const defaultName = dbUser.name || dbUser.walletAddress?.slice(0, 6) || 'Personal';
              const newOrg = await prisma.organization.create({
                data: {
                  name: `${defaultName} Workspace`,
                  billingPlan: 'developer',
                  users: {
                    connect: { id: dbUser.id }
                  }
                }
              });
              
              // Also auto-generate an API key for them so they can hit the ground running
              await prisma.apiKey.create({
                data: {
                  name: 'Default API Key',
                  keyHash: `nk_live_${Buffer.from(newOrg.id + Date.now().toString()).toString('base64url').slice(0, 32)}`,
                  organizationId: newOrg.id,
                }
              });

              userOrgId = newOrg.id;
            }

            token.walletAddress = dbUser.walletAddress;
            token.organizationId = userOrgId;
            token.role = resolveUserRole({
              role: dbUser.role,
              walletAddress: dbUser.walletAddress ?? user.walletAddress ?? tokenWallet,
              operatorWallets,
            });
          }
        }
      }

      return token;
    },
    async session({ session, token }) {
      if (token && session.user && token.id) {
        session.user.id = token.id as string;
        session.user.walletAddress = typeof token.walletAddress === 'string' ? token.walletAddress : null;
        session.user.organizationId = typeof token.organizationId === 'string' ? token.organizationId : null;
        session.user.role = typeof token.role === 'string' ? token.role : 'member';
      }

      return session;
    },
  },
  pages: {
    signIn: '/login', // Will redirect to the website's login in a real setup, or handle locally
  },
});
