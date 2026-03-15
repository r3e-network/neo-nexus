import NextAuth from "next-auth"
import GithubProvider from "next-auth/providers/github"
import GoogleProvider from "next-auth/providers/google"

if (!process.env.GITHUB_ID || !process.env.GITHUB_SECRET) {
  console.warn("GITHUB_ID or GITHUB_SECRET is missing. NextAuth GitHub provider may fail.");
}

if (!process.env.GOOGLE_ID || !process.env.GOOGLE_SECRET) {
  console.warn("GOOGLE_ID or GOOGLE_SECRET is missing. NextAuth Google provider may fail.");
}

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_ID!,
      clientSecret: process.env.GITHUB_SECRET!,
    }),
    GoogleProvider({
      clientId: process.env.GOOGLE_ID!,
      clientSecret: process.env.GOOGLE_SECRET!,
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
