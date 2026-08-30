"use client";

import Link from "next/link";
import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import {
  Shield,
  Activity,
  Zap,
  Users,
  Lock,
  Radio,
  Globe,
  Vote,
  ArrowRight,
  ArrowUpRight,
  Sparkles,
  Satellite,
} from "lucide-react";

/* ============================================================================
   Deterministic generators (seeded) — identical on server & client, so there
   is never a hydration mismatch. No Math.random / Date.now at render time.
   ========================================================================== */

function mulberry32(seed: number) {
  return function () {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/* --- The CP-1919 pulsar waveform (a.k.a. the "Unknown Pleasures" ridges) --- */
const RIDGE_W = 1000;
const RIDGE_H = 560;
const ROWS = 30;
const SAMPLES = 120;

type Ridge = { stroke: string; fill: string; opacity: number };

function buildRidges(): Ridge[] {
  const rnd = mulberry32(0x9e3779b1);
  const ridges: Ridge[] = [];
  const top = 70;
  const rowGap = (RIDGE_H - 150) / ROWS;
  const center = ROWS * 0.46;
  const ampSigma = ROWS * 0.3;

  for (let r = 0; r < ROWS; r++) {
    const baseline = top + r * rowGap;
    const ampScale = Math.exp(-((r - center) ** 2) / (2 * ampSigma ** 2));
    const peakX = 0.5 + (rnd() - 0.5) * 0.05;
    const peakW = 0.045 + rnd() * 0.05;
    const peak2X = 0.5 + (rnd() - 0.5) * 0.22;
    const peak2W = 0.03 + rnd() * 0.045;
    const maxAmp = (95 + rnd() * 55) * ampScale;

    const pts: Array<[number, number]> = [];
    for (let s = 0; s <= SAMPLES; s++) {
      const xN = s / SAMPLES;
      const x = xN * RIDGE_W;
      const g1 = Math.exp(-((xN - peakX) ** 2) / (2 * peakW ** 2));
      const g2 = 0.55 * Math.exp(-((xN - peak2X) ** 2) / (2 * peak2W ** 2));
      const edge = Math.exp(-((xN - 0.5) ** 2) / (2 * 0.2 ** 2));
      const noise = (rnd() - 0.5) * 16 * edge * ampScale;
      const h = maxAmp * (g1 + g2) + noise;
      const y = baseline - Math.max(0, h);
      pts.push([x, y]);
    }

    let d = `M ${pts[0][0].toFixed(1)} ${pts[0][1].toFixed(1)}`;
    for (let i = 1; i < pts.length; i++) {
      d += ` L ${pts[i][0].toFixed(1)} ${pts[i][1].toFixed(1)}`;
    }
    // Fill down to the canvas floor so nearer (lower) ridges occlude farther
    // ones — drawn back-to-front, top row first.
    const fill = `${d} L ${RIDGE_W} ${RIDGE_H} L 0 ${RIDGE_H} Z`;

    ridges.push({
      stroke: d,
      fill,
      opacity: 0.22 + ampScale * 0.65,
    });
  }
  return ridges;
}

const RIDGES = buildRidges();

/* --- Starfield ----------------------------------------------------------- */
type Star = { x: number; y: number; size: number; dur: number; delay: number };

function buildStars(count: number): Star[] {
  const rnd = mulberry32(0x1f123bb5);
  const stars: Star[] = [];
  for (let i = 0; i < count; i++) {
    stars.push({
      x: rnd() * 100,
      y: rnd() * 100,
      size: rnd() < 0.85 ? 1 + rnd() * 1.4 : 2 + rnd() * 1.8,
      dur: 3 + rnd() * 6,
      delay: rnd() * 6,
    });
  }
  return stars;
}

const STARS = buildStars(90);

/* ============================================================================
   Small client helpers
   ========================================================================== */

function Reveal({
  children,
  className = "",
  delay = 0,
}: {
  children: ReactNode;
  className?: string;
  delay?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setInView(true);
          io.disconnect();
        }
      },
      { threshold: 0.12, rootMargin: "0px 0px -7% 0px" }
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div
      ref={ref}
      className={`pt-reveal ${inView ? "pt-in" : ""} ${className}`}
      style={{ "--d": `${delay}ms` } as CSSProperties}
    >
      {children}
    </div>
  );
}

