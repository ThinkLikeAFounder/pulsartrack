import * as Sentry from "@sentry/nextjs";
import { validateRequiredEnv } from "./lib/stellar-config";

export async function register() {
  if (process.env.NEXT_RUNTIME === "nodejs") {
    await import("../sentry.server.config");

    // Validate required environment variables at startup in production so
    // missing contract addresses or network config fail loudly instead of
    // silently becoming empty strings (#915).
    if (process.env.NODE_ENV === "production") {
      validateRequiredEnv();
    }
  }

  if (process.env.NEXT_RUNTIME === "edge") {
    await import("../sentry.edge.config");
  }
}

export const onRequestError = Sentry.captureRequestError;
