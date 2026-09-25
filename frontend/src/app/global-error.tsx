"use client";

import * as Sentry from "@sentry/nextjs";
import { useEffect } from "react";
import { AlertTriangle, RefreshCw, Home } from "lucide-react";
import { logger } from "@/lib/logger";

interface GlobalErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

/**
 * Global error boundary (Next.js App Router `global-error.tsx`).
 *
 * This is the last-resort boundary — it replaces the entire layout (including
 * the root `<html>` tag) when an error escapes all route-level boundaries.
 * Prefer adding route-level `error.tsx` files closer to where the error can
 * occur so the header and navigation remain intact.
 *
 * - Reports the error to Sentry.
 * - Provides a "Try Again" (`reset()`) button.
 * - Provides a "Return Home" hard-navigation link.
 */
export default function GlobalError({ error, reset }: GlobalErrorProps) {
  useEffect(() => {
    Sentry.captureException(error);
    logger.error("Global error boundary caught an unrecoverable error", {
      message: error.message,
      digest: error.digest,
    });
  }, [error]);

  return (
    <html lang="en">
      <body className="min-h-screen bg-gray-50 dark:bg-gray-950 flex items-center justify-center px-4 antialiased">
        <div className="max-w-md w-full bg-white dark:bg-gray-900 rounded-2xl shadow-sm border border-red-100 dark:border-red-900/30 p-8 text-center">
          <div className="w-16 h-16 bg-red-50 dark:bg-red-900/20 rounded-full flex items-center justify-center mx-auto mb-6">
            <AlertTriangle
              className="w-8 h-8 text-red-600 dark:text-red-400"
              aria-hidden="true"
            />
          </div>

          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
            Something went wrong
          </h1>

          <p className="text-gray-600 dark:text-gray-400 mb-4">
            An unexpected error has occurred. You can try refreshing the page or
            return to the home screen.
          </p>

          {error.message && (
            <p className="text-xs font-mono text-red-500 bg-red-50 dark:bg-red-900/20 dark:text-red-400 p-2 rounded mb-6 truncate">
              {error.message}
            </p>
          )}

          <div className="flex flex-col gap-3">
            <button
              onClick={reset}
              className="flex items-center justify-center gap-2 w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium"
            >
              <RefreshCw className="w-4 h-4" aria-hidden="true" />
              Try Again
            </button>

            {/* Hard navigation since the root layout may be broken */}
            <a
              href="/"
              className="flex items-center justify-center gap-2 w-full py-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors font-medium"
            >
              <Home className="w-4 h-4" aria-hidden="true" />
              Return Home
            </a>
          </div>

          {error.digest && (
            <p className="mt-4 text-xs text-gray-400 dark:text-gray-600">
              Error ID: {error.digest}
            </p>
          )}
        </div>
      </body>
    </html>
  );
}
