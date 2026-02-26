import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Governance",
    description: "Participate in PulsarTrack DAO governance. Vote on proposals with PULSAR tokens and shape the future of decentralized ad tracking on Stellar.",
    openGraph: {
        title: "Governance | PulsarTrack",
        description: "Participate in PulsarTrack DAO governance. Vote on proposals with PULSAR tokens and shape the future of decentralized ad tracking on Stellar.",
    },
    twitter: {
        title: "Governance | PulsarTrack",
        description: "Participate in PulsarTrack DAO governance. Vote on proposals with PULSAR tokens and shape the future of decentralized ad tracking on Stellar.",
    },
};

export default function GovernanceLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return children;
}
