type MetadataRow = {
  label: string;
  value: string;
};

function labelize(key: string) {
  return key
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, (value) => value.toUpperCase());
}

export function formatEndpointActivityMetadata(metadata: unknown): MetadataRow[] {
  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) {
    return [];
  }

  return Object.entries(metadata)
    .filter(([, value]) => value !== null && value !== undefined)
    .map(([key, value]) => ({
      label: labelize(key),
      value: typeof value === 'string' ? value : JSON.stringify(value),
    }));
}
