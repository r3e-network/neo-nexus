const shimmer = "animate-pulse bg-slate-700/50 rounded";

export function StatSkeleton() {
  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="card">
          <div className="flex items-center justify-between">
            <div className="space-y-2 flex-1">
              <div className={`${shimmer} h-3 w-16`} />
              <div className={`${shimmer} h-7 w-10`} />
            </div>
            <div className={`${shimmer} w-12 h-12`} />
          </div>
        </div>
      ))}
    </div>
  );
}

export function TableRowSkeleton({ rows = 3 }: { rows?: number }) {
  return (
    <div className="space-y-3 py-2">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 px-4 py-3">
          <div className={`${shimmer} w-8 h-8 shrink-0`} />
          <div className="flex-1 space-y-2">
            <div className={`${shimmer} h-3 w-1/3`} />
            <div className={`${shimmer} h-2 w-1/5`} />
          </div>
          <div className={`${shimmer} h-3 w-16`} />
          <div className={`${shimmer} h-3 w-16`} />
          <div className={`${shimmer} h-6 w-20`} />
          <div className={`${shimmer} h-8 w-16`} />
        </div>
      ))}
    </div>
  );
}

export function CardSkeleton({ count = 3 }: { count?: number }) {
  return (
    <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="card space-y-4">
          <div className="flex items-start gap-4">
            <div className={`${shimmer} w-10 h-10 shrink-0`} />
            <div className="flex-1 space-y-2">
              <div className={`${shimmer} h-4 w-1/2`} />
              <div className={`${shimmer} h-3 w-3/4`} />
              <div className={`${shimmer} h-3 w-1/3`} />
            </div>
          </div>
          <div className={`${shimmer} h-9 w-full`} />
        </div>
      ))}
    </div>
  );
}
