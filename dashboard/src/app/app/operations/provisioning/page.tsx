import Link from 'next/link';
import { notFound } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getCurrentUserContext } from '@/server/organization';
import { clampOperationsItemsLimit } from '@/services/reconciliation/OperationsDrilldownWindow';
import {
  buildOperationsQueueHref,
  buildProvisioningQueueWhere,
  normalizeOperationsEnumFilter,
  normalizeOperationsSearchQuery,
} from '@/services/reconciliation/OperationsQueueFilters';
import RetryProvisioningOrdersButton from '../RetryProvisioningOrdersButton';
import RetryProvisioningQueueButton from '../RetryProvisioningQueueButton';

export const dynamic = 'force-dynamic';

export default async function ProvisioningQueuePage(
  { searchParams }: { searchParams?: Promise<Record<string, string | string[] | undefined>> },
) {
  const userContext = await getCurrentUserContext();
  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  const params = searchParams ? await searchParams : {};
  const rawLimit = typeof params.limit === 'string' ? params.limit : undefined;
  const limit = clampOperationsItemsLimit(rawLimit);
  const query = normalizeOperationsSearchQuery(typeof params.q === 'string' ? params.q : undefined);
  const provider = normalizeOperationsEnumFilter(
    typeof params.provider === 'string' ? params.provider : undefined,
    ['hetzner', 'digitalocean', 'shared'] as const,
  );
  const timing = normalizeOperationsEnumFilter(
    typeof params.timing === 'string' ? params.timing : undefined,
    ['due', 'scheduled'] as const,
  );
  const now = new Date();

  const orders = await prisma.provisioningOrder.findMany({
    where: buildProvisioningQueueWhere({
      provider,
      query,
      timing,
      now,
    }),
    orderBy: [{ nextAttemptAt: 'asc' }, { updatedAt: 'asc' }],
    take: limit,
    select: {
      id: true,
      provider: true,
      status: true,
      currentStep: true,
      attemptCount: true,
      nextAttemptAt: true,
      errorMessage: true,
      updatedAt: true,
      endpoint: {
        select: {
          id: true,
          name: true,
          status: true,
          cloudProvider: true,
          region: true,
        },
      },
      organization: {
        select: {
          id: true,
          name: true,
        },
      },
    },
  });

  return (
    <div className="space-y-8">
      <div>
        <Link href="/app/operations" className="text-sm text-gray-400 hover:text-white">
          Back to Operations
        </Link>
        <h1 className="text-2xl font-bold tracking-tight mt-3">Pending Provisioning Queue</h1>
        <p className="text-gray-400 mt-1">Showing up to {limit} provisioning orders that still require operator visibility or automatic retries.</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', {})}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            provider === 'all' && timing === 'all' && !query
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          All
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', { timing: 'due' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            timing === 'due'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Due Now
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', { timing: 'scheduled' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            timing === 'scheduled'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Scheduled
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', { provider: 'hetzner' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            provider === 'hetzner'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Hetzner
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', { provider: 'digitalocean' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            provider === 'digitalocean'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          DigitalOcean
        </Link>
        <Link
          href={buildOperationsQueueHref('/app/operations/provisioning', { provider: 'shared' })}
          className={`rounded-full px-3 py-1.5 text-xs font-medium transition-colors ${
            provider === 'shared'
              ? 'bg-[#00E599] text-black'
              : 'border border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-gray-300 hover:text-white'
          }`}
        >
          Shared
        </Link>
      </div>
      <form className="grid grid-cols-1 md:grid-cols-5 gap-3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
        <input
          type="text"
          name="q"
          defaultValue={query ?? ''}
          placeholder="Search endpoint or organization"
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        />
        <select
          name="provider"
          defaultValue={provider}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All providers</option>
          <option value="hetzner">Hetzner</option>
          <option value="digitalocean">DigitalOcean</option>
          <option value="shared">Shared</option>
        </select>
        <select
          name="timing"
          defaultValue={timing}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="all">All retry timing</option>
          <option value="due">Due now</option>
          <option value="scheduled">Scheduled later</option>
        </select>
        <select
          name="limit"
          defaultValue={String(limit)}
          className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-3 py-2 text-sm text-white"
        >
          <option value="25">25 rows</option>
          <option value="50">50 rows</option>
          <option value="100">100 rows</option>
          <option value="200">200 rows</option>
        </select>
        <div className="flex gap-2">
          <button type="submit" className="bg-[#00E599] hover:bg-[#00cc88] text-black px-4 py-2 rounded-md text-sm font-bold transition-colors">
            Apply
          </button>
          <Link href="/app/operations/provisioning" className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white px-4 py-2 rounded-md text-sm font-medium transition-colors">
            Reset
          </Link>
        </div>
      </form>
      <div className="flex justify-end">
        <RetryProvisioningOrdersButton orderIds={orders.map((order) => order.id)} />
      </div>

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden">
        {orders.length === 0 ? (
          <div className="p-6 text-sm text-gray-400">No pending provisioning orders.</div>
        ) : (
          <div className="divide-y divide-[var(--color-dark-border)]">
            {orders.map((order) => (
              <div key={order.id} className="p-6 space-y-3">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <Link href={`/app/endpoints/${order.endpoint.id}`} className="text-base font-medium text-white hover:text-[#00E599]">
                      {order.endpoint.name}
                    </Link>
                    <p className="text-xs text-gray-500 mt-1">
                      Org: {order.organization.name} • Provider: {order.provider} • Endpoint status: {order.endpoint.status}
                    </p>
                  </div>
                  <span className="text-xs text-gray-500">Order #{order.id}</span>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-300">
                  <div>Step: {order.currentStep}</div>
                  <div>Attempts: {order.attemptCount}</div>
                  <div>Updated: {order.updatedAt.toLocaleString()}</div>
                </div>
                {order.nextAttemptAt && (
                  <p className="text-xs text-gray-500">Next retry: {order.nextAttemptAt.toLocaleString()}</p>
                )}
                {order.errorMessage && (
                  <p className="text-sm text-red-300">{order.errorMessage}</p>
                )}
                <div className="flex flex-wrap gap-2">
                  <Link href={`/app/operations/provisioning/${order.id}`} className="inline-flex text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                    View Details
                  </Link>
                  <RetryProvisioningQueueButton orderId={order.id} />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
