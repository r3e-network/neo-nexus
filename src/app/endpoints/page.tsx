import { createClient } from '@/utils/supabase/server';
import EndpointsList, { Endpoint } from './EndpointsList';

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

export default async function EndpointsPage() {
  let endpoints: Endpoint[] = MOCK_ENDPOINTS;

  try {
    // Attempt to fetch from Supabase if env vars are present
    if (process.env.NEXT_PUBLIC_SUPABASE_URL && process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY) {
      const supabase = await createClient();
      const { data, error } = await supabase
        .from('endpoints')
        .select('*')
        .order('created_at', { ascending: false });

      if (!error && data && data.length > 0) {
        endpoints = data as Endpoint[];
      } else if (error) {
        console.warn('Supabase fetch failed, falling back to mock data:', error.message);
      }
    }
  } catch (error) {
    console.warn('Supabase not configured or failed, falling back to mock data.', error);
  }

  return <EndpointsList endpoints={endpoints} />;
}
