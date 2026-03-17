type IncidentTransitionInput = {
  triggered: boolean;
  hasOpenIncident: boolean;
  severity: string;
  message: string;
  now: Date;
};

export function getIncidentTransition(input: IncidentTransitionInput) {
  if (input.triggered && !input.hasOpenIncident) {
    return {
      action: 'open' as const,
      severity: input.severity,
      message: input.message,
      openedAt: input.now,
    };
  }

  if (!input.triggered && input.hasOpenIncident) {
    return {
      action: 'resolve' as const,
      resolvedAt: input.now,
    };
  }

  return {
    action: 'none' as const,
  };
}

export function buildManualIncidentResolution(now: Date) {
  return {
    status: 'Resolved' as const,
    resolvedAt: now,
  };
}
