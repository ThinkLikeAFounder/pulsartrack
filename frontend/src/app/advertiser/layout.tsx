import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Advertiser Dashboard",
    description: "Create and manage ad campaigns on PulsarTrack. Real-time bidding, budget controls, and on-chain analytics powered by Stellar and Soroban smart contracts.",
    openGraph: {
        title: "Advertiser Dashboard | PulsarTrack",
        description: "Create and manage ad campaigns on PulsarTrack. Real-time bidding, budget controls, and on-chain analytics powered by Stellar and Soroban smart contracts.",
    },
    twitter: {
        title: "Advertiser Dashboard | PulsarTrack",
        description: "Create and manage ad campaigns on PulsarTrack. Real-time bidding, budget controls, and on-chain analytics powered by Stellar and Soroban smart contracts.",
    },
};

export default function AdvertiserLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return children;
}
