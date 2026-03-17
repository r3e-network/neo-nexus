import type { Prisma } from '@prisma/client';

export function normalizeOperationsSearchQuery(value: string | undefined) {
  const trimmed = value?.trim() ?? '';
  return trimmed || null;
}

export function normalizeOperationsEnumFilter<T extends string>(
  value: string | undefined,
  allowed: readonly T[],
  fallback: T | 'all' = 'all',
) {
  if (!value) {
    return fallback;
  }

  return allowed.includes(value as T) ? (value as T) : fallback;
}

export function buildOperationsQueueHref(
  basePath: string,
  params: Record<string, string | number | null | undefined>,
) {
  const searchParams = new URLSearchParams();

  for (const [key, rawValue] of Object.entries(params).sort(([left], [right]) => left.localeCompare(right))) {
    if (rawValue === undefined || rawValue === null) {
      continue;
    }

    const value = String(rawValue).trim();
    if (!value || value === 'all') {
      continue;
    }

    searchParams.set(key, value);
  }

  const query = searchParams.toString();
  return query ? `${basePath}?${query}` : basePath;
}

export function buildProvisioningQueueWhere(input: {
  provider: 'all' | 'hetzner' | 'digitalocean' | 'shared';
  query: string | null;
  timing: 'all' | 'due' | 'scheduled';
  now: Date;
}): Prisma.ProvisioningOrderWhereInput {
  const filters: Prisma.ProvisioningOrderWhereInput[] = [
    { status: { notIn: ['ready', 'failed'] } },
  ];

  if (input.provider !== 'all') {
    filters.push({ provider: input.provider });
  }

  if (input.timing === 'due') {
    filters.push({
      OR: [
        { nextAttemptAt: null },
        { nextAttemptAt: { lte: input.now } },
      ],
    });
  } else if (input.timing === 'scheduled') {
    filters.push({ nextAttemptAt: { gt: input.now } });
  }

  if (input.query) {
    filters.push({
      OR: [
        { endpoint: { name: { contains: input.query, mode: 'insensitive' } } },
        { organization: { name: { contains: input.query, mode: 'insensitive' } } },
      ],
    });
  }

  return filters.length === 1 ? filters[0] : { AND: filters };
}

export function buildAlertQueueWhere(input: {
  query: string | null;
  snoozed: 'all' | 'snoozed' | 'unsnoozed';
  actionType: 'all' | 'email' | 'webhook';
  timing: 'all' | 'due' | 'scheduled';
  now: Date;
}): Prisma.AlertRuleWhereInput {
  const filters: Prisma.AlertRuleWhereInput[] = [
    {
      isActive: true,
      nextDeliveryAt: { not: null },
    },
  ];

  if (input.snoozed === 'snoozed') {
    filters.push({ snoozedUntil: { gt: input.now } });
  } else if (input.snoozed === 'unsnoozed') {
    filters.push({
      OR: [
        { snoozedUntil: null },
        { snoozedUntil: { lte: input.now } },
      ],
    });
  }

  if (input.actionType !== 'all') {
    filters.push({ actionType: input.actionType });
  }

  if (input.timing === 'due') {
    filters.push({ nextDeliveryAt: { lte: input.now } });
  } else if (input.timing === 'scheduled') {
    filters.push({ nextDeliveryAt: { gt: input.now } });
  }

  if (input.query) {
    filters.push({
      OR: [
        { name: { contains: input.query, mode: 'insensitive' } },
        { target: { contains: input.query, mode: 'insensitive' } },
        { endpoint: { name: { contains: input.query, mode: 'insensitive' } } },
        { endpoint: { organization: { name: { contains: input.query, mode: 'insensitive' } } } },
      ],
    });
  }

  return filters.length === 1 ? filters[0] : { AND: filters };
}

export function buildIncidentQueueWhere(input: {
  query: string | null;
  severity: 'all' | 'critical' | 'warning';
  snoozed: 'all' | 'snoozed' | 'unsnoozed';
  now: Date;
}): Prisma.AlertIncidentWhereInput {
  const filters: Prisma.AlertIncidentWhereInput[] = [
    { status: 'Open' },
  ];

  if (input.severity !== 'all') {
    filters.push({ severity: input.severity });
  }

  if (input.snoozed === 'snoozed') {
    filters.push({ alertRule: { snoozedUntil: { gt: input.now } } });
  } else if (input.snoozed === 'unsnoozed') {
    filters.push({
      OR: [
        { alertRule: { snoozedUntil: null } },
        { alertRule: { snoozedUntil: { lte: input.now } } },
      ],
    });
  }

  if (input.query) {
    filters.push({
      OR: [
        { message: { contains: input.query, mode: 'insensitive' } },
        { endpoint: { name: { contains: input.query, mode: 'insensitive' } } },
        { endpoint: { organization: { name: { contains: input.query, mode: 'insensitive' } } } },
        { alertRule: { name: { contains: input.query, mode: 'insensitive' } } },
      ],
    });
  }

  return filters.length === 1 ? filters[0] : { AND: filters };
}
