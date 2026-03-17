'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import toast from 'react-hot-toast';
import { resolveIncidentsAction } from './actions';

export default function ResolveIncidentsButton({ incidentIds }: { incidentIds: number[] }) {
  const router = useRouter();
  const [isResolving, setIsResolving] = useState(false);

  const handleResolve = async () => {
    setIsResolving(true);
    const result = await resolveIncidentsAction(incidentIds);
    if (result.success) {
      toast.success(`Resolved ${result.resolvedCount} incident${result.resolvedCount === 1 ? '' : 's'}.`);
      router.refresh();
    } else {
      toast.error('Failed to resolve incidents.');
    }
    setIsResolving(false);
  };

  return (
    <button
      onClick={handleResolve}
      disabled={isResolving || incidentIds.length === 0}
      className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
    >
      {isResolving ? 'Resolving…' : 'Resolve All Visible'}
    </button>
  );
}