function Stat({
  value,
  prefix = "",
  suffix = "",
  label,
  staticValue,
}: {
  value?: number;
  prefix?: string;
  suffix?: string;
  label: string;
  staticValue?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [n, setN] = useState(0);

  useEffect(() => {
    if (staticValue !== undefined || value === undefined) return;
    const el = ref.current;
    if (!el) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const io = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        io.disconnect();
        if (reduce) {
          setN(value);
          return;
        }
        const dur = 1500;
        const t0 = performance.now();
        const tick = (t: number) => {
          const p = Math.min(1, (t - t0) / dur);
          const eased = 1 - Math.pow(1 - p, 3);
          setN(value * eased);
          if (p < 1) requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      },
      { threshold: 0.4 }
    );
    io.observe(el);
    return () => io.disconnect();
  }, [value, staticValue]);

  return (
    <div ref={ref}>
      <div className="pt-display text-4xl sm:text-5xl text-[var(--ink)] leading-none">
        {staticValue !== undefined ? (
          <span className="pt-grad">{staticValue}</span>
        ) : (
          <span>
            {prefix}
            {Math.round(n)}
            {suffix}
          </span>
        )}
      </div>
      <div className="pt-mono mt-3 text-[0.62rem] text-[var(--faint)]">{label}</div>
    </div>
  );
}

/** Cursor-tracking spotlight for the glass cards. */
function spotlight(e: React.MouseEvent<HTMLElement>) {
  const el = e.currentTarget;
  const rect = el.getBoundingClientRect();
  el.style.setProperty("--mx", `${e.clientX - rect.left}px`);
  el.style.setProperty("--my", `${e.clientY - rect.top}px`);
}

/* ============================================================================
   Content
   ========================================================================== */

const CAPABILITIES = [
  {
    icon: Shield,
    title: "Fraud Prevention",
    body:
      "On-chain view verification with per-viewer rate limiting kills click fraud and fake impressions before they ever touch your budget.",
  },
  {
    icon: Activity,
    title: "Real-Time Analytics",
    body:
      "Transparent campaign metrics streamed live over WebSocket — every impression, click and conversion provable on Stellar.",
  },
  {
    icon: Zap,
    title: "Instant XLM Settlement",
    body:
      "Publishers are paid the moment a view is verified. Native Soroban token transfers, ~5-second finality, zero invoicing.",
  },
  {
    icon: Lock,
    title: "Zero-Knowledge Privacy",
    body:
      "ZKP consent management and GDPR-grade data handling. Reach the right audience without ever harvesting an identity.",
  },
  {
    icon: Radio,
    title: "RTB Auction Engine",
    body:
      "A real-time bidding marketplace with floor and reserve pricing, cleared trustlessly on a per-impression basis.",
  },
  {
    icon: Globe,
    title: "Targeting Engine",
    body:
      "Privacy-preserving targeting across geography, interest, device and age — scored and matched fully on-chain.",
  },
  {
    icon: Users,
    title: "Publisher Network",
    body:
      "A reputation-scored publisher ecosystem with tiered KYC verification and an immutable on-chain earnings history.",
  },
  {
    icon: Vote,
    title: "PULSAR Governance",
    body:
      "Steer the protocol with PULSAR. Token-weighted DAO voting with timelocked, auditable on-chain execution.",
  },
];

