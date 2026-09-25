/**
 * Root loading state (Next.js App Router `loading.tsx`).
 *
 * Shown as a React Suspense fallback while any page in this route segment is
 * streaming or performing server-side data fetching. The skeleton matches the
 * general app layout (header height + content area) to avoid layout shift.
 */
export default function Loading() {
  return (
    <div
      className="min-h-screen flex flex-col"
      role="status"
      aria-label="Loading page"
    >
      {/* Content skeleton */}
      <div className="flex-1 px-4 py-8 max-w-7xl mx-auto w-full space-y-6 animate-pulse">
        {/* Page title skeleton */}
        <div className="h-8 w-48 bg-gray-200 dark:bg-gray-700 rounded-lg" />

        {/* Stats row skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div
              key={i}
              className="h-24 bg-gray-200 dark:bg-gray-700 rounded-2xl"
            />
          ))}
        </div>

        {/* Main content skeleton */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 h-64 bg-gray-200 dark:bg-gray-700 rounded-2xl" />
          <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded-2xl" />
        </div>

        {/* Table skeleton */}
        <div className="space-y-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div
              key={i}
              className="h-12 bg-gray-200 dark:bg-gray-700 rounded-lg"
            />
          ))}
        </div>
      </div>

      <span className="sr-only">Loading…</span>
    </div>
  );
}
