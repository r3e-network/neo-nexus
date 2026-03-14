import { prisma } from '@/utils/prisma';
import SecurityClient, { ApiKeyType } from './SecurityClient';

export default async function SecurityPage() {
  let keys: ApiKeyType[] = [
    {
      id: 'mock-id-1',
      name: 'Production DApp Key',
      keyHash: 'fakehashfakehashfakehash',
      createdAt: new Date(),
      isActive: true,
    }
  ];

  if (process.env.DATABASE_URL) {
    try {
      const data = await prisma.apiKey.findMany({
        orderBy: { createdAt: 'asc' }
      });
      if (data && data.length > 0) {
        keys = data.map(k => ({
          id: k.id,
          name: k.name,
          keyHash: k.keyHash,
          createdAt: k.createdAt,
          isActive: k.isActive
        }));
      }
    } catch (e) {
      console.warn("Failed to fetch API keys from DB", e);
    }
  }

  return <SecurityClient initialKeys={keys} />;
}
