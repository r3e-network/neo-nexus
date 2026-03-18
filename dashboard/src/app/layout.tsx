import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "NeoNexus | Industrial-grade Neo Infrastructure",
  description: "The ultimate Web3 cloud for Neo N3 and Neo X. Deploy dedicated nodes, manage plugins, and scale your dApps instantly with zero-touch infrastructure.",
  keywords: ["Neo", "N3", "Neo X", "Node as a Service", "RPC", "Blockchain Infrastructure", "dApp"],
  openGraph: {
    title: "NeoNexus | Industrial-grade Neo Infrastructure",
    description: "The ultimate Web3 cloud for Neo N3 and Neo X. Deploy dedicated nodes, manage plugins, and scale your dApps instantly with zero-touch infrastructure.",
    type: "website",
    siteName: "NeoNexus",
  },
  twitter: {
    card: "summary_large_image",
    title: "NeoNexus | Industrial-grade Neo Infrastructure",
    description: "The ultimate Web3 cloud for Neo N3 and Neo X. Deploy dedicated nodes, manage plugins, and scale your dApps instantly.",
  }
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark scroll-smooth">
      <body className={`${inter.className} bg-[var(--color-dark-bg)] text-white antialiased`}>
        {children}
      </body>
    </html>
  );
}
