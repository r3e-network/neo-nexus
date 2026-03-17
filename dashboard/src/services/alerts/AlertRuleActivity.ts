type AlertActivityLike = {
  id: number;
  action: string;
  message: string;
  createdAt: Date;
  metadata: unknown;
};

export function filterAlertRuleActivities(
  alertRuleId: number,
  activities: AlertActivityLike[],
) {
  return activities.filter((activity) => {
    if (!activity.metadata || typeof activity.metadata !== 'object' || Array.isArray(activity.metadata)) {
      return false;
    }

    const metadata = activity.metadata as { alertRuleId?: unknown };
    return metadata.alertRuleId === alertRuleId;
  });
}
