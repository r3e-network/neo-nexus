import { PrismaAdapter } from '@auth/prisma-adapter';
import NextAuth from 'next-auth';
import type { Provider } from 'next-auth/providers';
import GithubProvider from 'next-auth/providers/github';
import GoogleProvider from 'next-auth/providers/google';
import { isDatabaseConfigured } from '@/server/organization';
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

export const { handlers, auth, signIn, signOut } = NextAuth({
  adapter: isDatabaseConfigured() ? PrismaAdapter(prisma) : undefined,
  providers,
  session: {
    strategy: 'jwt',
  },
  callbacks: {
    async jwt({ token, user }) {
      if (user?.id) {
        token.id = user.id;
        token.organizationId = user.organizationId ?? token.organizationId ?? null;

        if (isDatabaseConfigured()) {
          const dbUser = await prisma.user.findUnique({
            where: { id: user.id },
            select: { organizationId: true },
          });

          if (dbUser?.organizationId) {
            token.organizationId = dbUser.organizationId;
          }
        }
      }

      return token;
    },
    async session({ session, token }) {
      if (token && session.user && token.id) {
        session.user.id = token.id as string;
        session.user.organizationId = typeof token.organizationId === 'string' ? token.organizationId : null;
      }

      return session;
    },
  },
  pages: {
    signIn: '/login', // Will redirect to the website's login in a real setup, or handle locally
  },
});
