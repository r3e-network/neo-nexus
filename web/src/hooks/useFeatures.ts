import { useEffect, useState } from 'react';

interface Features {
  hermesAgent: boolean;
  neox: boolean;
}

const DEFAULT: Features = { hermesAgent: false, neox: false };

let cached: Features | null = null;
const subscribers = new Set<(f: Features) => void>();

async function fetchFeatures(): Promise<Features> {
  try {
    const res = await fetch('/api/health');
    if (!res.ok) return DEFAULT;
    const body = (await res.json()) as { features?: Features };
    return body.features ?? DEFAULT;
  } catch {
    return DEFAULT;
  }
}

export function useFeatures(): Features {
  const [features, setFeatures] = useState<Features>(cached ?? DEFAULT);

  useEffect(() => {
    if (cached) return;
    const subscriber = (f: Features) => setFeatures(f);
    subscribers.add(subscriber);
    void fetchFeatures().then((f) => {
      cached = f;
      subscribers.forEach((s) => s(f));
    });
    return () => {
      subscribers.delete(subscriber);
    };
  }, []);

  return features;
}
