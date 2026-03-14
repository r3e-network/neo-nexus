'use server';

import { createClient } from '@/utils/supabase/server';
import { revalidatePath } from 'next/cache';

export async function createEndpointAction(formData: {
  name: string;
  network: string;
  type: string;
  clientEngine: string;
  provider: string;
  region: string;
  syncMode: string;
}) {
  const isSupabaseConfigured = process.env.NEXT_PUBLIC_SUPABASE_URL && process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

  if (isSupabaseConfigured) {
    const supabase = await createClient();
    
    // Generate a mock URL based on config
    const randomId = Math.random().toString(36).substring(2, 8);
    const url = formData.type === 'dedicated' 
      ? `https://node-${formData.region}-${randomId}.neonexus.io/v1`
      : `https://${formData.network}.neonexus.io/v1/${randomId}`;

    const { data, error } = await supabase
      .from('endpoints')
      .insert([
        {
          name: formData.name,
          network: formData.network === 'mainnet' ? 'N3 Mainnet' : 'N3 Testnet',
          type: formData.type.charAt(0).toUpperCase() + formData.type.slice(1),
          url: url,
          status: 'Syncing', // Starts in syncing state
          requests: '0',
          // Extended info can be stored in a JSONB 'config' column if we added one, 
          // or just ignored for now as MVP.
        }
      ])
      .select()
      .single();

    if (error) {
      console.error('Error creating endpoint:', error);
      return { success: false, error: error.message };
    }

    revalidatePath('/endpoints');
    return { success: true, id: data.id };
  } else {
    // Mock successful creation for local dev without Supabase
    console.log('Supabase not configured. Mocking endpoint creation:', formData);
    return { success: true, id: 1 }; // Return mock ID 1
  }
}
