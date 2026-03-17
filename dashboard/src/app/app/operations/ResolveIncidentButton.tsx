'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { resolveIncidentAction } from './actions';

export default function ResolveIncidentButton({ incidentId }: { incidentId: number }) {
  const router = useRouter();
  const [isResolving, setIsResolving] = useState(false);

  const handleResolve = async () => {
    setIsResolving(true);
    const result = await resolveIncidentAction(incidentId);
    if (result.success) {
      toast.success('Incident resolved.');
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to resolve incident.');
    }
    setIsResolving(false);
  };

  return (
    <button
      onClick={handleResolve}
      disabled={isResolving}
      className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
    >
      {isResolving ? 'Resolving…' : 'Resolve Incident'}
    </button>
  );
}
