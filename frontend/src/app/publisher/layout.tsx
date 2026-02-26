import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Publisher Dashboard",
    description: "Monetize your ad inventory on PulsarTrack. Earn XLM through real-time bidding auctions, track your reputation score, and manage subscriptions on Stellar.",
    openGraph: {
        title: "Publisher Dashboard | PulsarTrack",
        description: "Monetize your ad inventory on PulsarTrack. Earn XLM through real-time bidding auctions, track your reputation score, and manage subscriptions on Stellar.",
    },
    twitter: {
        title: "Publisher Dashboard | PulsarTrack",
        description: "Monetize your ad inventory on PulsarTrack. Earn XLM through real-time bidding auctions, track your reputation score, and manage subscriptions on Stellar.",
    },
};

export default function PublisherLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return children;
}
