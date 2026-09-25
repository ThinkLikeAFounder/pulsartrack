import Link from "next/link";
import type { Metadata } from "next";
import { Home, SearchX } from "lucide-react";

export const metadata: Metadata = {
  title: "Page Not Found",
  description: "The page you are looking for does not exist.",
};

/**
 * Custom 404 page (Next.js App Router `not-found.tsx`).
 *
 * Rendered when `notFound()` is called anywhere in the app, or when Next.js
 * cannot match the incoming URL to any route. Displays within the root layout
 * so the header and navigation remain visible.
 */
export default function NotFoundPage() {
  return (
    <div className="min-h-[60vh] flex items-center justify-center px-4 py-16">
      <div className="max-w-md w-full text-center">
        <div className="w-20 h-20 bg-indigo-50 dark:bg-indigo-900/20 rounded-full flex items-center justify-center mx-auto mb-6">
          <SearchX
            className="w-10 h-10 text-indigo-600 dark:text-indigo-400"
            aria-hidden="true"
          />
        </div>

        <p className="text-6xl font-bold text-indigo-600 dark:text-indigo-400 mb-2">
          404
        </p>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-3">
          Page not found
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          The page you&apos;re looking for doesn&apos;t exist or has been moved.
          Check the URL or head back to the dashboard.
        </p>

        <Link
          href="/"
          className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium"
        >
          <Home className="w-4 h-4" aria-hidden="true" />
          Back to Home
        </Link>
      </div>
    </div>
  );
}