const CATALOG = [
  { group: "Core Ad Engine", code: "CAE", items: ["Ad Registry", "Campaign Orchestrator", "Escrow Vault", "Fraud Prevention", "Payment Processor"] },
  { group: "Governance", code: "GOV", items: ["Governance Token", "Governance DAO", "Governance Core", "Timelock Executor"] },
  { group: "Publishers", code: "PUB", items: ["Publisher Verification", "Publisher Network", "Publisher Reputation"] },
  { group: "Analytics", code: "ANL", items: ["Analytics Aggregator", "Campaign Analytics", "Campaign Lifecycle"] },
  { group: "Privacy & Targeting", code: "PRV", items: ["Privacy Layer", "Targeting Engine", "Audience Segments"] },
  { group: "Identity & Access", code: "IDN", items: ["Identity Registry", "KYC Registry", "Access Control"] },
  { group: "Marketplace", code: "MKT", items: ["Auction Engine", "Creative Marketplace"] },
  { group: "Subscriptions", code: "SUB", items: ["Subscription Manager", "Subscription Benefits"] },
  { group: "Finance", code: "FIN", items: ["Liquidity Pool", "Milestone Tracker", "Multisig Treasury", "Oracle Integration", "Payout Automation", "Performance Oracle", "Recurring Payment", "Refund Processor", "Revenue Settlement", "Rewards Distributor", "Treasury Manager", "Vesting Schedule"] },
  { group: "Bridge", code: "BRG", items: ["Token Bridge", "Wrapped Token"] },
  { group: "Utility", code: "UTL", items: ["Dispute Resolution", "Budget Optimizer", "Anomaly Detector", "Whitelist Registry"] },
];

const CONTRACT_TOTAL = CATALOG.reduce((s, c) => s + c.items.length, 0);

const STEPS = [
  {
    k: "01",
    title: "Broadcast",
    body:
      "An advertiser funds a campaign. The budget is locked inside the Escrow Vault contract — held trustlessly, released only against verified delivery.",
  },
  {
    k: "02",
    title: "Capture",
    body:
      "Publishers serve the creative. Every view is checked by the Fraud Prevention contract — rate-limited, deduplicated and scored before it counts.",
  },
  {
    k: "03",
    title: "Settle",
    body:
      "Verified views trigger instant XLM payouts from escrow to the publisher, while analytics update live. No clearing house, no waiting.",
  },
];

/* ============================================================================
   Page
   ========================================================================== */

