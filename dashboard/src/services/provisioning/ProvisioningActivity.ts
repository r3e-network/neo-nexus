type ProvisioningActivityLike = {
  id: number;
  category: string;
  message: string;
  createdAt: string;
};

export function filterProvisioningActivities(
  activities: ProvisioningActivityLike[],
) {
  return activities
    .filter((activity) => activity.category === 'provisioning')
    .sort((left, right) => (
      new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime()
    ));
}
