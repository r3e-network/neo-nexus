import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "NeoNexus | Industrial-grade Neo Infrastructure",
  description: "Neo ecosystem's premier Web3 infrastructure provider.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark scroll-smooth">
      <body className={`${inter.className} bg-[#0F111A] text-white antialiased`}>
        {children}
      </body>
    </html>
  );
}
