import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  serverExternalPackages: ["@stellar/stellar-sdk", "@stellar/stellar-base"],
};

export default nextConfig;
