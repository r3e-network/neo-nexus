import { notFound } from 'next/navigation';
import { getCurrentUserContext } from '@/server/organization';
import OperationsClient from './OperationsClient';

export const dynamic = 'force-dynamic';

export default async function OperationsPage() {
  const userContext = await getCurrentUserContext();

  if (!userContext || userContext.role !== 'operator') {
    notFound();
  }

  return <OperationsClient />;
}
