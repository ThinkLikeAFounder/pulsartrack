import type { Metadata } from "next";
import Link from "next/link";
import { TrendingUp, Users, Shield, Zap, Radio, Lock, BarChart3, Globe } from "lucide-react";

export const metadata: Metadata = {
  title: "PulsarTrack - Decentralized Ad Tracking on Stellar",
  description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network. Real-time bidding, on-chain reputation, and instant XLM settlements.",
  openGraph: {
    title: "PulsarTrack - Decentralized Ad Tracking on Stellar",
    description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network. Real-time bidding, on-chain reputation, and instant XLM settlements.",
  },
  twitter: {
    title: "PulsarTrack - Decentralized Ad Tracking on Stellar",
    description: "Privacy-preserving, blockchain-powered advertising platform on the Stellar network. Real-time bidding, on-chain reputation, and instant XLM settlements.",
  },
};

export default function Home() {
  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="px-4 pt-20 pb-16 sm:px-6 lg:px-8 bg-gradient-to-br from-indigo-50 via-white to-cyan-50">
        <div className="mx-auto max-w-7xl text-center">
          <div className="inline-flex items-center gap-2 px-4 py-1.5 bg-indigo-100 rounded-full text-indigo-700 text-sm font-medium mb-6">
            <Radio className="w-4 h-4" />
            Powered by Stellar &amp; Soroban Smart Contracts
          </div>
          <h1 className="text-5xl font-bold tracking-tight text-gray-900 sm:text-6xl">
            Ad Tracking That Pulses
            <br />
            <span className="text-indigo-600">With the Stellar Network</span>
          </h1>
          <p className="mt-6 text-lg leading-8 text-gray-600 max-w-2xl mx-auto">
            PulsarTrack connects advertisers and publishers through Soroban smart contracts.
            Zero-knowledge privacy, real-time bidding, and instant XLM settlements on Stellar.
          </p>
          <div className="mt-10 flex items-center justify-center gap-6 flex-wrap">
            <Link
              href="/advertiser"
              className="rounded-lg bg-indigo-600 px-6 py-3 text-base font-semibold text-white hover:bg-indigo-700 transition-colors"
            >
              Launch Campaign
            </Link>
            <Link
              href="/publisher"
              className="rounded-lg border-2 border-indigo-600 px-6 py-3 text-base font-semibold text-indigo-600 hover:bg-indigo-50 transition-colors"
            >
              Become a Publisher
            </Link>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="px-4 py-16 sm:px-6 lg:px-8 bg-white">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-12">
            Why PulsarTrack?
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            {[
              {
                icon: Shield,
                title: "Fraud Prevention",
                desc: "On-chain view verification with rate limiting prevents click fraud and fake impressions.",
              },
              {
                icon: TrendingUp,
                title: "Real-time Analytics",
                desc: "Transparent, on-chain campaign metrics with live WebSocket streaming.",
              },
              {
                icon: Zap,
                title: "XLM Settlements",
                desc: "Instant Stellar Lumens payments via Soroban token contracts. No delays.",
              },
              {
                icon: Users,
                title: "Publisher Network",
                desc: "Reputation-scored publisher ecosystem with tiered KYC verification.",
              },
              {
                icon: Lock,
                title: "Privacy Layer",
                desc: "Zero-knowledge proof consent management. GDPR-compliant data handling.",
              },
              {
                icon: BarChart3,
                title: "RTB Auctions",
                desc: "Real-time bidding auction engine with floor pricing and reserve prices.",
              },
              {
                icon: Globe,
                title: "Targeting Engine",
                desc: "Privacy-preserving audience targeting: geo, interests, device, age.",
              },
              {
                icon: Radio,
                title: "PULSAR Governance",
                desc: "On-chain DAO voting with PULSAR tokens for platform decisions.",
              },
            ].map(({ icon: Icon, title, desc }) => (
              <div key={title} className="text-center p-4">
                <div className="mx-auto w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center mb-4">
                  <Icon className="w-6 h-6 text-indigo-600" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900 mb-2">{title}</h3>
                <p className="text-gray-600 text-sm">{desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Contracts Section */}
      <section className="px-4 py-16 sm:px-6 lg:px-8 bg-gray-50">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-center text-gray-900 mb-4">
            39 Soroban Smart Contracts
          </h2>
          <p className="text-center text-gray-600 mb-10">
            Every feature backed by auditable, on-chain Rust/Wasm contracts on Stellar.
          </p>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            {[
              "Ad Registry", "Campaign Orchestrator", "Escrow Vault", "Fraud Prevention",
              "Payment Processor", "Governance Token", "Publisher Verification", "Auction Engine",
              "Privacy Layer", "Targeting Engine", "Analytics Aggregator", "Publisher Reputation",
              "Subscription Manager", "Identity Registry", "KYC Registry", "Dispute Resolution",
              "Revenue Settlement", "Rewards Distributor", "Governance DAO", "Timelock Executor",
              "Oracle Integration", "Liquidity Pool", "Milestone Tracker", "Multisig Treasury",
              "Payout Automation", "Recurring Payment", "Refund Processor", "Token Bridge",
            ].map((name) => (
              <div
                key={name}
                className="px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-700 text-center"
              >
                {name}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="px-4 py-16 sm:px-6 lg:px-8 bg-indigo-600">
        <div className="mx-auto max-w-7xl">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 text-center">
            {[
              { value: "39", label: "Soroban Contracts" },
              { value: "XLM", label: "Native Settlement" },
              { value: "ZKP", label: "Privacy Layer" },
              { value: "DAO", label: "On-chain Governance" },
            ].map(({ value, label }) => (
              <div key={label}>
                <div className="text-4xl font-bold text-white">{value}</div>
                <div className="mt-2 text-indigo-200">{label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
