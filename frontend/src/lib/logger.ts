/**
 * Minimal shared logging utility (issue #827). A thin wrapper around
 * console.* for now, so every error-handling call site funnels through one
 * place that can later be pointed at a real error-tracking service with a
 * single change here instead of touching every call site again.
 *
 * Convention: use `logger.error` for anything silently logged, and pair it
 * with a `useToast()` call at the call site when the error should also be
 * user-facing — this file does not decide that, the call site does.
 */
export const logger = {
  error(message: string, context?: unknown): void {
    console.error(message, context);
  },
  warn(message: string, context?: unknown): void {
    console.warn(message, context);
  },
};
