import "dotenv/config";
import * as Sentry from "@sentry/node";
import { scrubEvent } from "./lib/sentry-scrubber";

Sentry.init({
  dsn: process.env.SENTRY_DSN,
  enabled: Boolean(process.env.SENTRY_DSN),
  environment: process.env.NODE_ENV,
  sendDefaultPii: false,
  beforeSend(event) {
    return scrubEvent(event);
  },
});

