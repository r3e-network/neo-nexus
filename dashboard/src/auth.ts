import { PrismaAdapter } from '@auth/prisma-adapter';
import NextAuth from 'next-auth';
import type { Provider } from 'next-auth/providers';
import GithubProvider from 'next-auth/providers/github';
import GoogleProvider from 'next-auth/providers/google';
import CredentialsProvider from 'next-auth/providers/credentials';
import { isDatabaseConfigured } from '@/server/organization';
import { getConfiguredOperatorEmails, resolveUserRole } from '@/server/userRoles';
import { prisma } from '@/utils/prisma';

const providers: Provider[] = [];

if (process.env.GITHUB_ID && process.env.GITHUB_SECRET) {
  providers.push(
    GithubProvider({
      clientId: process.env.GITHUB_ID,
      clientSecret: process.env.GITHUB_SECRET,
    }),
  );
}

if (process.env.GOOGLE_ID && process.env.GOOGLE_SECRET) {
  providers.push(
    GoogleProvider({
      clientId: process.env.GOOGLE_ID,
      clientSecret: process.env.GOOGLE_SECRET,
    }),
  );
}

// Fallback for development/testing environments to ensure login works without OAuth configured
providers.push(
  CredentialsProvider({
    name: 'Email (Dev)',
    credentials: {
      email: { label: 'Email', type: 'email' },
    },
    async authorize(credentials) {
      if (!credentials?.email || typeof credentials.email !== 'string') return null;
      
      if (!isDatabaseConfigured()) {
        return {
          id: 'dev_user',
          email: credentials.email,
          name: credentials.email.split('@')[0],
          role: 'operator',
        };
      }

      let user = await prisma.user.findUnique({
        where: { email: credentials.email },
      });

      if (!user) {
        user = await prisma.user.create({
          data: {
            email: credentials.email,
            name: credentials.email.split('@')[0],
          }
        });
      }

      return user;
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
      const operatorEmails = getConfiguredOperatorEmails();
      const tokenEmail = typeof token.email === 'string' ? token.email : null;
      const tokenRole = typeof token.role === 'string' ? token.role : null;

      if (user?.id) {
        const userRole = typeof user.role === 'string' ? user.role : null;
        token.id = user.id;
        token.organizationId = user.organizationId ?? token.organizationId ?? null;
        token.role = resolveUserRole({
          role: userRole ?? tokenRole,
          email: user.email ?? tokenEmail,
          operatorEmails,
        });

        if (isDatabaseConfigured()) {
          const dbUser = await prisma.user.findUnique({
            where: { id: user.id },
            select: { organizationId: true, role: true, email: true },
          });

          if (dbUser) {
            token.organizationId = dbUser.organizationId ?? null;
            token.role = resolveUserRole({
              role: dbUser.role,
              email: dbUser.email ?? user.email ?? tokenEmail,
              operatorEmails,
            });
          }
        }
      }

      return token;
    },
    async session({ session, token }) {
      if (token && session.user && token.id) {
        session.user.id = token.id as string;
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
