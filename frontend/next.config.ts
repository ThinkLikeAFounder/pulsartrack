import type { NextConfig } from "next";
import { withSentryConfig } from "@sentry/nextjs/config";

const nextConfig: NextConfig = {
  output: "standalone",
  serverExternalPackages: ["@stellar/stellar-sdk", "@stellar/stellar-base"],
  experimental: {
    agentUpgrade: "security",
  },
};

export default withSentryConfig(nextConfig, {
  silent: true,
  webpack: {
    treeshake: {
      removeDebugLogging: true,
    },
  },
});
