import MarketplaceClient from './MarketplaceClient';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function MarketplacePage() {
  const userContext = await getCurrentUserContext();
  const billingPlan = userContext?.billingPlan ?? 'developer';

  return <MarketplaceClient billingPlan={billingPlan} />;
}
