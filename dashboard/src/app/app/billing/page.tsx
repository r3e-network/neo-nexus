import BillingClient from './BillingClient';
import { getCurrentUserContext } from '@/server/organization';
import { getPublicCryptoBillingConfig } from '@/services/billing/CryptoBillingService';

export const dynamic = 'force-dynamic';

export default async function BillingPage() {
  const userContext = await getCurrentUserContext();
  const billingPlan = userContext?.billingPlan ?? 'developer';
  const cryptoBillingConfig = getPublicCryptoBillingConfig();

  return <BillingClient billingPlan={billingPlan} cryptoBillingConfig={cryptoBillingConfig} />;
}
