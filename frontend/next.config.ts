import type { NextConfig } from "next";
import { withSentryConfig } from "@sentry/nextjs";

const nextConfig: NextConfig = {
  output: "standalone",
  serverExternalPackages: ["@stellar/stellar-sdk", "@stellar/stellar-base"],
};

export default withSentryConfig(nextConfig, {
  silent: true,
  disableLogger: true,
});
