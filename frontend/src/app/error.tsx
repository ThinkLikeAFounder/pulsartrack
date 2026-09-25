"use client";

import * as Sentry from "@sentry/nextjs";
import Link from "next/link";
import { useEffect } from "react";
import { AlertTriangle, Home, RefreshCw } from "lucide-react";
import { logger } from "@/lib/logger";

interface ErrorPageProps {
  error: Error & { digest?: string };
  reset: () => void;
}

/**
 * Route-level error boundary (Next.js App Router `error.tsx`).
 *
 * Catches errors thrown while rendering any page that is a sibling or
 * descendant of this file. Unlike `global-error.tsx`, this renders inside
 * the root layout so the header and navigation remain visible.
 *
 * - Reports the error to Sentry.
 * - Provides a "Try Again" button that calls `reset()` to re-render the segment.
 * - Provides a "Return Home" link for when retrying would not help.
 */
export default function ErrorPage({ error, reset }: ErrorPageProps) {
  useEffect(() => {
    Sentry.captureException(error);
    logger.error("Route error boundary caught an error", {
      message: error.message,
      digest: error.digest,
    });
  }, [error]);

  return (
    <div className="min-h-[60vh] flex items-center justify-center px-4 py-16">
      <div className="max-w-md w-full bg-white dark:bg-gray-900 rounded-2xl shadow-sm border border-red-100 dark:border-red-900/30 p-8 text-center">
        <div className="w-16 h-16 bg-red-50 dark:bg-red-900/20 rounded-full flex items-center justify-center mx-auto mb-6">
          <AlertTriangle
            className="w-8 h-8 text-red-600 dark:text-red-400"
            aria-hidden="true"
          />
        </div>

        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
          Something went wrong
        </h2>

        <p className="text-gray-600 dark:text-gray-400 mb-4">
          An unexpected error occurred while loading this page.
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

          <Link
            href="/"
            className="flex items-center justify-center gap-2 w-full py-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors font-medium"
          >
            <Home className="w-4 h-4" aria-hidden="true" />
            Return Home
          </Link>
        </div>

        {error.digest && (
          <p className="mt-4 text-xs text-gray-400 dark:text-gray-600">
            Error ID: {error.digest}
          </p>
        )}
      </div>
    </div>
  );
}
