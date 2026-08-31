import * as Sentry from "@sentry/nextjs";
import { scrubEvent } from "./src/lib/sentry";

Sentry.init({
  dsn: process.env.SENTRY_DSN,
  enabled: Boolean(process.env.SENTRY_DSN),
  environment: process.env.NODE_ENV,
  sendDefaultPii: false,
  beforeSend: scrubEvent,
});
