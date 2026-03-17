type IncidentInput = {
  openedAt: string;
  resolvedAt: string | null;
  lastDeliveredAt: string | null;
  lastDeliveryError: string | null;
};

type ActivityInput = {
  id: number;
  category: string;
  action: string;
  status: string;
  message: string;
  createdAt: string;
};

export type IncidentTimelineItem =
  | {
      kind: 'incident';
      status: 'opened' | 'resolved';
      message: string;
      createdAt: string;
    }
  | {
      kind: 'delivery';
      status: 'success' | 'error';
      message: string;
      createdAt: string;
    }
  | {
      kind: 'activity';
      status: string;
      message: string;
      createdAt: string;
    };

export function buildIncidentTimeline(input: {
  incident: IncidentInput;
  activities: ActivityInput[];
}) {
  const items: IncidentTimelineItem[] = [
    {
      kind: 'incident',
      status: 'opened',
      message: 'Incident opened.',
      createdAt: input.incident.openedAt,
    },
    ...(input.incident.resolvedAt
      ? [{
          kind: 'incident' as const,
          status: 'resolved' as const,
          message: 'Incident resolved.',
          createdAt: input.incident.resolvedAt,
        }]
      : []),
    ...(input.incident.lastDeliveredAt
      ? [{
          kind: 'delivery' as const,
          status: 'success' as const,
          message: 'Alert delivery succeeded.',
          createdAt: input.incident.lastDeliveredAt,
        }]
      : []),
    ...(input.incident.lastDeliveryError
      ? [{
          kind: 'delivery' as const,
          status: 'error' as const,
          message: input.incident.lastDeliveryError,
          createdAt: input.incident.lastDeliveredAt ?? input.incident.openedAt,
        }]
      : []),
    ...input.activities.map((activity) => ({
      kind: 'activity' as const,
      status: activity.status,
      message: activity.message,
      createdAt: activity.createdAt,
    })),
  ];

  return items.sort((left, right) => (
    new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime()
  ));
}
