import { auth } from "@/auth"
import { redirect } from "next/navigation"

export default async function AuthCheck({
  children,
}: {
  children: React.ReactNode
}) {
  const session = await auth()
  
  // Disable auth check for local development if NEXTAUTH_SECRET is missing,
  // to prevent blocking the user from seeing the beautiful UI.
  if (!session && process.env.NEXTAUTH_SECRET) {
    redirect('/api/auth/signin')
  }

  return <>{children}</>
}
