import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import Sidebar from "@/components/Sidebar";
import AuthCheck from "@/components/AuthCheck";
import { Toaster } from "react-hot-toast";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "NeoNexus | Industrial-grade N3 Node Cloud",
  description: "Neo ecosystem's Chainstack + exclusive Web3 marketplace.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} flex h-screen overflow-hidden bg-[#111111] text-white`}>
        <Toaster position="top-right" toastOptions={{ style: { background: '#333', color: '#fff' } }} />
        <Sidebar />
        <main className="flex-1 overflow-y-auto bg-[#111111]">
          <div className="mx-auto max-w-7xl px-8 py-8">
            <AuthCheck>
              {children}
            </AuthCheck>
          </div>
        </main>
      </body>
    </html>
  );
}
