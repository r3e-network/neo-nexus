import { createClient } from '@/utils/supabase/server';
import EndpointDetailsClient from './EndpointDetailsClient';
import { Endpoint } from '../EndpointsList';

const MOCK_ENDPOINTS: Endpoint[] = [
  {
    id: 1,
    name: 'Default Shared Endpoint',
    network: 'N3 Mainnet',
    type: 'Shared',
    url: 'https://mainnet.neonexus.io/v1/a8f9c2e4-...',
    status: 'Active',
    requests: '1.2M',
  },
  {
    id: 2,
    name: 'GameFi Indexer Node',
    network: 'N3 Mainnet',
    type: 'Dedicated',
    url: 'https://node-tokyo-01.neonexus.io/v1/xyz...',
    status: 'Active',
    requests: '4.5M',
  },
  {
    id: 3,
    name: 'Testnet Integration',
    network: 'N3 Testnet',
    type: 'Dedicated',
    url: 'https://node-frankfurt-test.neonexus.io/v1/abc...',
    status: 'Syncing',
    requests: '0',
  }
];

export default async function EndpointDetailsPage({ params }: { params: { id: string } }) {
  const { id } = await params;
  let endpoint: Endpoint | null = null;

  try {
    if (process.env.NEXT_PUBLIC_SUPABASE_URL && process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY) {
      const supabase = await createClient();
      const { data, error } = await supabase
        .from('endpoints')
        .select('*')
        .eq('id', id)
        .single();

      if (!error && data) {
        endpoint = data as Endpoint;
      }
    }
  } catch (error) {
    console.warn('Supabase not configured or failed, falling back to mock data.', error);
  }

  // Fallback to mock data if not found in Supabase or Supabase is not configured
  if (!endpoint) {
    endpoint = MOCK_ENDPOINTS.find(ep => ep.id.toString() === id) || MOCK_ENDPOINTS[0]; // fallback to first mock if ID mismatch
  }

  return <EndpointDetailsClient endpoint={endpoint} />;
}
