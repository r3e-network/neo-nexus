import { getCurrentUserContext } from '@/server/organization';
import OverviewClient from './OverviewClient';

export const dynamic = 'force-dynamic';

export default async function OverviewPage() {
  const userContext = await getCurrentUserContext();

  return <OverviewClient showOperations={userContext?.role === 'operator'} />;
}
