import NextAuth from "next-auth"
import GithubProvider from "next-auth/providers/github"
import GoogleProvider from "next-auth/providers/google"

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_ID || 'mock_id',
      clientSecret: process.env.GITHUB_SECRET || 'mock_secret',
    }),
    GoogleProvider({
      clientId: process.env.GOOGLE_ID || 'mock_id',
      clientSecret: process.env.GOOGLE_SECRET || 'mock_secret',
    })
  ],
  session: {
    strategy: "jwt",
  },
  callbacks: {
    async jwt({ token, user }) {
      if (user) {
        token.id = user.id;
      }
      return token;
    },
    async session({ session, token }) {
      if (token && session.user) {
        session.user.id = token.id as string;
      }
      return session;
    }
  },
  pages: {
    signIn: '/login',
  }
})
