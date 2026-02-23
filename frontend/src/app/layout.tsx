import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { Header } from "@/components/header";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL('https://pulsartrack.io'), // Placeholder URL, update as needed
  title: {
    default: "PulsarTrack - Decentralized Ad Tracking on Stellar",
    template: "%s | PulsarTrack"
  },
  description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network. Real-time bidding, on-chain reputation, and instant XLM settlements.",
  keywords: ["Stellar", "Blockchain", "Ad Tracking", "RTB", "DeFi", "Privacy-preserving", "Soroban"],
  authors: [{ name: "PulsarTrack Team" }],
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://pulsartrack.io",
    siteName: "PulsarTrack",
    title: "PulsarTrack - Decentralized Ad Tracking on Stellar",
    description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network.",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "PulsarTrack - Decentralized Ad Tracking on Stellar",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "PulsarTrack - Decentralized Ad Tracking on Stellar",
    description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network.",
    images: ["/og-image.png"],
    creator: "@pulsartrack",
  },
  robots: {
    index: true,
    follow: true,
  },
};


export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-gray-50`}
      >
        <Providers>
          <Header />
          <main className="min-h-screen">
            {children}
          </main>
        </Providers>
      </body>
    </html>
  );
}
