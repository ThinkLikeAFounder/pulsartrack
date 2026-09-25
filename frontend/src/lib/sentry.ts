import type { Event } from "@sentry/nextjs";
import { scrubEvent as scrubEventLib } from "./sentry-scrubber";

export function scrubEvent<T extends Event>(event: T): T {
  return scrubEventLib(event);
}