export default function Home() {
  return (
    <div className="pt-cosmos">
      {/* Starfield */}
      <div className="pt-stars" aria-hidden>
        {STARS.map((s, i) => (
          <span
            key={i}
            className="pt-star"
            style={
              {
                left: `${s.x}%`,
                top: `${s.y}%`,
                width: `${s.size}px`,
                height: `${s.size}px`,
                "--t": `${s.dur}s`,
                "--dl": `${s.delay}s`,
              } as CSSProperties
            }
          />
        ))}
      </div>

      {/* ================= HERO ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 pt-20 pb-24 sm:pt-28">
        {/* radio-telescope sweep behind the hero */}
        <div
          className="pt-sweep"
          aria-hidden
          style={{ top: "-160px", right: "-180px", width: "640px", height: "640px" }}
        />
        <div
          className="pt-sweep-ring"
          aria-hidden
          style={{ top: "-20px", right: "-40px", width: "360px", height: "360px" }}
        />

        <div className="mx-auto max-w-6xl text-center relative">
          <div className="pt-load flex justify-center" style={{ "--d": "0ms" } as CSSProperties}>
            <span className="pt-pill">
              <span className="pt-live-dot" />
              <span className="pt-mono text-[0.62rem] text-[var(--muted)]">
                Live on Stellar · Soroban smart contracts
              </span>
            </span>
          </div>

          <h1
            className="pt-load pt-display mt-8 text-[3.1rem] leading-[1.02] sm:text-7xl lg:text-8xl text-[var(--ink)]"
            style={{ "--d": "90ms" } as CSSProperties}
          >
            The <span className="pt-grad italic">pulse</span> of
            <br className="hidden sm:block" /> decentralized advertising.
          </h1>

          <p
            className="pt-load mx-auto mt-7 max-w-2xl text-base sm:text-lg leading-relaxed text-[var(--muted)]"
            style={{ "--d": "180ms" } as CSSProperties}
          >
            PulsarTrack links advertisers and publishers through {CONTRACT_TOTAL} Soroban
            smart contracts on Stellar — zero-knowledge privacy, real-time bidding and
            instant XLM settlement. No middlemen. No blind spots.
          </p>

          <div
            className="pt-load mt-10 flex flex-wrap items-center justify-center gap-4"
            style={{ "--d": "270ms" } as CSSProperties}
          >
            <Link href="/advertiser" className="pt-btn">
              Launch a campaign <ArrowRight className="w-4 h-4" />
            </Link>
            <Link href="/publisher" className="pt-btn-ghost">
              Become a publisher
            </Link>
          </div>

          {/* telemetry readout */}
          <div
            className="pt-load mt-12 flex flex-wrap items-center justify-center gap-x-6 gap-y-2 pt-mono text-[0.6rem] text-[var(--faint)]"
            style={{ "--d": "360ms" } as CSSProperties}
          >
            <span className="text-[var(--signal-dim)]">signal ◦ 1.337 GHz</span>
            <span className="opacity-30">/</span>
            <span>latency ◦ 5s / ledger</span>
            <span className="opacity-30">/</span>
            <span>
              contracts ◦ {CONTRACT_TOTAL}
              <span className="text-[var(--signal-dim)]">/{CONTRACT_TOTAL} online</span>
            </span>
            <span className="opacity-30">/</span>
            <span>net ◦ stellar</span>
          </div>
        </div>

        {/* The pulsar waveform */}
        <div
          className="pt-load mx-auto mt-16 max-w-5xl relative"
          style={{ "--d": "440ms" } as CSSProperties}
        >
          <div className="absolute -left-2 top-1/2 -translate-y-1/2 hidden sm:flex flex-col items-center gap-3">
            <span className="pt-beacon" />
          </div>
          <svg
            viewBox={`0 0 ${RIDGE_W} ${RIDGE_H}`}
            className="pt-ridges"
            role="img"
            aria-label="A stylised CP-1919 pulsar signal waveform"
          >
            {RIDGES.map((r, i) => (
              <g key={i}>
                <path d={r.fill} fill="var(--void)" />
                <path
                  d={r.stroke}
                  fill="none"
                  stroke="var(--signal)"
                  strokeWidth={1.4}
                  strokeOpacity={r.opacity}
                  strokeLinejoin="round"
                  strokeLinecap="round"
                />
              </g>
            ))}
          </svg>
          <div className="pt-mono mt-2 flex justify-between text-[0.55rem] text-[var(--faint)] px-1">
            <span>PSR B1919+21</span>
            <span>periodic radio emission · 1.3373 s</span>
          </div>
        </div>
      </section>

      {/* ================= STATS ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 py-10">
        <Reveal>
          <div className="mx-auto max-w-5xl rounded-2xl border border-[var(--line)] bg-[rgba(143,162,222,0.04)] backdrop-blur-md px-6 sm:px-10 py-10">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
              <Stat value={CONTRACT_TOTAL} suffix="+" label="Soroban contracts" />
              <Stat value={100} suffix="%" label="On-chain & auditable" />
              <Stat value={5} prefix="~" suffix="s" label="Settlement finality" />
              <Stat staticValue="ZKP" label="Privacy layer" />
            </div>
          </div>
        </Reveal>
      </section>

      {/* ================= CAPABILITIES ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 py-24">
        <div className="mx-auto max-w-6xl">
          <Reveal className="max-w-2xl">
            <span className="pt-mono text-[0.62rem] text-[var(--signal-dim)]">
              ◦ Capabilities
            </span>
            <h2 className="pt-display mt-4 text-4xl sm:text-5xl text-[var(--ink)]">
              An entire ad stack,
              <span className="italic pt-grad"> rebuilt on-chain.</span>
            </h2>
            <p className="mt-5 text-[var(--muted)] leading-relaxed">
              Every feature is a verifiable Soroban contract — not a black box. Composable,
              auditable, and impossible to quietly tamper with.
            </p>
          </Reveal>

          <div className="mt-14 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
            {CAPABILITIES.map(({ icon: Icon, title, body }, i) => (
              <Reveal key={title} delay={(i % 4) * 80}>
                <article className="pt-card h-full p-6" onMouseMove={spotlight}>
                  <div className="pt-icon">
                    <Icon className="w-[22px] h-[22px]" strokeWidth={1.6} />
                  </div>
                  <h3 className="mt-5 text-lg font-semibold text-[var(--ink)]">{title}</h3>
                  <p className="mt-2.5 text-sm leading-relaxed text-[var(--muted)]">{body}</p>
                </article>
              </Reveal>
            ))}
          </div>
        </div>
      </section>

      {/* ================= HOW IT WORKS ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 py-12">
        <div className="mx-auto max-w-6xl">
          <Reveal className="text-center max-w-2xl mx-auto">
            <span className="pt-mono text-[0.62rem] text-[var(--signal-dim)]">
              ◦ The signal path
            </span>
            <h2 className="pt-display mt-4 text-4xl sm:text-5xl text-[var(--ink)]">
              From broadcast to <span className="italic pt-grad">settlement.</span>
            </h2>
          </Reveal>

          <div className="mt-16 relative">
            {/* connecting beam */}
            <div className="hidden md:block absolute top-[58px] left-[12%] right-[12%] pt-beamline" />
            <div className="grid grid-cols-1 md:grid-cols-3 gap-10">
              {STEPS.map((s, i) => (
                <Reveal key={s.k} delay={i * 120}>
                  <div className="text-center md:px-4">
                    <div className="mx-auto relative w-[116px] h-[116px] grid place-items-center">
                      <div className="absolute inset-0 rounded-full border border-[var(--line-bright)]" />
                      <div className="absolute inset-3 rounded-full border border-dashed border-[rgba(94,234,255,0.25)] animate-[pt-rotate_22s_linear_infinite]" />
                      <span className="pt-display text-3xl pt-grad relative z-10">{s.k}</span>
                    </div>
                    <h3 className="mt-6 text-xl font-semibold text-[var(--ink)]">{s.title}</h3>
                    <p className="mt-3 text-sm leading-relaxed text-[var(--muted)]">{s.body}</p>
                  </div>
                </Reveal>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ================= CONTRACT CATALOG ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 py-24">
        <div className="mx-auto max-w-6xl">
          <Reveal className="flex flex-wrap items-end justify-between gap-6">
            <div className="max-w-2xl">
              <span className="pt-mono text-[0.62rem] text-[var(--signal-dim)]">
                ◦ The constellation
              </span>
              <h2 className="pt-display mt-4 text-4xl sm:text-5xl text-[var(--ink)]">
                {CONTRACT_TOTAL} Soroban contracts,
                <span className="italic pt-grad"> charted.</span>
              </h2>
              <p className="mt-5 text-[var(--muted)] leading-relaxed">
                A full catalog of audited Rust/Wasm contracts powering the protocol — each
                one independently deployable and verifiable on the Stellar ledger.
              </p>
            </div>
            <div className="pt-mono text-right text-[0.6rem] text-[var(--faint)] leading-relaxed">
              <div className="text-[var(--signal-dim)]">catalog ◦ PSR-INDEX</div>
              <div>{CATALOG.length} subsystems</div>
              <div>{CONTRACT_TOTAL} modules online</div>
            </div>
          </Reveal>

          <div className="mt-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {CATALOG.map((cat, i) => (
              <Reveal key={cat.group} delay={(i % 3) * 70}>
                <div className="pt-card h-full p-5" onMouseMove={spotlight}>
                  <div className="flex items-center justify-between">
                    <h3 className="text-base font-semibold text-[var(--ink)]">{cat.group}</h3>
                    <span className="pt-mono text-[0.58rem] text-[var(--signal-dim)] border border-[var(--line-bright)] rounded-md px-1.5 py-0.5">
                      {cat.code}
                    </span>
                  </div>
                  <div className="mt-4 space-y-0.5">
                    {cat.items.map((name, j) => (
                      <div key={name} className="pt-cat-item">
                        <span className="pt-dot" />
                        <span className="pt-mono text-[0.58rem] text-[var(--faint)] w-12 flex-none">
                          {cat.code}-{String(j + 1).padStart(2, "0")}
                        </span>
                        <span className="text-sm text-[var(--muted)] truncate">{name}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </Reveal>
            ))}
          </div>
        </div>
      </section>

      {/* ================= FINAL CTA ================= */}
      <section className="relative px-4 sm:px-6 lg:px-8 pb-28 pt-8">
        <Reveal>
          <div className="mx-auto max-w-5xl relative overflow-hidden rounded-3xl border border-[rgba(94,234,255,0.25)] px-6 sm:px-12 py-16 text-center">
            <div
              className="pt-sweep"
              aria-hidden
              style={{
                top: "-50%",
                left: "50%",
                transform: "translateX(-50%)",
                width: "560px",
                height: "560px",
                opacity: 0.5,
              }}
            />
            <div className="relative">
              <div className="flex justify-center">
                <span className="pt-icon">
                  <Satellite className="w-[22px] h-[22px]" strokeWidth={1.6} />
                </span>
              </div>
              <h2 className="pt-display mt-7 text-5xl sm:text-6xl text-[var(--ink)]">
                Tune in to the <span className="italic pt-grad">signal.</span>
              </h2>
              <p className="mx-auto mt-5 max-w-xl text-[var(--muted)] leading-relaxed">
                Connect a Stellar wallet and start broadcasting campaigns or earning as a
                publisher in minutes. The ledger is already listening.
              </p>
              <div className="mt-9 flex flex-wrap items-center justify-center gap-4">
                <Link href="/advertiser" className="pt-btn">
                  Launch a campaign <ArrowRight className="w-4 h-4" />
                </Link>
                <Link href="/governance" className="pt-btn-ghost">
                  Explore governance <Sparkles className="w-4 h-4" />
                </Link>
              </div>
            </div>
          </div>
        </Reveal>
      </section>

      {/* ================= FOOTER ================= */}
      <footer className="relative px-4 sm:px-6 lg:px-8 pb-14">
        <div className="mx-auto max-w-6xl">
          <div className="pt-beamline mb-10" />
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            <div className="col-span-2 md:col-span-1">
              <div className="flex items-center gap-2">
                <span className="w-8 h-8 rounded-lg grid place-items-center bg-gradient-to-br from-[var(--signal)] to-[var(--violet)] text-[#04050d]">
                  <Zap className="w-5 h-5" />
                </span>
                <span className="pt-display text-2xl text-[var(--ink)]">PulsarTrack</span>
              </div>
              <p className="mt-4 text-sm text-[var(--faint)] leading-relaxed max-w-xs">
                Privacy-preserving, blockchain-powered ad tracking on the Stellar network.
              </p>
            </div>

            {[
              {
                h: "Platform",
                links: [
                  ["Advertiser", "/advertiser"],
                  ["Publisher", "/publisher"],
                  ["Governance", "/governance"],
                ],
              },
              {
                h: "Protocol",
                links: [
                  ["Smart contracts", "/governance"],
                  ["Privacy layer", "/"],
                  ["Security", "/"],
                ],
              },
              {
                h: "Network",
                links: [
                  ["Stellar", "https://stellar.org"],
                  ["Soroban", "https://soroban.stellar.org"],
                  ["Freighter", "https://freighter.app"],
                ],
              },
            ].map((col) => (
              <div key={col.h}>
                <div className="pt-mono text-[0.6rem] text-[var(--signal-dim)] mb-4">{col.h}</div>
                <ul className="space-y-2.5">
                  {col.links.map(([label, href]) => (
                    <li key={label}>
                      <a
                        href={href}
                        className="group inline-flex items-center gap-1 text-sm text-[var(--muted)] hover:text-[var(--ink)] transition-colors"
                      >
                        {label}
                        <ArrowUpRight className="w-3 h-3 opacity-0 -translate-x-1 group-hover:opacity-100 group-hover:translate-x-0 transition-all" />
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>

          <div className="mt-12 flex flex-wrap items-center justify-between gap-4 pt-mono text-[0.58rem] text-[var(--faint)]">
            <span>© 2026 PulsarTrack · all signals reserved</span>
            <span className="flex items-center gap-2">
              <span className="pt-live-dot" /> built on Stellar · Soroban
            </span>
          </div>
        </div>
      </footer>
    </div>
  );
}
