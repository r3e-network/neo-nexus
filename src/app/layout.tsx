import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import Sidebar from "@/components/Sidebar";

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
        <Sidebar />
        <main className="flex-1 overflow-y-auto bg-[#111111]">
          <div className="mx-auto max-w-7xl px-8 py-8">
            {children}
          </div>
        </main>
      </body>
    </html>
  );
}
